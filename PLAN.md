# FOL Lowering Plan

Last updated: 2026-03-15

This plan replaces the old completed typecheck milestone record.

The next compiler stage is `fol-lower`.
Its job is to take a valid typed `V1` FOL workspace and lower it into a smaller,
backend-oriented intermediate representation that is still independent of LLVM,
C code generation, linking, or runtime policy.

## 0. Plan Basis

This plan is based on a targeted rescan of the current repository and book state:

- workspace manifest in `Cargo.toml`
- current public typecheck API in `fol-typecheck/src/lib.rs`
- current typed model in `fol-typecheck/src/model.rs`
- current checked type model in `fol-typecheck/src/types.rs`
- current typecheck diagnostics surface in `fol-typecheck/src/errors.rs`
- current resolver workspace and mounted-symbol model in `fol-resolver/src/model.rs`
- current project status in `README.md`
- current implementation ledger in `PROGRESS.md`
- current version boundary document in `VERSIONS.md`
- relevant `V1` book surfaces for declarations, routines, structured data,
  control flow, modules, errors, and conversion

## 1. Why `fol-lower` Is Next

The current chain is:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck`

That is enough to prove that a `V1` program is syntactically valid, package-aware,
name-resolved, and type-correct.

It is not enough to produce a binary.

The compiler still lacks:

- a compact backend-facing IR
- explicit lowered control flow
- explicit lowered data construction
- explicit package/export/entry metadata for a backend
- a stable handoff point for later code generation

So the next chain should become:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck -> fol-lower -> later backend`

## 2. Scope Of This Plan

`fol-lower` is a `V1` milestone only.

It is responsible for:

- consuming `TypedWorkspace`
- lowering currently supported `V1` semantics into a backend-oriented IR
- preserving exact package identity, symbol identity, and source origins
- normalizing high-level typed syntax into explicit control-flow and data ops
- producing a stable handoff for a later backend crate

It is not responsible for:

- LLVM integration
- C code generation
- native machine code generation
- linking
- runtime packaging
- ownership or borrowing
- standards, blueprints, or generics
- C ABI import/export
- `V2` or `V3` features from `VERSIONS.md`

## 3. Boundary Decisions

The plan assumes these design decisions up front.

### 3.1 Input Boundary

Input to `fol-lower` is `TypedWorkspace`, not parser output and not resolver
output.

That means lowering may assume:

- package discovery is already complete
- import mounting is already complete
- all references are already resolved
- `V1` type errors are already rejected
- lowering never redoes name resolution or type checking

### 3.2 Output Boundary

Output should be a backend-neutral lowered IR.

It should be rich enough for a later backend, but simpler than the typed AST.

The first IR should be:

- explicit
- deterministic
- non-SSA
- package-aware
- source-map aware
- simple enough to verify

SSA can come later if a future LLVM backend needs it.

### 3.3 Workspace Boundary

Lowering must operate on the whole typed workspace, not only the entry package.

This matters because:

- `loc`, `std`, and `pkg` imports are already real package graph edges
- backends will need one coherent lowered workspace
- imported routines, types, globals, and exports must lower exactly once

### 3.4 No Backend Leakage

`fol-lower` must not hardcode LLVM concepts, C syntax, linker flags, or ABI
rules into the IR.

The IR should describe program behavior, not one backend format.

## 4. What Must Be Lowered In `V1`

The lowering surface should match the current `V1` typecheck boundary.

That includes:

- package-local and imported globals
- functions and procedures
- alias-backed type references
- records
- entries
- builtin scalars
- arrays
- vectors
- sequences
- sets
- maps
- optional shells
- error shells
- `nil`
- `unwrap`
- variable initialization
- assignment
- plain and qualified calls
- method calls after receiver resolution
- field access
- indexing and container access that are already part of typed `V1`
- `return`
- `report`
- `when`
- loops and loop exits that are already supported in typed `V1`

It must also preserve enough metadata for later backend entrypoint and export
selection.

## 5. What Stays Out

The following remain explicitly out of scope for this plan:

- blueprints
- standards
- generics
- metaprogramming execution
- ownership and borrowing
- pointer semantics as a full memory model
- concurrency or processor chapter semantics
- C ABI headers, objects, libraries, or foreign symbol modeling
- backend-specific calling conventions
- optimizer passes

If any of those surfaces are already parsed and happen to survive into typed
models, `fol-lower` should reject them explicitly with lowering-specific
diagnostics instead of silently guessing.

## 6. Proposed Crate Shape

Add a new workspace crate:

- `fol-lower`

Suggested initial modules:

- `src/lib.rs`
- `src/errors.rs`
- `src/model.rs`
- `src/ids.rs`
- `src/session.rs`
- `src/types.rs`
- `src/decls.rs`
- `src/exprs.rs`
- `src/control.rs`
- `src/verify.rs`

Possible later modules:

- `src/constants.rs`
- `src/source_map.rs`
- `src/debug.rs`

## 7. Proposed IR Direction

The first lowered IR should be intentionally plain.

### 7.1 Workspace And Package Layer

Proposed top-level handoff:

- `LoweredWorkspace`
- `LoweredPackage`
- `LoweredTypeTable`
- `LoweredSourceMap`

`LoweredWorkspace` should contain:

- entry package identity
- all lowered packages in deterministic order
- a workspace-wide lowered type arena or explicit per-package translated type tables
- exported symbol metadata
- entrypoint candidates
- a source map for diagnostics and backend debug output

`LoweredPackage` should contain:

- package identity
- lowered globals
- lowered routines
- lowered declared runtime types
- export metadata
- package-local symbol translation tables

### 7.2 Type Layer

The lowered IR should not keep raw `CheckedTypeId` values as the only runtime
type story.

It needs a lowering-owned type layer that can describe:

- builtin scalars
- record layouts
- entry layouts
- container runtime shapes
- optional/error shell runtime shapes
- routine signatures

Aliases should generally erase to their underlying lowered type, while debug and
source metadata may preserve alias names separately.

### 7.3 Control Layer

The IR should use explicit basic blocks and terminators.

Proposed core shape:

- `LoweredRoutine`
- `LoweredBlock`
- `LoweredInstr`
- `LoweredTerminator`

Important decision:

- first IR should be block-based and non-SSA
- explicit temporaries and locals are acceptable
- branch join values should use explicit temporary slots, not SSA phi nodes

### 7.4 Operation Layer

Likely instruction families:

- constant literal creation
- copy/move between locals or temporaries
- load global
- store global
- load local
- store local
- construct record
- construct entry
- construct container
- extract field
- extract entry payload
- index container
- call routine
- call procedure
- explicit shell wrap
- explicit shell unwrap
- explicit cast/coercion op where `V1` typecheck already decided it is valid

Likely terminators:

- jump
- branch
- return
- report
- unreachable

## 8. Lowering Invariants

The lowered IR should satisfy these invariants:

- every value-producing node has exactly one lowered result slot or explicit
  terminal behavior
- every control-flow edge is explicit
- every lowered routine body has a valid entry block
- every block ends in exactly one terminator
- every symbol reference points to an owning lowered definition, not a mounted
  duplicate shell
- imported package members lower exactly once in their owning package
- all source-map references remain stable enough for later diagnostics
- all backend-visible types come from lowering-owned translated type data
- aliases do not survive as runtime-distinct shapes unless later backend work
  explicitly needs that

## 9. Key Semantic Transformations

Lowering should deliberately erase front-end-only structure where it is no
longer needed.

### 9.1 Imports And Qualification

By the time lowering runs:

- imports are no longer syntax to preserve
- qualified names are no longer needed as source-level lookup structures
- mounted imported symbols should lower to their owning package definitions

So lowering should erase:

- `use` syntax
- import alias routing
- qualified-path syntax

And preserve:

- package identity
- actual owning symbol identity
- export visibility metadata needed later

### 9.2 Methods

Method calls should lower to direct routine calls with an explicit receiver
argument.

The lowered IR should not keep a separate “method call” semantic form unless a
later backend absolutely needs one.

### 9.3 `when`

`when` should lower to explicit block control flow.

If `when` is value-producing, lowering should:

- create a destination temporary
- lower each branch to assign into that destination
- join at an explicit continuation block

### 9.4 Loops

Loop lowering should produce explicit blocks for:

- header / condition
- body
- exit

Any `break`-style exit should become an explicit jump to the exit block, with
value transport only if typed `V1` semantics require it.

### 9.5 Records And Entries

Records and entries should stop looking like parser syntax and start looking like
runtime data operations.

That means:

- record initialization lowers to explicit field construction order
- field access lowers to explicit extraction ops
- entry construction lowers to explicit tag plus optional payload construction
- any variant/payload inspection that already exists in typed `V1` lowers to
  explicit IR ops

### 9.6 Containers

Container literals should lower to explicit container construction instructions.

The first lowering stage does not need to choose the final runtime memory layout.
It only needs a backend-neutral IR node that says:

- array literal with element list and optional static size
- vector literal
- sequence literal
- set literal
- map literal

