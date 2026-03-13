# FOL Resolver Plan

Last rebuilt: 2026-03-13
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

Status: done

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

Status: done

- Resolve `FolType::Named` references.
- Add tests for type declarations, aliases, generic parameters, and unresolved types.

#### Slice 3.4

Status: done

- Resolve `FolType::QualifiedNamed`.
- Add tests for qualified type names across namespaces and imports.

#### Slice 3.5

Status: done

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

Status: done

- Resolve `QualifiedIdentifier` paths using package roots, namespace roots, and import aliases.
- Add tests for successful and failing qualified value references.

#### Slice 4.5

Status: done

- Resolve `QualifiedFunctionCall` paths with the same root/path rules.
- Add tests for successful and failing qualified call references.

### Phase 5: Diagnostics Quality

#### Slice 5.1

Status: done

- Harden unresolved-name diagnostics:
- exact name
- exact role
- exact location
- Add message and location tests.

#### Slice 5.2

Status: done

- Harden duplicate and ambiguity diagnostics.
- Report both primary and secondary sites where useful.
- Add multi-file duplicate/ambiguity tests.

#### Slice 5.3

Status: done

- Add shadowing diagnostics only where the chosen resolver contract requires them.
- Keep this explicit and test-backed instead of heuristic.

### Phase 6: CLI And Integration

#### Slice 6.1

Status: done

- Wire resolver into the root CLI after parser success.
- Make resolution errors visible through `fol-diagnostics`.
- Add end-to-end CLI-path tests for parse-clean but resolution-bad programs.

#### Slice 6.2

Status: done

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

Status: done

- Audit every parser AST form that can introduce or reference a name.
- Ensure no supported syntax surface is silently skipped by resolver traversal.
- Add fixture coverage for missed surfaces found in the audit.

#### Slice 7.3

Status: done

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

## 14. Continuation After Deep Scan

The milestone above closed the first resolver implementation pass, but a deep scan on 2026-03-13 found several semantic gaps that are too important to leave as "later polish".

The resolver should therefore not be treated as semantically complete yet.

This continuation keeps the completed history above intact and defines the remaining work required before `fol-resolver` can be considered genuinely done for the current language contract.

## 15. Verified Gaps Reopened By The Scan

### 15.1 Imported Names Are Not Exposed Through `use`

Current behavior:

- `use` lowers into an alias symbol plus an import record.
- Plain name lookup does not expose exported declarations from the imported target.
- In practice, `use math: loc = {math}` followed by `return answer` still fails.

Required behavior:

- `use` must expose exported declarations from the imported package or namespace.
- The alias symbol itself must still exist and remain usable for qualified access.
- Imported plain names must participate in resolution without requiring the caller to restate the namespace root in every use site.

Resolver contract to implement:

- local lexical bindings still win over imported names
- imported names may shadow nothing automatically; if multiple visible imports expose the same canonical name, resolution must become ambiguous
- `hid` declarations must never become import-visible
- non-exported declarations must not be exposed through imports
- imported exported routines, values, types, aliases, and other named top-level declarations should all follow the same visibility filter

### 15.2 Qualified Paths Through Import Aliases Are Not Reliable

Current behavior:

- qualified paths work through package roots
- qualified paths work through namespace roots
- qualified paths only work through import aliases when the alias spelling accidentally matches the target namespace root already handled by fallback logic

Required behavior:

- `use api: loc = {net::http}` must allow:
- `api::handler`
- `api::handler()`
- `api::Number`
- qualified inquiry targets rooted at `api`

This must work for:

- top-level `use`
- local `use` inside routines or blocks
- alias spellings that differ from the imported namespace root

Implementation constraint:

- import targets must be resolved before qualified-root lookup depends on them, or traversal must be split so qualified-path binding happens only after import targets are known

### 15.3 File-Private `hid` Visibility Is Structurally Modeled But Semantically Broken

Current behavior:

