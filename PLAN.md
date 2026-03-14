# FOL V1 Typecheck Plan

Last rebuilt: 2026-03-14
Scope: `fol-typecheck` plus the minimal resolver/CLI/doc refactors required to insert whole-program type checking after `fol-resolver` for the `V1` language subset only

## 0. Purpose

- The current front-end chain is in strong shape:
  - `fol-stream`
  - `fol-lexer`
  - `fol-parser`
  - `fol-package`
  - `fol-resolver`
- The next compiler question is no longer "what does this name refer to?"
- The next compiler question is "is this resolved program type-correct, and what is the type of every meaningful expression and declaration surface?"
- This plan defines the first real whole-program typechecking milestone for
  `V1`, not for the entire long-term book surface.
- The version boundary for the language now lives in [`VERSIONS.md`](./VERSIONS.md).
- That means this plan is intentionally narrower than the whole book:
  - it must make the `V1` core language real
  - it must reject `V2` and `V3` features explicitly
  - it must not quietly expand into later-version semantics

## 1. What Was Scanned For This Plan

This plan is based on the current code plus the relevant book chapters that define the type surface the compiler claims to support.

### 1.1 Code Surfaces Checked

- workspace structure in `Cargo.toml`
- current parser AST in `fol-parser/src/ast/mod.rs`
- current prepared-package boundary in `fol-package`
- current resolver output and session/loading boundary in `fol-resolver`
- shared utility state in `fol-types`

### 1.2 Book Surfaces Checked

- `book/src/follang.md`
- `book/src/400_type/*`
- `book/src/500_items/100_variables.md`
- `book/src/500_items/200_routines/*`
- `book/src/500_items/300_constructs/*`
- `book/src/500_items/400_standards.md`
- `book/src/500_items/500_generics.md`
- `book/src/200_expressions/200_sta/100_control.md`
- `book/src/650_errors/*`
- `book/src/750_conversion/*`
- `book/src/800_memory/*`
- `book/src/900_processor/*`
- [`VERSIONS.md`](./VERSIONS.md)

### 1.3 Important Scan Outcome

- The book names a wide type surface, but some semantic chapters are still only partial drafts.
- Most notably:
  - coercion and casting chapters are placeholders
  - many advanced surfaces exist syntactically but have no current semantic contract in code
  - ownership/borrowing and standard/contract enforcement are clearly later phases
- Therefore the typechecker plan must be explicit about:
  - what `V1` will fully support
  - what `V2` and `V3` surfaces it will reject with exact unsupported diagnostics
  - what later compiler work still belongs to `V1` after typechecking

## 2. Main Decision

We add a new crate named `fol-typecheck`.

Pipeline order becomes:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck`

Crate dependency direction becomes:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck`

That means:

- `fol-typecheck` consumes resolver output, not raw parser output
- package discovery/import loading remains package work
- name binding and import visibility remain resolver work
- type rules, type inference, coercion policy, and typed diagnostics for the
  `V1` language subset become typechecker work

## 3. What `fol-typecheck` Must Own For V1

`fol-typecheck` is responsible for:

- semantic type representation beyond raw `FolType` syntax
- checking declared types on:
  - bindings
  - parameters
  - return types
  - routine error types
  - aliases
  - record fields
  - entry variants
- typing expressions and statements
- checking assignments, calls, returns, and reports
- typing field/index/access surfaces
- enforcing operator typing rules
- defining the initial coercion vs cast contract
- local inference where source omits a type
- exact type diagnostics with precise source origins

`fol-typecheck` is not responsible for:

- package acquisition
- ordinary name lookup or import target resolution
- ownership or borrowing analysis
- standard/contract conformance in the full book sense
- runtime lowering
- linking
- code generation

## 4. What Must Stay Out Of This V1 Plan

Anything assigned to `V2` or `V3` in [`VERSIONS.md`](./VERSIONS.md) is outside
this plan unless it is needed only for an explicit unsupported diagnostic.

These remain elsewhere:

- `fol-package`
  - package metadata
  - `build.fol` extraction
  - package roots
  - dependency preparation
  - native artifact declarations
