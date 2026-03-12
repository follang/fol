# FOL Resolver Plan

Last rebuilt: 2026-03-12
Scope: `fol-resolver` plus the minimal parser/CLI/doc changes required to support name resolution cleanly

## 0. Purpose

- The stream, lexer, parser, and diagnostics layers are already in strong shape.
- The next compiler phase is whole-program name resolution.
- This plan defines the new crate, its contract, its limits, its data model, its test strategy, and the slice order for implementing it.
- This plan is intentionally limited to resolution. It does not plan type checking, ownership, runtime, interpreter work, backend work, or code generation.

## 1. Current Baseline

- Implemented and green today:
- `fol-stream`
- `fol-lexer`
- `fol-parser`
- `fol-diagnostics`
- root CLI parse-and-report driver
- Missing today:
- whole-program name resolution
- whole-program type checking
- ownership/borrowing enforcement
- runtime/backend/codegen work

## 2. Vision

We add a new crate named `fol-resolver` that sits directly after parsing:

`fol-stream -> fol-lexer -> fol-parser -> fol-resolver -> later semantic phases`

`fol-resolver` is responsible for turning syntax-only AST into a semantically indexed program:

- collect declarations across the whole source set
- build lexical scopes
- resolve imports introduced by `use`
- resolve plain and qualified names used in value, call, type, and inquiry positions
- assign stable symbol IDs
- report unresolved, ambiguous, duplicate, shadowing, and unsupported-resolution diagnostics

It must not:

- infer or check expression types
- prove method/field validity from receiver types
- evaluate ownership, borrowing, lifetimes, aliasing, or effects
- execute code
- lower to backend IR

## 3. Hard Design Rules

### 3.1 Parser AST Stays Syntax-First

- `fol-parser` remains the syntax boundary.
- We do not overload parser AST variants with semantic meaning just to make resolution convenient.
- `AstNode::syntactic_type_hint()` stays a parser-local helper and must not become a resolver shortcut.

### 3.2 Resolver Owns Semantic State

- `fol-resolver` owns symbol tables, scopes, references, import targets, and resolution results.
- Resolution data should live in resolver-owned structs rather than being baked back into parser AST nodes.

### 3.3 Use Stream Identity, Not Ad-Hoc Path Strings

- Resolver import and module lookup must use canonical source identity from `fol-stream`.
- Namespace/package comparisons must use stream-derived package + namespace data, not raw file-call spelling.

### 3.4 Exact Diagnostics Are Mandatory

- Resolver errors must carry concrete source locations.
- No silent fallback to unresolved string matching, no “best effort” hidden coercions, and no generic “failed to resolve something” diagnostics.

### 3.5 Unsupported Semantics Must Fail Explicitly

- If a syntax form is parser-valid but not yet resolver-supported, the resolver must emit an explicit targeted diagnostic.
- It must never silently skip or erase semantic work.

## 4. Critical Constraint To Solve First

Successful parser AST nodes currently do not retain spans or file origins.

That is a real blocker for `fol-resolver`, because resolution diagnostics need to point at:

- the unresolved identifier token
- the duplicate declaration token
- the ambiguous qualified path
- the unsupported import kind
- the shadowing declaration site

So the resolver phase must begin with a parser-adjacent support layer that exposes stable syntax origins for successfully parsed nodes and references.

## 5. External Contract For `fol-resolver`

### 5.1 Inputs

- parsed syntax tree from `fol-parser`
- source inventory from `fol-stream`
- exact source/file/package/namespace identity for every parsed top-level node

### 5.2 Outputs

- a resolver-owned `ResolvedProgram`
- stable symbol IDs
- stable scope IDs
- stable reference IDs
- explicit import target records
- explicit resolution diagnostics

### 5.3 CLI Integration

- The root CLI should run resolver immediately after parser success.
- Resolver errors must flow into `fol-diagnostics` through the existing `Glitch` path.
- A parse-clean program is no longer “successful” if resolution fails.

## 6. Proposed Crate Shape

Workspace addition:

- `fol-resolver`

Expected crate surface:

- `fol-resolver/src/lib.rs`
- `fol-resolver/src/errors.rs`
- `fol-resolver/src/ids.rs`
- `fol-resolver/src/model.rs`
- `fol-resolver/src/scopes.rs`
- `fol-resolver/src/symbols.rs`
- `fol-resolver/src/imports.rs`
- `fol-resolver/src/collect.rs`
- `fol-resolver/src/resolve.rs`
- `fol-resolver/src/traverse.rs`

The exact file split can move, but these responsibilities must exist.

## 7. Resolver Data Model

### 7.1 IDs