### 9.7 Shells

Optional and error shells are part of current `V1`.

So lowering must make them explicit instead of leaving them as typed AST
conventions.

That includes:

- `nil`
- shell wrapping
- shell-aware coercions already accepted by typecheck
- `unwrap`
- `report`

The backend can decide representation later.
Lowering must decide the semantic operation boundary now.

## 10. Source Maps And Diagnostics

Lowering must preserve source precision.

Each important lowered entity should be traceable back to source:

- globals
- routines
- parameters
- blocks where useful
- instructions where useful
- terminators

This is needed for:

- lowering diagnostics
- later backend diagnostics
- debug dumps
- future optimization diagnostics

Lowering-specific errors should not use `InvalidInput` or `Internal` for normal
unsupported user code.

They should have dedicated lowering error kinds such as:

- unsupported
- invalid lowered contract
- impossible typed input
- internal verifier failure

## 11. Testing Strategy

`fol-lower` needs the same style of heavy test coverage that parser, resolver,
and typecheck already have.

Required coverage categories:

- unit tests for IR data structures
- unit tests for lowering-owned type translation
- fixture tests for declaration lowering
- fixture tests for expression lowering
- fixture tests for control-flow lowering
- workspace tests across `loc`, `std`, and `pkg`
- end-to-end CLI tests for successful lowering once the CLI is wired to it
- negative tests for explicit unsupported diagnostics
- source-map tests for exact lowering error locations
- import/mounted-symbol parity tests so imported code lowers exactly like local code

Important rule:

- every lowering feature slice should land with its relevant tests in the same commit

## 12. Definition Of Done

This plan is complete only when all of the following are true:

- `fol-lower` exists as a workspace crate
- it consumes `TypedWorkspace`
- it lowers the full current `V1` typed surface
- it handles imported `loc`, `std`, and `pkg` packages through the workspace graph
- it emits explicit IR for control flow and data construction
- it preserves exact source origins for diagnostics
- it rejects out-of-scope surfaces explicitly
- the CLI can produce and validate lowered workspaces
- docs are updated to describe lowering as the next completed stage after typecheck

## 13. Implementation Slices

### Phase 0. Crate Foundation

- `0.1` `done` Add `fol-lower` to the workspace and root crate dependencies.
- `0.2` `done` Add `fol-lower` public API shell with `LoweringResult`, `Lowerer`, and error surface.
- `0.3` `done` Add lowering error kinds and diagnostics integration.
- `0.4` `done` Add smoke tests proving a typed workspace can be handed to the lowerer.

### Phase 1. IR Model Foundation

- `1.1` `done` Define lowering-owned ID types for packages, globals, routines, blocks, locals, instructions, and lowered types.
- `1.2` `done` Define `LoweredWorkspace`, `LoweredPackage`, and source-map shells.
- `1.3` `done` Define `LoweredTypeTable` and lowered runtime type shapes.
- `1.4` `done` Define `LoweredRoutine`, `LoweredBlock`, `LoweredInstr`, and `LoweredTerminator`.
- `1.5` `done` Add unit tests for deterministic ID allocation and core IR invariants.

### Phase 2. Typed Workspace Translation

- `2.1` `done` Create lowering session state over `TypedWorkspace`.
- `2.2` `done` Translate package identities, source units, and symbol ownership into lowering tables.
- `2.3` `done` Translate typecheck package-local type IDs into lowering-owned type IDs.
- `2.4` `done` Preserve mounted imported symbol provenance so imported definitions lower in their owning package.
- `2.5` `done` Preserve syntax origins and build lowering source maps across the whole workspace.
- `2.6` `done` Keep single-program lowering as an explicit compatibility shim over workspace lowering.

### Phase 3. Declaration And Type Lowering

- `3.1` `pending` Lower builtin scalar types and routine signature types into lowering-owned type data.
- `3.2` `pending` Lower aliases by erasing them to underlying runtime shapes while preserving debug metadata.
- `3.3` `pending` Lower record type declarations into explicit field layouts.
- `3.4` `pending` Lower entry type declarations into explicit tag/payload layouts.
- `3.5` `pending` Lower globals and top-level bindings into lowered storage declarations.
- `3.6` `pending` Lower routine and procedure declarations into lowered routine shells with parameters and result contracts.
- `3.7` `pending` Add declaration-level tests across local and imported packages.

### Phase 4. Core Expression Lowering