- `fol-resolver`
  - scopes
  - symbol identity
  - import aliasing
  - package/namespace/file visibility
- later semantic passes
  - standards / protocol / blueprint / extension fulfillment
  - generics beyond structural preservation
  - ownership/borrowing
  - effect/purity enforcement
- later backend phases
  - calling convention lowering
  - linking
  - runtime layout/codegen

## 5. C ABI Boundary

Full C ABI compatibility is a `V3` feature in [`VERSIONS.md`](./VERSIONS.md).

It does not belong in this `V1` typecheck plan.

The only active requirement here is:

- `V1` must fail explicitly if a source surface requires foreign/ABI semantics
- the typechecker data model should avoid obviously blocking a later foreign type
  family

## 6. V1 Typechecking Contract

The first milestone must be strict and honest.

### 6.1 Fully Supported In V1

- builtin scalar types:
  - `int`
  - `flt`
  - `bol`
  - `chr`
  - builtin `str`
- named and qualified named type references
- aliases
- routine parameters
- routine returns
- routine error types
- plain/local binding declarations
- plain identifier and call typing over resolved symbols
- assignments
- `return`
- `report`
- record field typing
- entry member typing
- simple container typing:
  - arrays
  - vectors
  - sequences
  - sets
  - maps
- basic optional / error structural checking
- exact diagnostics

### 6.2 Explicitly Unsupported But Diagnosed In V1

These must not silently pass unchecked:

- `V2` surfaces:
  - generics beyond basic declaration-surface preservation
  - standards / contracts / protocol / blueprint / extension conformance
  - advanced `any` / `multiple` / `union` semantics
  - advanced meta-level language semantics
- `V3` surfaces:
  - ownership / borrowing semantics
  - pointer semantics
  - eventual / coroutine / channel / mutex semantics
  - native / C ABI semantics
- additional non-`V1` surfaces:
  - matrix semantics beyond structural parsing
  - reactive / static / range-constrained semantic enforcement
- build-script semantics for `build.fol`

If encountered, they should produce explicit `TypecheckUnsupported` errors with exact locations.

## 7. Initial Semantic Policy Decisions

The book’s conversion chapter is still too sparse to use as an implementation contract by itself, so the compiler needs an explicit initial policy.

### 7.1 Coercion Policy For V1

V1 coercion should be intentionally narrow:

- exact type match is always allowed
- `never` can flow into any expected type
- integer literals may be admitted to concrete integer declarations if they fit
- float literals may be admitted to concrete float declarations if they fit the declared family
- no implicit int-to-float coercion
- no implicit float-to-int coercion
- no implicit signed-width changes
- no implicit container/string structural conversions

### 7.2 Cast Policy For V1

- explicit cast surfaces should remain separate from coercions
- if the parser already preserves cast syntax, typechecker should own legality checks
- unsupported cast families should produce exact diagnostics instead of silently degenerating

### 7.3 Error/Return Policy For V1

- `return expr` must match the routine return type
- `report expr` must match the routine error type
- `report` in routines without an error type is invalid
- `panic` should type as `never`

## 8. Proposed Crate Shape

Workspace addition:

- `fol-typecheck`

Expected crate surface:

- `fol-typecheck/src/lib.rs`
- `fol-typecheck/src/errors.rs`
- `fol-typecheck/src/config.rs`
- `fol-typecheck/src/types.rs`
- `fol-typecheck/src/builtins.rs`
- `fol-typecheck/src/model.rs`
- `fol-typecheck/src/session.rs`
- `fol-typecheck/src/decls.rs`
- `fol-typecheck/src/exprs.rs`
- `fol-typecheck/src/containers.rs`
- `fol-typecheck/src/control.rs`
- `fol-typecheck/src/operators.rs`
- `fol-typecheck/src/conversions.rs`

The file split may change, but these responsibilities must exist.

## 9. Core Data Model

### 9.1 Semantic Type Representation

Raw `FolType` is syntax.

`fol-typecheck` needs a normalized semantic type model, likely something like:

- `CheckedTypeId`
- `CheckedType`
- `TypeTable`

This avoids comparing raw AST syntax directly everywhere.