- `SourceUnitId`
- `ScopeId`
- `SymbolId`
- `ReferenceId`
- `ImportId`
- `SyntaxNodeId`

IDs must be small, copyable, stable within one resolution run, and testable.

### 7.2 Syntax Origins

Needed support structure:

- `SyntaxOrigin { file, line, column, length }`
- `SyntaxIndex` mapping resolver-visible syntax handles to origins

Preferred implementation direction:

- keep `AstParser::parse()` for compatibility
- add a second parser-facing entry point such as `parse_indexed()` or a wrapper result like `ParsedProgram`
- `ParsedProgram` should contain:
- `root: AstNode`
- `syntax_index`
- enough source-local origin data for resolver diagnostics and file regrouping

### 7.3 Symbol Model

Each symbol should record at least:

- `id`
- `name`
- canonical comparison key
- `kind`
- defining scope
- defining source unit
- declaration origin
- optional export/visibility metadata

Planned `SymbolKind` families:

- package root
- namespace root
- import alias
- value binding
- label binding
- routine symbol
- type symbol
- alias symbol
- generic parameter
- parameter
- capture
- destructure binding
- loop binder
- rolling binder
- standard/segment/definition symbol where syntactically named

### 7.4 Scope Model

Scopes should be explicit and hierarchical.

Planned scope kinds:

- program root
- source-unit root
- routine declaration
- anonymous routine
- block
- `when` case body
- loop body
- rolling/comprehension binding region
- implementation body if needed as a distinct declaration scope

Each scope should track:

- parent scope
- declared symbols by canonical key
- imports visible in that scope
- source unit affiliation

### 7.5 Reference Model

Every resolvable use-site should become a reference record rather than a mutated AST node.

Planned `ReferenceKind` families:

- identifier expression
- qualified identifier
- free function/procedure/logical call
- qualified function/procedure/logical call
- named type reference
- qualified type reference
- inquiry target reference
- import target reference

Each reference should store:

- syntax handle
- reference kind
- textual spelling or path segments
- enclosing scope
- resolved target or failure state

### 7.6 Import Model

Each `use` declaration should lower into an explicit import record:

- alias name symbol
- source-kind form from `path_type`
- parsed `path_segments`
- enclosing scope
- resolved target status

## 8. Resolution Semantics To Implement

### 8.1 Whole-Program Declaration Collection

- Collect all top-level named declarations before resolving top-level bodies.
- Top-level declaration order must not block references to later declarations in the same resolution domain.
- Duplicate detection must use canonical identifier comparison, not raw spelling only.

### 8.2 Lexical Value Resolution

- Parameters and captures are visible across the whole routine body.
- Local declarations are visible from declaration point forward, not retroactively.
- Inner scopes may shadow outer locals/imports/top-level names only where the language permits it.
- Duplicate local declarations in the same scope are errors.

### 8.3 Type Name Resolution

- Resolve `FolType::Named` and `FolType::QualifiedNamed`.
- Generic parameters must resolve before outer type symbols when in scope.
- Qualified type paths should resolve through namespace/import roots before falling back to bare name diagnostics.

### 8.4 Qualified Value And Call Resolution

- Resolve `QualifiedIdentifier` and `QualifiedFunctionCall`.
- Resolution should work against:
- package roots
- namespace roots
- imported aliases
- top-level declarations that have fully qualified identities

Resolver should not attempt to semantically prove member access on:

- `FieldAccess`
- `MethodCall`
- `TemplateCall`
- `IndexAccess`

Those remain later semantic work.

### 8.5 Inquiry Target Resolution

- Resolve `InquiryTarget::Named`
- Resolve `InquiryTarget::Qualified`
- Validate `self` / `this` against enclosing resolver context

### 8.6 Import Resolution

Initial contract:

- `use` introduces a bound alias symbol in the enclosing scope.
- Import resolution only targets source inventory already supplied by `fol-stream`.
- The first fully supported semantic import kind should be `FolType::Location`.

Initial non-goal contract:

- `FolType::Url`
- `FolType::Standard`
- other parser-accepted but not yet semantically defined import source kinds

These should emit explicit “unsupported import source kind in resolver phase” diagnostics until intentionally implemented.

### 8.7 Comments

- `AstNode::Comment` and `AstNode::Commented` remain parser-visible but semantically inert here.
- Resolver must traverse through wrappers to reach real semantic nodes.
- Comments must not break scope construction or reference ordering.

## 9. Explicit Non-Goals For This Phase

- expression type inference
- method dispatch
- field lookup on records/entries
- overload selection
- ownership or borrowing validation
- return-type checking
- effect checking
- protocol/standard conformance
- dead-code analysis
- code generation

## 10. Test Strategy

Resolver needs the same culture as parser and lexer:

- many small targeted fixtures
- positive and negative tests for every feature
- exact diagnostic wording checks where the contract matters
- exact location checks for failure sites
- integration tests that prove the full pipeline still composes

### 10.1 New Test Layout

- `test/resolver/test_resolver.rs`
- `test/resolver/test_resolver_parts/*.rs`
- `test/resolver/*.fol`

`test/run_tests.rs` should gain:

- `mod resolver { include!("resolver/test_resolver.rs"); }`

### 10.2 Required Test Layers

- crate-local unit tests for IDs, scope tables, canonical keys, and import normalization
- resolver integration tests over real `.fol` fixtures
- end-to-end stream -> lexer -> parser -> resolver integration tests
- CLI/diagnostics tests for resolution failures once hooked up

### 10.3 Required Resolver Test Categories

- top-level declaration collection
- canonical duplicate detection
- forward references at top level
- local lexical scope ordering
- block shadowing
- parameter visibility
- capture visibility
- loop/each/for binder visibility
- rolling/comprehension binder visibility
- destructuring binding surfaces
- type-name resolution
- generic-parameter precedence over outer types
- qualified type path resolution
- qualified value path resolution
- qualified call resolution
- import alias binding
- `use loc` target resolution
- unsupported import-kind diagnostics
- unresolved identifier diagnostics
- unresolved qualified-path diagnostics
- ambiguous reference diagnostics
- duplicate symbol diagnostics
- shadowing diagnostics where contractually required
- inquiry target resolution
- invalid `self` / `this` diagnostics
- comment-wrapper transparency
- cross-file namespace/package resolution
- source-location precision in diagnostics

### 10.4 Testing Standard

For each new resolver behavior slice:

- add at least one success fixture
- add at least one failure fixture
- assert either exact resolution result shape or exact diagnostic content
- if the slice introduces a new diagnostic class, assert file/line/column too

Resolver should not be considered stable until it has a genuinely broad suite. The target should be “parser-like density”, not a handful of smoke tests.

## 11. Execution Slices

Each slice is intended to be buildable, testable, and commit-safe on its own. Every slice must land with its tests in the same commit.

### Phase 0: Foundation

#### Slice 0.1

Status: done

- Add workspace crate `fol-resolver`.
- Add empty public API, crate wiring, and test harness inclusion.
- Add a first resolver smoke test proving the crate is callable.

#### Slice 0.2

Status: done

- Introduce resolver error type(s) implementing `fol_types::Glitch`.
- Add a dedicated diagnostic-location conversion path for resolver failures.
- Add tests for basic resolver error formatting and diagnostics integration.

#### Slice 0.3

Status: done

- Add parser-adjacent syntax-origin support for successful AST nodes.
- Expose a resolver-consumable parsed wrapper with syntax index metadata.
- Add tests proving successful syntax nodes retain file/line/column origins.

#### Slice 0.4

Status: done

- Add resolver ID types and arena/basic table utilities.
- Add unit tests for ID stability and table behavior.

### Phase 1: Global Declaration Graph

#### Slice 1.1

Status: done

- Add `SourceUnitId` lowering based on `fol-stream::Source`.
- Regroup parsed top-level nodes by source identity using syntax origins.
- Add tests for cross-file regrouping and deterministic source-unit ordering.

#### Slice 1.2

Status: done

- Collect top-level named declarations into root/source-unit scopes.
- Cover:
- `var`
- `lab`
- `fun`
- `pro`
- `log`
- `typ`
- `ali`
- `def`
- `seg`
- `imp`
- `std`
- `use`
- Add tests for successful top-level collection across multiple files.

#### Slice 1.3

Status: done

- Implement canonical duplicate detection for top-level symbols.
- Add tests for raw duplicates and canonical duplicates across files.

#### Slice 1.4

- Allow forward references among top-level declarations in the same resolution domain.
- Add tests for top-level value and type references that point forward.

### Phase 2: Local Scopes

#### Slice 2.1

Status: done

- Add routine scopes and local declaration traversal.
- Resolve parameters and captures in routine bodies.
- Add tests for parameter/capture visibility and duplicates.

#### Slice 2.2

Status: done

- Add block scopes.
- Resolve local `var`, `lab`, and destructuring bindings with declaration-order visibility.
- Add tests for same-scope duplicates, shadowing, and use-before-bind failures.

#### Slice 2.3

Status: done

- Add loop iteration binder scopes.
- Cover `loop`, `for`, and `each`-style binders already lowered by parser.
- Add tests for binder visibility inside loop bodies and invisibility outside.

#### Slice 2.4

Status: done

- Add rolling/comprehension binder scopes.
- Add tests for rolling-expression binding visibility and shadowing.