- parser and resolver record `hid` as file scope
- same-file routines do not actually see file-private declarations
- cross-file isolation therefore exists accidentally, but same-file visibility is broken

Required behavior:

- a `hid` declaration must be visible everywhere inside its own source file where the language allows that declaration kind to be referenced
- the same declaration must remain invisible from sibling files, even in the same package or namespace

Design rule:

- source-unit/file scope must participate in real lookup, not only in metadata
- fixing this must not accidentally leak file-private names to sibling files through package or namespace scopes

### 15.4 Built-In `str` Is Still Being Treated As A User-Defined Name

Current behavior:

- book-level code using `str` still resolves as an unresolved named type

Required behavior:

- `str` must behave like a real built-in type keyword, not as a resolver lookup against user symbols

Scope boundary:

- this continuation only requires the built-in `str` contract to be made correct
- do not silently broaden this into a larger type-system redesign
- any additional keyword-type gaps such as `num` should be treated separately once the language contract is clarified

### 15.5 Exact Origin Coverage Is Still Incomplete For Common Unresolved Names

Current behavior:

- qualified-path diagnostics usually keep exact locations
- unresolved plain identifiers, plain free calls, and plain named types can still surface with `location: null`

Required behavior:

- every resolver-produced unresolved or ambiguity diagnostic for supported reference forms must carry real file, line, column, and length data
- this includes at minimum:
- plain identifier expressions
- plain free calls
- plain named type references

Design rule:

- origin coverage must come from parser-visible syntax IDs or equivalent stable syntax handles
- resolver must not fabricate fake locations for these cases

## 16. Resolver Semantics To Finish Before Calling This Phase Done

### 16.1 Plain Lookup Must Gain Import Exposure Semantics

Resolution order should become:

- nearest lexical declarations in the current scope chain
- imported exported names visible from the current scope chain
- ambiguity if multiple equally visible imported targets expose the same canonical name and no direct lexical declaration wins

Notes:

- plain import exposure should not mutate parser AST
- imported-name lookup should be resolver-owned, explicit, and testable
- imported-name exposure should use canonical identifier comparison, not raw spelling only

### 16.2 Qualified Lookup Must Use Real Import Targets

Qualified-root resolution should support three root families cleanly:

- package root
- namespace root
- resolved import alias

Notes:

- alias-root success must not depend on alias spelling matching the namespace name
- local import aliases must work in the same way as top-level aliases
- qualified type, value, call, and inquiry resolution should share one coherent root-resolution policy

### 16.3 File Scope Must Be Part Of Semantic Lookup

Same-file name resolution should distinguish:

- package-visible declarations
- namespace-visible declarations
- file-visible declarations

Required guarantee:

- file-private names are visible from same-file routines and same-file top-level declarations where language rules allow lookup
- file-private names are not visible from any other source unit

### 16.4 Built-In `str` Must Exit The Named-Lookup Path

One of these must happen explicitly:

- parser lowers `str` into a dedicated built-in `FolType` variant
- or resolver treats parser-produced `str` as a built-in special case in a narrowly defined, test-backed way

Preferred direction:

- keep built-in type identity in parser/type representation, not in ad-hoc resolver string matching

### 16.5 Resolver Diagnostics Must Be Exact For Plain References

Required end state:

- unresolved `missing`
- unresolved `helper(...)`
- unresolved `MissingType`

must all produce non-null locations in CLI JSON and in resolver diagnostic structs.

## 17. Test Expansion Required By This Continuation

The current suite is broad, but the reopened gaps need a second wave of tests that are more semantic than structural.

### 17.1 Import Exposure Tests

Required new tests:

- success: imported exported value resolves as a plain identifier
- success: imported exported routine resolves as a plain free call
- success: imported exported type resolves as a plain named type
- failure: imported non-exported declaration stays unresolved
- failure: imported `hid` declaration stays unresolved
- ambiguity: two imports expose the same plain exported name
- precedence: local lexical binding beats imported plain name
- precedence: direct same-scope declaration beats imported plain name