### 9.2 Typed Program Output

Resolver output is not enough anymore.

We need a typed handoff, likely something like:

- `TypedProgram`
- `TypedSourceUnit`
- `TypedScopeFacts`
- `TypedSymbol`
- `TypedReference`
- `TypedNode`

Important rule:

- typed output should reference resolver identities where possible
- typechecker must not rebuild name resolution from scratch

### 9.3 Typed Node Coverage

Not every AST node needs a full typed wrapper immediately, but the following do:

- literals
- identifiers
- calls
- field access
- index access
- assignments
- returns
- reports
- container literals
- record construction
- branch expressions/final expressions where type agreement matters

### 9.4 Symbol Type Facts

Typechecker should attach or derive semantic type facts for:

- value bindings
- parameters
- routines
- type declarations
- aliases
- record fields
- imported callable/value/type symbols once resolver has exposed them

## 10. Diagnostics

Type diagnostics must have the same quality bar as parser/resolver diagnostics:

- exact file
- exact line
- exact column
- useful label text
- role-specific messages

Required diagnostic families:

- incompatible assignment
- incompatible initializer
- unresolved semantic precondition/internal mismatch
- call target is not callable
- wrong call arity
- wrong argument type
- wrong return type
- wrong report type
- field not present on typed receiver
- indexing non-indexable value
- branch type mismatch
- unsupported semantic surface

## 11. Test Strategy

The testing bar should match parser/resolver style:

- lots of focused Rust tests
- small fixtures
- exact diagnostic coverage
- CLI JSON coverage for end-to-end type failures

### 11.1 New Test Areas

- `test/typecheck/`
- `test/typecheck/test_typecheck.rs`
- `test/typecheck/test_typecheck_parts/*`

### 11.2 Required Coverage Themes

- builtin scalar typing
- explicit declaration typing
- local inference
- alias and named type resolution
- call typing and arity
- assignment compatibility
- return/report compatibility
- container literal typing
- record/entry typing
- import-exposed symbol typing
- unsupported surface diagnostics
- exact location retention through CLI JSON

### 11.3 Slice Discipline

Each feature/fix slice must land with its test in the same commit.

## 12. Implementation Phases

### Phase 0: Foundation

Status: pending

#### 0.1

Status: done

- Add `fol-typecheck` to the workspace with a small public API and smoke tests.

#### 0.2

Status: done

- Add `TypecheckError` kinds and exact diagnostic-location plumbing.

#### 0.3

Status: done

- Add semantic type interning/canonical builtin type tables.

#### 0.4

Status: done

- Add smoke coverage from `ResolvedProgram` into a no-op `TypedProgram` shell.

### Phase 1: Semantic Type Model

Status: pending

#### 1.1

Status: done

- Define normalized semantic types for the currently implemented builtin and declared type surface.

#### 1.2

Status: done

- Add `TypedProgram`, `TypedSourceUnit`, `TypedSymbol`, and `TypedNode` result models.

#### 1.3

Status: done

- Lower resolved declaration signatures into semantic type facts without checking bodies yet.

#### 1.4

Status: done

- Lock tests for builtin `str`, named types, aliases, and qualified named-type lowering.

### Phase 2: Declaration Signatures

Status: pending

#### 2.1

Status: done

- Check declared types on bindings and destructuring surfaces.

#### 2.2

Status: done

- Check routine parameter, return, and error-type declarations.

#### 2.3

Status: done

- Check alias declarations and record/entry member type declarations.

#### 2.4

Status: done

- Lock forward and cross-file declared-type extraction tests through resolver output.

### Phase 3: Core Expression Typing

Status: pending

#### 3.1

Status: done

- Type literals and resolved plain/qualified identifiers.

#### 3.2

Status: done

- Type block/final-expression bodies and local initializer surfaces.

#### 3.3

Status: done

- Type assignments and assignment-target compatibility.

#### 3.4

Status: done

- Type free calls and method calls, including arity checks.

#### 3.5

Status: done

- Type field access, index access, slice/access basics, and non-callable/non-indexable errors.

### Phase 4: Routine And Control Semantics