- `4.1` `pending` Lower literals into constant or constructor instructions.
- `4.2` `pending` Lower resolved identifiers into explicit local/global loads.
- `4.3` `pending` Lower initializer and body expressions into explicit destination slots.
- `4.4` `pending` Lower assignments into explicit store instructions.
- `4.5` `pending` Lower plain and qualified calls into direct callee calls.
- `4.6` `pending` Lower method calls into routine calls with explicit receiver arguments.
- `4.7` `pending` Lower postfix field access into explicit extraction instructions.
- `4.8` `pending` Lower indexing/subscript forms that are already part of typed `V1`.
- `4.9` `pending` Add expression-lowering parity tests across local and imported packages.

### Phase 5. Control-Flow Lowering

- `5.1` `pending` Lower `return` into explicit return terminators.
- `5.2` `pending` Lower `report` into explicit report terminators.
- `5.3` `pending` Lower `when` as statement-style control flow.
- `5.4` `pending` Lower value-producing `when` into branch blocks plus explicit join temporary.
- `5.5` `pending` Lower loop forms into explicit header/body/exit blocks.
- `5.6` `pending` Lower loop exits such as `break` into explicit control-flow edges.
- `5.7` `pending` Add control-flow graph verifier checks for block termination and reachability basics.
- `5.8` `pending` Add control-flow fixture tests with exact lowered block-shape assertions.

### Phase 6. Aggregate, Container, And Shell Lowering

- `6.1` `pending` Lower record initialization into explicit constructor instructions.
- `6.2` `pending` Lower entry construction into explicit tag/payload instructions.
- `6.3` `pending` Lower array, vector, and sequence literals.
- `6.4` `pending` Lower set and map literals.
- `6.5` `pending` Lower `nil` into explicit optional/error shell constructors.
- `6.6` `pending` Lower `unwrap` into explicit shell-unwrapping instructions with preserved source origins.
- `6.7` `pending` Lower shell apparent-type overrides into concrete lowered runtime operations.
- `6.8` `pending` Lower `V1` implicit coercions and shell wraps into explicit conversion instructions.
- `6.9` `pending` Add aggregate/container/shell tests across both local and imported declarations.

### Phase 7. Workspace Exports And Entrypoints

- `7.1` `pending` Lower package export metadata from prepared packages into backend-facing lowered export data.
- `7.2` `pending` Mark candidate entry routines for the later binary-producing stage.
- `7.3` `pending` Ensure imported packages lower exactly once even when mounted in multiple places.
- `7.4` `pending` Add workspace tests proving `loc`, `std`, and `pkg` packages lower coherently.

### Phase 8. Unsupported And Verifier Hardening

- `8.1` `pending` Audit the current typed `V1` surface and list any remaining typed forms without a lowering rule.
- `8.2` `pending` Convert user-triggerable fallback paths from `Internal`/`InvalidInput` into explicit lowering diagnostics.
- `8.3` `pending` Add verifier checks for dangling lowered references, mismatched type IDs, and impossible mounted-symbol ownership.
- `8.4` `pending` Add negative tests for every intentionally unsupported lowering surface.

### Phase 9. CLI And Debug Handoff

- `9.1` `pending` Add root-CLI support for lowering a fully typechecked workspace.
- `9.2` `pending` Add debug/snapshot output for lowered IR so tests and humans can inspect it deterministically.
- `9.3` `pending` Add end-to-end CLI tests for successful lowering across local, std, and pkg graphs.
- `9.4` `pending` Add end-to-end CLI tests for lowering failures and structured JSON diagnostics.

### Phase 10. Documentation Closeout

- `10.1` `pending` Update `README.md` and `PROGRESS.md` to describe `fol-lower` as the next implemented compiler stage.
- `10.2` `pending` Update `VERSIONS.md` references where the `V1` pipeline is described so it includes lowering on the road to a binary.
- `10.3` `pending` Update relevant book/compiler docs to explain the role of lowering without promising backend details too early.
- `10.4` `pending` Rewrite this file into a completion record only after the lowering stage is truly implemented and test-backed.

## 14. After This Plan

Only after `fol-lower` is complete should the project choose the first real
backend path.

That later decision can compare:

- a simple C backend
- an LLVM backend
- another native backend strategy

But that decision should happen after the IR exists, not before.

## 15. Future Note On C ABI

Full C ABI support is not part of this plan.

When it arrives later, it should be split across multiple layers:

- `fol-package` for package/build ownership of foreign artifacts
- a future FFI/C-ABI layer for header/object/library modeling
- `fol-resolver` for foreign name mounting
- `fol-typecheck` for FOL-to-C type compatibility
- later backend/link stages for actual ABI lowering and linking

That belongs to a later version boundary, not to this `V1` lowering plan.