### 17.2 Qualified Import Alias Tests

Required new tests:

- success: `use api: loc = {net::http}` plus `api::handler`
- success: `use api: loc = {net::http}` plus `api::handler()`
- success: `use api: loc = {net::http}` plus `api::Number`
- success: same cases for local `use` inside a routine
- failure: missing alias-root target still reports the exact qualified path location

### 17.3 File-Private Visibility Tests

Required new tests:

- success: same-file routine can resolve a `hid` value
- success: same-file routine can resolve a `hid` routine or type where relevant
- failure: sibling file in same package cannot resolve the `hid` symbol
- failure: nested namespace file cannot resolve the `hid` symbol from another file
- coverage: file-private forward references inside one file still follow the chosen top-level forward-reference contract

### 17.4 Built-In `str` Tests

Required new tests:

- parser-level or resolver-level positive test for `str` parameter types
- positive test for `str` local/field/type-alias surfaces that currently use named-type resolution
- negative control proving user-defined `str` declarations do not replace the built-in meaning unless the language explicitly intends that

### 17.5 Diagnostic Origin Tests

Required new tests:

- unresolved plain identifier returns exact file/line/column
- unresolved plain free call returns exact file/line/column
- unresolved plain named type returns exact file/line/column
- ambiguous imported plain name returns exact use-site location
- CLI JSON path keeps those locations without dropping them to `null`

## 18. Continuation Execution Slices

These slices are new pending work. They are part of the same resolver phase and should be implemented with the same discipline as before: buildable, test-backed, and commit-safe one slice at a time.

### Phase 8: Import Exposure Semantics

#### Slice 8.1

Status: done

- Add resolver-owned import-member exposure for plain lookup.
- Keep the alias symbol itself intact.
- Add tests for imported exported value resolution as a plain identifier.

#### Slice 8.2

Status: done

- Extend import-member exposure to plain free-call and plain named-type lookup.
- Add tests for imported exported routines and types resolving without qualification.

#### Slice 8.3

Status: done

- Filter import-visible members by declaration visibility.
- `exp` is import-visible.
- default/package-visible is not import-visible outside the imported boundary.
- `hid` is never import-visible.
- Add positive and negative tests for exported vs non-exported vs hidden imported declarations.

#### Slice 8.4

Status: done

- Add ambiguity and shadowing rules for imported plain names.
- Local lexical bindings and direct declarations should still win where intended.
- Add tests for duplicate exported names across multiple imports and for local shadowing of imported names.

### Phase 9: Qualified Import Alias Completion

#### Slice 9.1

Status: done

- Rework import-resolution ordering so qualified alias roots use resolved import targets instead of partially initialized records.
- Add tests where alias spelling differs from namespace root spelling.

#### Slice 9.2

Status: done

- Extend the alias-root fix across qualified identifiers, qualified calls, qualified type names, and qualified inquiry targets.
- Add tests for all supported qualified reference families.

#### Slice 9.3

Status: done

- Ensure local `use` declarations inside routines/blocks participate in the same qualified-root rules.
- Add tests for local import aliases with non-matching alias names.

### Phase 10: File-Private Visibility Semantics

#### Slice 10.1

Status: done

- Make file/source-unit scope part of semantic lookup for same-file references.
- Add tests proving same-file routines can resolve `hid` values.

#### Slice 10.2

Status: done

- Extend same-file `hid` visibility across other relevant named declaration kinds.
- Add tests for same-file hidden routines and types where parser/resolver surfaces exist.

#### Slice 10.3

Status: done

- Lock the negative side of the contract: sibling files and other namespaces must still fail to resolve `hid` names.
- Add multi-file package tests and cross-namespace negative tests.

### Phase 11: Built-In `str` Completion

#### Slice 11.1