Status: pending

#### 4.1

Status: done

- Enforce `return` compatibility with declared routine return types.

#### 4.2

Status: done

- Enforce `report` compatibility with declared routine error types.

#### 4.3

Status: done

- Type `if`/`when` branch agreement where result typing matters.

#### 4.4

Status: done

- Type loop/control-flow basics and reserve `break`/`yeild` behavior explicitly.

#### 4.5

Status: done

- Introduce `never`-aware control typing for `panic`, early exits, and unreachable tails.

### Phase 5: Container And Aggregate Types

Status: pending

#### 5.1

Status: done

- Type array/vector/sequence literals and element agreement.

#### 5.2

Status: done

- Type set/map literals and container access compatibility.

#### 5.3

Status: done

- Type record construction and field initializer compatibility.

#### 5.4

Status: done

- Type entry/enum-like value surfaces.

#### 5.5

Status: done

- Add structural typing checks for optional/error shells used by current surfaces and make pointer surfaces fail explicitly as `V3` semantics.

### Phase 6: Operators And Conversions

Status: pending

#### 6.1

Status: done

- Freeze unary/binary operator typing matrices for the supported scalar families.

#### 6.2

Status: done

- Implement the narrow v1 coercion policy and use it in assignment/call/return/report checking.

#### 6.3

Status: done

- Implement explicit cast legality checks or reject unsupported cast families with exact diagnostics.

#### 6.4

Status: done

- Lock literal-fit behavior for integer/float declarations and argument passing.

#### 6.5

Status: done

- Sync docs to the real coercion vs cast contract once frozen in code.

### Phase 7: Explicit V2 And V3 Boundaries

Status: pending

#### 7.1

Status: done

- Emit explicit unsupported diagnostics for generic semantic surfaces that `V1` does not actually enforce yet.

#### 7.2

Status: done

- Emit explicit unsupported diagnostics for `V2` contract/conformance surfaces such as standards, protocols, blueprints, and extensions.

#### 7.3

Status: done

- Emit explicit unsupported diagnostics for `V3` systems surfaces such as ownership, borrowing, pointers, eventuals, coroutines, channels, and C ABI, plus remaining non-`V1` reactive/static/range semantics.

#### 7.4

Status: done

- Make the boundary explicit that `build.fol` package semantics are not part of ordinary typechecking yet.

### Phase 8: CLI And End-To-End Integration

Status: pending

#### 8.1

Status: done

- Wire the root CLI to run `fol-typecheck` after resolver.

#### 8.2

Status: pending

- Add end-to-end integration tests for successful and failing typecheck runs, including JSON diagnostics.

#### 8.3

Status: pending

- Update `README.md`, `PROGRESS.md`, and `FRONTEND_CONTRACT.md` to describe the new stage boundary.

#### 8.4

Status: pending

- Rewrite this file into a completion record once `fol-typecheck` is integrated.

## 13. Definition Of Done

This plan is complete when all of the following are true:

- `fol-typecheck` exists as a workspace crate
- it consumes `ResolvedProgram`
- it produces a typed semantic result for the `V1` language subset
- declaration signatures are checked
- core expression/call/assignment/return/report surfaces are checked
- the initial coercion/cast contract is explicit and test-backed
- `V2` and `V3` surfaces fail explicitly instead of silently passing unchecked
- the CLI runs typechecking after resolution
- exact type diagnostics survive to CLI JSON
- docs describe `fol-typecheck` as the next `V1` semantic stage in the compiler chain

## 14. Next Boundary After This Plan

If this plan finishes cleanly, the next compiler work should still stay inside
`V1` until the compiler can carry the `V1` subset toward a binary-producing
pipeline.

That means the immediate follow-up should be the remaining `V1` compiler stages,
not a jump to `V2` features.

Only after `V1` has been carried through later compiler stages should the
project return to:

- `V2` language semantics:
  - standards / contracts conformance
  - generics
  - richer advanced type semantics
- `V3` systems and interop:
  - ownership / borrowing analysis
  - future foreign / C ABI integration
- and in parallel with the `V1` compiler path itself:
  - lowering / codegen / backend work