#### Slice 2.5

Status: done

- Make comment nodes and `AstNode::Commented` wrappers transparent to traversal.
- Add tests proving comments do not disturb scope construction or reference binding.

### Phase 3: Reference Resolution

#### Slice 3.1

Status: done

- Resolve plain identifier expressions against lexical scopes plus top-level symbols.
- Add tests for locals, outer bindings, imports, and unresolved-name errors.

#### Slice 3.2

Status: done

- Resolve plain free calls (`FunctionCall { name, .. }`) against routine-capable symbols.
- Add tests for routine lookup, unresolved call names, and wrong-kind lookup diagnostics if needed.

#### Slice 3.3

- Resolve `FolType::Named` references.
- Add tests for type declarations, aliases, generic parameters, and unresolved types.

#### Slice 3.4

- Resolve `FolType::QualifiedNamed`.
- Add tests for qualified type names across namespaces and imports.

#### Slice 3.5

- Resolve `InquiryTarget::Named` and `InquiryTarget::Qualified`.
- Validate `self` / `this` resolver-context rules.
- Add tests for valid and invalid inquiry targets.

### Phase 4: Imports And Qualified Paths

#### Slice 4.1

Status: done

- Add import record lowering from `use` declarations.
- Bind import aliases into scope as first-class symbols.
- Add tests for alias visibility and duplicate imported names.

#### Slice 4.2

Status: done

- Implement semantic support for `use ... : loc = ...`.
- Match import targets against the source set already known to `fol-stream`.
- Add tests for:
- same-package imports
- nested-namespace imports
- missing local import target
- ambiguous local import target

#### Slice 4.3

Status: done

- Emit explicit unsupported diagnostics for `use` kinds not yet semantically implemented.
- Add tests for `url`, `std`, and any parser-accepted but resolver-unsupported import kind used in real fixtures.

#### Slice 4.4

- Resolve `QualifiedIdentifier` paths using package roots, namespace roots, and import aliases.
- Add tests for successful and failing qualified value references.

#### Slice 4.5

- Resolve `QualifiedFunctionCall` paths with the same root/path rules.
- Add tests for successful and failing qualified call references.

### Phase 5: Diagnostics Quality

#### Slice 5.1

- Harden unresolved-name diagnostics:
- exact name
- exact role
- exact location
- Add message and location tests.

#### Slice 5.2

- Harden duplicate and ambiguity diagnostics.
- Report both primary and secondary sites where useful.
- Add multi-file duplicate/ambiguity tests.

#### Slice 5.3

- Add shadowing diagnostics only where the chosen resolver contract requires them.
- Keep this explicit and test-backed instead of heuristic.

### Phase 6: CLI And Integration

#### Slice 6.1

Status: done

- Wire resolver into the root CLI after parser success.
- Make resolution errors visible through `fol-diagnostics`.
- Add end-to-end CLI-path tests for parse-clean but resolution-bad programs.

#### Slice 6.2

- Add integration tests for:
- stream -> lexer -> parser -> resolver happy path
- cross-file import resolution
- exact diagnostic location propagation through the full pipeline

### Phase 7: Hardening And Cleanup

#### Slice 7.1

Status: done

- Extract or centralize canonical identifier comparison helpers so parser and resolver do not drift.
- Add shared-behavior tests if this logic moves.

#### Slice 7.2

- Audit every parser AST form that can introduce or reference a name.
- Ensure no supported syntax surface is silently skipped by resolver traversal.
- Add fixture coverage for missed surfaces found in the audit.

#### Slice 7.3

- Sync docs:
- `PROGRESS.md`
- `FRONTEND_CONTRACT.md`
- `README.md`
- resolver crate docs/comments

## 12. Definition Of Done For `fol-resolver` Phase

Resolver phase is complete for this milestone only when all of the following are true:

- `fol-resolver` exists as a workspace crate and is used by the CLI
- successful AST nodes expose enough source-origin information for resolver diagnostics
- top-level declarations resolve across the whole loaded source set
- lexical locals resolve with explicit scope and declaration-order rules
- type names resolve
- plain and qualified value/call references resolve where in scope
- `use loc` works as a real semantic import form against the loaded source set
- unsupported import kinds fail explicitly
- unresolved, ambiguous, duplicate, and invalid-context errors are diagnostic-quality
- resolver tests are broad, fixture-heavy, and no longer just smoke tests

## 13. What Comes After This Plan

Only after resolver is stable should the next phase start:

- type checking / type inference
- member and method resolution that depends on type information
- protocol/standard conformance analysis
- ownership and borrowing analysis
- later backend/runtime work

That is later work. This plan is only the bridge from syntax to resolved names.