Status: done

- Add explicit built-in handling for `str` in parser/type representation or a narrowly scoped resolver bridge if parser work is truly unnecessary.
- Prefer a representation-level fix over ad-hoc string checks.
- Add focused parser/resolver tests for `str`.

#### Slice 11.2

Status: done

- Extend `str` coverage across routine signatures, local declarations, and alias/type-definition surfaces that already consume `FolType`.
- Add end-to-end CLI tests proving `str` no longer hits unresolved-type diagnostics.

### Phase 12: Exact Origin Completion

#### Slice 12.1

Status: done

- Add syntax-origin coverage for plain identifier expressions.
- Add resolver tests proving unresolved plain names keep exact locations.

#### Slice 12.2

Status: done

- Add syntax-origin coverage for plain free-call names and plain named-type references.
- Add resolver tests proving unresolved plain calls and plain named types keep exact locations.

#### Slice 12.3

Status: done

- Add CLI JSON integration tests for the new non-null location guarantees.
- Ensure these checks cover unresolved and ambiguous plain-name scenarios.

### Phase 13: Docs And Re-Closeout

#### Slice 13.1

Status: done

- Sync `PROGRESS.md`, `README.md`, `FRONTEND_CONTRACT.md`, and resolver crate docs after the continuation work lands.
- Rewrite the resolver definition-of-done section so it reflects the finished semantic contract, not just the first milestone pass.

## 19. Resolver Phase Completion Record

The resolver phase is now complete for the current language contract at head.

Completed criteria:

- the original slices above are complete
- imported exported names are visible through `use` in plain lookup
- qualified alias-root resolution works regardless of alias spelling
- file-private `hid` names resolve inside the same file and nowhere else
- built-in `str` no longer enters unresolved named-type lookup
- plain unresolved identifier/call/type diagnostics keep exact non-null locations
- CLI JSON diagnostics keep exact non-null locations for unresolved and ambiguous plain-name cases
- the new tests in sections 17 and 18 are green

The next phase can now move on to post-resolution semantic work instead of further
resolver hardening.

## 20. Reopened Continuation: `package.yaml` + `build.fol`

The previous import continuation is no longer the target contract.

Current head proves the resolver can load:

- `loc` directory imports
- `std` directory imports
- `pkg` package-store imports

But it does so with the wrong package model for external packages:

- `pkg` still expects `package.fol`
- `package.fol` currently carries dependency declarations
- `build.fol` is explicitly excluded from resolver discovery

That is no longer the intended design.

This continuation reopens the import work with a stricter package split:

- `loc`: manifest-free local directory import
- `std`: manifest-free toolchain directory import
- `pkg`: formal external package import
- `package.yaml`: metadata only
- `build.fol`: dependency and export definition

## 21. Settled Contract For The Reset

### 21.1 `use` Still Imports Directories, Not Files

- `use` continues to import directory-backed package or namespace surfaces.
- A path ending in `foo.fol` remains invalid for `loc`, `std`, and `pkg`.
- Files are source units inside an imported root, not standalone modules.

### 21.2 `loc` Stays Manifest-Free

- `loc` points at a real local directory.
- No `package.yaml` is required.
- No `build.fol` is required.
- Resolver loads the exact supplied directory as the imported root.

### 21.3 `std` Stays Toolchain-Rooted

- `std` behaves like `loc`, but under the configured stdlib root.
- No `package.yaml` is required for the first stdlib version.
- No `build.fol` is required for the first stdlib version.

### 21.4 `pkg` Is The Formal External Package Kind

- `pkg` points at an installed external package root.
- That root must contain `package.yaml`.
- That root must contain `build.fol`.
- Missing either file is an explicit resolver error.

### 21.5 `package.yaml` Is Metadata Only

`package.yaml` is not a FOL source file and must not carry package graph semantics.

Allowed content should be limited to simple metadata, for example:

- `name`
- `version`
- `kind`
- optional descriptive metadata such as `description`, `license`, `authors`

Forbidden content:

- dependencies
- exports
- build logic
- anything that requires the FOL parser

### 21.6 `build.fol` Defines The Package Graph

`build.fol` is where a formal package defines:

- dependencies
- exported roots / namespaces
- package assembly rules needed by resolver

This file is where `def` belongs.

That means:

- ordinary source files use `use` to consume functionality
- `build.fol` uses `def` to define package dependencies and export surfaces

### 21.7 No Dual Manifest Authority

- `package.yaml` is the one manifest format
- `package.fol` is removed as a package-manifest concept
- a package root containing both must be rejected explicitly

## 22. Required Architectural Reset

This is not a small extension. It changes the meaning of `pkg`.

Required structural changes:

- remove FOL-AST-based package manifest parsing from resolver
- introduce a plain metadata loader for `package.yaml`
- introduce a dedicated `build.fol` package-definition loader
- stop reading dependency edges from metadata
- start reading dependency and export edges from `build.fol`
- keep excluding package-definition files from ordinary source-unit parsing

Important boundary:

- `package.yaml` should not go through `fol-parser`
- `build.fol` may reuse `fol-parser` syntax infrastructure if the DSL is a FOL subset
  or FOL-shaped surface, but it needs a package-definition validation layer on top

## 23. `package.yaml` Schema

The first schema should stay intentionally small.

Required fields:

- `name`
- `version`

Recommended early optional fields:

- `kind`
- `description`
- `license`

Validation rules:

- `name` must follow package-identifier rules
- `version` must be a non-empty string
- unknown top-level fields should fail explicitly in the first version
- malformed YAML should produce an explicit resolver input error

Non-goals for this schema:

- dependency declaration
- export declaration
- lockfile data
- fetch transport details

## 24. `build.fol` Contract

The exact DSL still needs to be settled in code, but the semantic contract is clear.

`build.fol` must be able to define at least:

- package dependencies
- exported roots / namespaces

And it must do so with `def`, not `use`.

So the first implementation should validate a narrow package-definition surface such as:

- `def dep ...: pkg = { ... }`
- `def export ...: loc = { ... }`

or an equivalent single-root package object if that proves easier to parse.

The implementation priority is not DSL richness. It is:

- unambiguous package definition
- explicit diagnostics
- resolver-friendly extraction of dependency and export records

## 25. Resolver Semantics After The Reset

### 25.1 `loc`

- unchanged in spirit
- still loads a local directory directly
- still rejects file targets
- still has no metadata/build requirements

### 25.2 `std`

- unchanged in spirit
- still loads from configured std roots
- still rejects file targets
- still has no metadata/build requirements for now

### 25.3 `pkg`

`pkg` changes materially:

- package identity comes from `package.yaml`
- dependency edges come from `build.fol`
- exported resolver-visible roots come from `build.fol`
- ordinary package source loading must exclude `package.yaml` and `build.fol`

This means a `pkg` import should no longer expose "all files under the root by default".
It should expose only what `build.fol` defines as public package surface.

## 26. Test Matrix For This Reset

### 26.1 `package.yaml` Tests

- success: minimal valid metadata parses
- success: optional metadata fields parse
- failure: missing `name`
- failure: missing `version`
- failure: invalid package name
- failure: malformed YAML
- failure: unknown field is rejected

### 26.2 `build.fol` Tests

- success: dependency definitions parse and validate
- success: export definitions parse and validate
- failure: `use` inside `build.fol` is rejected for package-definition semantics
- failure: malformed export target is rejected
- failure: malformed dependency target is rejected
- failure: unsupported build-surface node is rejected explicitly

### 26.3 `pkg` Loading Tests

- success: package root with `package.yaml` + `build.fol` loads
- failure: missing `package.yaml`
- failure: missing `build.fol`
- failure: both `package.yaml` and legacy `package.fol` present
- failure: `package.fol`-only root is rejected as legacy/unsupported

### 26.4 Export-Surface Tests

- success: only exported roots are visible to consumers
- success: exported namespace roots resolve through plain and qualified lookup
- failure: non-exported internal roots stay hidden from consumers
- success: transitive `pkg` dependencies load through `build.fol` dependency records

## 27. Execution Slices

### Phase 20: Contract Reset

#### Slice 20.1

Status: done

- Rewrite docs and plan to state that `package.yaml` is the only package manifest.
- Mark the old `package.fol`-based `pkg` continuation as superseded.

#### Slice 20.2

Status: pending

- Add explicit compatibility diagnostics for legacy `package.fol` package roots.
- Add tests locking the legacy rejection wording.

### Phase 21: `package.yaml` Loader

#### Slice 21.1

Status: pending

- Replace the current AST-based manifest parser with a YAML metadata loader.
- Introduce a resolver-owned `PackageMetadata` model.

#### Slice 21.2

Status: pending

- Validate required and optional metadata fields.
- Add targeted tests for missing, malformed, and unknown-field cases.

### Phase 22: `build.fol` Definition Surface

#### Slice 22.1

Status: pending

- Freeze the initial `build.fol` package-definition grammar.
- Keep the grammar intentionally narrow and resolver-driven.

#### Slice 22.2

Status: pending

- Implement dependency-definition extraction from `build.fol`.
- Ensure `def`, not `use`, is the accepted definition mechanism.

#### Slice 22.3

Status: pending

- Implement export-definition extraction from `build.fol`.
- Add negative tests for invalid export forms.

#### Slice 22.4

Status: pending

- Reject unsupported nodes inside `build.fol` with exact diagnostics.
- Add location-precise tests for those failures.

### Phase 23: `pkg` Loader Reset

#### Slice 23.1

Status: pending

- Change `pkg` root loading to require both `package.yaml` and `build.fol`.
- Reject missing-file combinations explicitly.

#### Slice 23.2

Status: pending

- Stop reading dependencies from package metadata.
- Read dependency edges from `build.fol` instead.

#### Slice 23.3

Status: pending

- Keep `package.yaml` and `build.fol` out of ordinary package source-unit parsing.
- Add tests locking the parsed-source exclusion behavior.

### Phase 24: Export Surface Enforcement

#### Slice 24.1

Status: pending

- Mount only `build.fol`-exported roots into consumer-visible `pkg` imports.
- Keep non-exported roots internal to the package.

#### Slice 24.2

Status: pending

- Resolve plain and qualified names through exported roots only.
- Add positive and negative export-visibility tests.

#### Slice 24.3

Status: pending

- Rewire transitive `pkg` dependency loading through `build.fol` dependency records.
- Keep cache and cycle behavior intact.

### Phase 25: Closeout

#### Slice 25.1

Status: pending

- Sync `README.md`, `PROGRESS.md`, `FRONTEND_CONTRACT.md`, and the book to the final
  `package.yaml` + `build.fol` contract.

#### Slice 25.2

Status: pending

- Rewrite the definition of done once the reset is implemented and fully tested.

## 28. Definition Of Done For This Reset

This reset is not complete at head.

It is complete only when all of the following are true:

- `loc` remains manifest-free
- `std` remains manifest-free for the current stdlib phase
- `pkg` requires `package.yaml`
- `pkg` requires `build.fol`
- `package.fol` no longer acts as a package manifest
- `package.yaml` is metadata-only
- `build.fol` defines dependency and export records with `def`
- consumer-visible `pkg` surfaces are restricted to exported roots
- transitive `pkg` dependencies load through `build.fol`
- the test matrix in section 26 is green

Current state at head:

- the previous `package.fol`-based continuation is implemented
- that implementation is now considered superseded for `pkg`
- this reset must be completed before the package-system contract is considered settled
