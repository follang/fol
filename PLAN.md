# FOL Backend Plan

Last updated: 2026-03-16

This file defines the next compiler milestone: create `fol-backend`, the first
real backend stage that turns the current lowered `V1` compiler output into a
runnable artifact.

This backend plan intentionally starts with **Rust emission and Rust-based
builds**, while keeping the crate boundary generic enough that a later LLVM
backend can be added without re-architecting the compiler again.

The current implemented `V1` chain is:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck -> fol-lower -> fol-runtime`

The next chain should become:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck -> fol-lower -> fol-runtime -> fol-backend`

## 0. What `fol-backend` Is

`fol-backend` is **not**:

- another front-end phase
- another semantic checker
- the runtime support crate
- the package manager
- the standard library
- an optimizer
- an LLVM backend yet

`fol-backend` **is**:

- the code-generation and artifact-production layer
- the owner of target-emission strategy
- the owner of generated crate/file/module layout
- the owner of backend symbol naming and mangling
- the owner of build orchestration for the first runnable artifacts
- the layer that consumes lowered IR plus `fol-runtime` and produces binaries

For the first milestone, `fol-backend` will target **Rust source emission**
internally.

That means:

- the crate is still named `fol-backend`
- the first implemented target inside it is Rust
- later LLVM work may either:
  - add another target module inside `fol-backend`, or
  - split out into another backend crate once the generic backend interface is stable

## 1. Why Backend Is The Next Step

The current compiler already has:

- package loading
- name resolution
- typed workspace semantics
- lowered workspace IR
- a stable runtime support crate for current `V1`

So the main remaining `V1` gap is no longer “what does this program mean?”
The gap is:

- how do we turn that lowered program into a runnable artifact?

That is exactly backend work.

Without a backend, the current compiler can:

- parse
- resolve
- type-check
- lower
- dump the lowered IR

but it still cannot:

- emit a native artifact
- emit target source code
- produce a runnable binary

So `fol-backend` is now the correct next step.

## 2. First-Backend Direction

The first backend should target **generated Rust crates**.

That choice is pragmatic:

- `fol-runtime` is already Rust
- generated output is inspectable
- build/debug iteration is faster than going directly to LLVM
- the current `V1` support surface maps well to Rust
- recoverable error handling maps naturally to Rust-style branching/matches

The first backend should support two modes:

- **emit mode**
  - generate a Rust crate and stop
- **build mode**
  - generate a Rust crate
  - compile it
  - return a runnable artifact path

So the CLI story later should look like:

- `fol build app/`
- `fol build --emit-rust app/`
- `fol build --emit-rust --keep-build-dir app/`

The exact CLI spellings can be finalized later, but the backend design should
assume both emission and build workflows from the start.

## 3. Core Design Decision: Generate A Rust Crate, Not One File

The first backend should **not** dump one giant Rust file.

It should generate:

- one temporary Rust crate per lowered FOL workspace

That generated crate should:

- depend on `fol-runtime`
- contain modules grouped by lowered package and namespace
- contain one Rust entrypoint for the selected FOL entry routine
- compile through Cargo first

This matters because FOL programs already have:

- multiple packages
- multiple namespaces
- imported `loc`, `std`, and `pkg` graphs
- lowered package identity and export metadata

Those structures should become a generated crate/module tree, not a flat file.

## 4. What `fol-backend` Must Own

The backend crate should own:

- backend session and orchestration
- target selection interface
- generated artifact directories
- Rust crate layout generation
- Rust module file generation
- symbol naming and mangling
- type emission
- global emission
- routine signature emission
- routine body/control-flow emission
- intrinsic emission strategy
- runtime integration calls
- entrypoint emission
- cargo/rustc invocation
- backend diagnostics
- source-to-output traceability for debugging

## 5. What `fol-backend` Must Not Own

The backend crate must not take over:

- parsing
- package loading
- name resolution
- type checking
- lowering
- runtime semantics
- `core` / `std`
- C ABI
- optimization passes beyond tiny local backend necessities

Those boundaries should stay intact.

## 6. Backend Input Contract

The backend input should be:

- `LoweredWorkspace`

from:

- [`fol-lower/src/model.rs`](./fol-lower/src/model.rs)
- [`fol-lower/src/control.rs`](./fol-lower/src/control.rs)
- [`fol-lower/src/types.rs`](./fol-lower/src/types.rs)

The backend must not consume:

- parser AST directly
- resolved programs directly
- typed programs directly

Everything should go through the lowered IR boundary first.

## 7. High-Level Backend Shape

The first backend should likely expose:

- a public `Backend` entry API
- a `BackendSession`
- a `BackendConfig`
- a `BackendArtifact`
- a `BackendError`
- a Rust-specific emitter module

One likely internal structure:

- `fol-backend/src/lib.rs`
- `fol-backend/src/config.rs`
- `fol-backend/src/error.rs`
- `fol-backend/src/session.rs`
- `fol-backend/src/model.rs`
- `fol-backend/src/layout.rs`
- `fol-backend/src/names.rs`
- `fol-backend/src/write.rs`
- `fol-backend/src/build.rs`
- `fol-backend/src/rust/mod.rs`
- `fol-backend/src/rust/types.rs`
- `fol-backend/src/rust/globals.rs`
- `fol-backend/src/rust/routines.rs`
- `fol-backend/src/rust/control.rs`
- `fol-backend/src/rust/exprs.rs`
- `fol-backend/src/rust/modules.rs`
- `fol-backend/src/rust/entry.rs`

This is only a likely split, but the important part is:

- generic backend session at the crate root
- Rust target logic isolated under a target-specific module

## 8. Output Model

The backend should produce a structured result, not just “write files and hope”.

Likely backend result model:

- generated crate root path
- emitted source file list
- selected build mode
- produced artifact path if compilation succeeded
- entry routine identity
- target kind

So a later backend API can return something like:

- `BackendArtifact::RustSourceCrate { root, files }`
- `BackendArtifact::CompiledBinary { crate_root, binary_path }`

## 9. Artifact Directory Strategy

The backend needs a deterministic working directory model.

The first version should generate under a backend-owned build directory, for example:

- `target/fol-backend/<workspace-hash>/`

Inside that directory:

- `Cargo.toml`
- `src/main.rs` or `src/lib.rs`
- generated package/namespace modules
- optional backend metadata/debug files

Recommended first policy:

- deterministic directory name derived from entry package identity + content hash
- overwrite-safe recreation
- optional “keep generated crate” mode for debugging

This should avoid random temp directories by default, because inspection matters
for early backend bring-up.

## 10. Generated Rust Crate Layout

The first backend should generate one Rust crate with:

- one crate root
- one module tree grouped by lowered package + namespace
- one entry wrapper

Recommended shape:

```text
target/fol-backend/<hash>/
  Cargo.toml
  src/
    main.rs
    runtime_prelude.rs        # optional helper re-exports if useful
    packages/
      entry_pkg/
        mod.rs
        root.rs
        sub_a.rs
        sub_b.rs
      imported_pkg/
        mod.rs
        root.rs
        net.rs
```

The backend should **not** preserve one Rust file per source `.fol` file unless
that later proves necessary. The semantic grouping should be:

- by lowered package
- then by lowered namespace

not by original source-file count.

## 11. Package And Namespace Mapping

The backend should map:

- each lowered package identity
- into one generated Rust top-level package module

and then:

- each lowered namespace
- into one generated Rust submodule

This is the cleanest mapping because:

- FOL already treats same-folder files as one package scope
- subfolders already become namespaces
- lowered workspaces already retain package/source-unit/namespace identity

The backend should therefore:

- merge declarations by namespace
- not create target module boundaries based on incidental file splits

## 12. Entry Selection

The backend should use lowered entry candidates from:

- `LoweredWorkspace::entry_candidates()`

The first backend must define:

- how one entry routine is selected
- what happens if there are zero entry candidates
- what happens if there are multiple plausible entry candidates

Recommended first policy:

- require exactly one backend-buildable entry candidate in normal build mode
- error explicitly on zero or multiple candidates

The backend should then generate:

- a Rust `main()` wrapper for process-style entry
- or later a library-style surface if/when that mode is added

## 13. Name Mangling

Generated Rust names must be deterministic and collision-safe.

The backend should not rely on raw source names alone because later we may need
to survive:

- duplicate short names across packages
- reserved Rust keywords
- same routine names across namespaces
- future overload-like expansions

Recommended rule:

- keep human-readable suffixes where useful
- prefix with kind + stable backend ID

Examples:

- type: `t_14_User`
- global: `g_8_default_name`
- routine: `r_22_main`
- local: `l_5_current`

This gives:

- deterministic output
- collision resistance
- still somewhat readable generated Rust

The mangling plan should also preserve:

- package identity
- namespace identity
- entry wrapper names

## 14. Runtime Dependency Integration

The first backend must depend on:

- `fol-runtime`

That dependency should be path-based at first.

Generated Rust should import runtime items through:

- `fol_runtime`

and preferably alias:

- `use fol_runtime::prelude as rt;`

The backend should not:

- inline custom runtime clones into generated crates
- duplicate `len`, `echo`, recoverable, or shell semantics directly in emission

## 15. Type Mapping Strategy

The backend must map lowered `V1` types into Rust target types.

### 15.1 Native-Mappable Scalars

Current likely direct mapping:

- `int` -> backend chosen signed integer type
- `flt` -> backend chosen floating type
- `bol` -> `bool`
- `chr` -> `char`
- `never` -> `!` where possible, or explicit control-flow-only handling

The exact Rust integer/float choice must be frozen early.

Recommended first policy:

- follow the already-frozen runtime scalar aliases in `fol-runtime`
- do not invent a second scalar mapping in the backend

### 15.2 Runtime-Mapped Types

These should map to runtime types:

- `str` -> `fol_runtime::strings::FolStr`
- `vec[T]` -> `fol_runtime::containers::FolVec<T>`
- `seq[T]` -> `fol_runtime::containers::FolSeq<T>`
- `set[T]` -> `fol_runtime::containers::FolSet<T>`
- `map[K, V]` -> `fol_runtime::containers::FolMap<K, V>`
- `opt[T]` -> `fol_runtime::shell::FolOption<T>`
- `err[T]` -> `fol_runtime::shell::FolError<T>`
- `T / E` -> `fol_runtime::abi::FolRecover<T, E>`

### 15.3 Aggregate Types

Records and entries should become emitted Rust structs/enums.

The backend should also emit:

- `impl FolRecord` for records where runtime formatting is required
- `impl FolEntry` for entries where runtime formatting is required

## 16. Global Emission

The backend must decide how lowered globals become Rust globals.

Open design question:

- emit as `const`
- emit as `static`
- emit as lazy initialization wrappers

Recommended first rule:

- immutable compile-time scalar/string/container literals that are easy to emit
  directly may become simple static-style definitions if Rust permits
- otherwise, emit helper functions or initialization routines rather than
  overcomplicating static initialization immediately

The important thing is not to block the first backend on perfect global
initialization purity.

## 17. Routine Signature Emission

The backend must lower:

- ordinary routines
- receiver-qualified routines
- entry routines
- recoverable routines

Each lowered routine signature should become one Rust function with:

- stable mangled function name
- target parameter list
- target return type

Recoverable routines should return:

- `FolRecover<T, E>`

not ad hoc tuples.

## 18. Routine Body Emission

The backend must lower:

- locals
- instructions
- block structure
- terminators

Because `fol-lower` already gives explicit blocks and terminators, the backend
should preserve that shape instead of trying to reconstruct high-level syntax.

Recommended initial style:

- emit structured Rust blocks/matches/loops where clean
- fall back to explicit block labels/state-machine style only when needed

The first backend does not need elegant emitted Rust.
It needs:

- deterministic
- correct
- inspectable

## 19. Lowered Instruction Mapping

The backend must explicitly map each current lowered instruction kind.

### 19.1 Plain Instruction Families

These are expected to map mostly directly:

- `Const`
- `LoadLocal`
- `StoreLocal`
- `LoadGlobal`
- `StoreGlobal`
- `Call`
- `FieldAccess`

### 19.2 Intrinsic Instruction Families

These split into:

- native target operations
- runtime helper calls

Current rule from `fol-runtime`:

- `.eq`, `.nq`, `.lt`, `.gt`, `.ge`, `.le` -> native Rust comparisons
- `.not` -> native Rust negation
- `.len` -> runtime helper
- `.echo` -> runtime helper
- `check` -> runtime helper

### 19.3 Runtime-Shaped Instruction Families

These must respect the runtime contract:

- `CheckRecoverable`
- `UnwrapRecoverable`
- `ExtractRecoverableError`
- `ConstructLinear`
- `ConstructSet`
- `ConstructMap`
- `ConstructOptional`
- `ConstructError`
- `IndexAccess`
- `UnwrapShell`

## 20. Terminator Mapping

The backend must explicitly map:

- `Jump`
- `Branch`
- `Return`
- `Report`
- `Panic`
- `Unreachable`

The hardest ones are:

- `Report`
- `Panic`

because they define the difference between:

- recoverable failure
- unrecoverable abort

Current rule:

- `Report` constructs or forwards the error lane of `FolRecover`
- `Panic` aborts control flow via target panic strategy

## 21. Error-Handling Codegen Contract

The backend must make current `V1` recoverable behavior executable.

That includes:

- success path values
- error path values
- propagation
- `check(...)`
- `expr || fallback`
- top-level recoverable outcomes

The backend should preserve the lowering/runtime distinction already locked:

- routine recoverable results are **not** shell values
- shell unwrap `!` is **not** routine-call error handling

So the backend must not collapse:

- `FolRecover<T, E>`
- `FolError<T>`
- `FolOption<T>`

into one accidental representation.

## 22. Container Codegen Contract

The backend must make the current `V1` container subset executable through the
runtime contract.

Current required families:

- arrays
- vectors
- sequences
- sets
- maps

The backend should rely on:

- runtime constructors
- runtime length helpers
- runtime rendering helpers where exposed through `.echo(...)`
- runtime indexing helpers when direct native indexing would violate the
  current contract

## 23. Aggregate Codegen Contract

The backend must emit:

- records
- entries

with stable field/variant layouts derived from lowered types.

For current `V1`, backend-emitted aggregates must support:

- construction
- field access
- entry construction
- entry inspection/rendering where needed through runtime formatting hooks

## 24. Backend Diagnostics

The backend needs its own structured diagnostics layer, integrated with
`fol-diagnostics`.

It should report:

- impossible lowered shapes
- unsupported backend target situations
- generated crate write failures
- cargo/rustc invocation failures
- entry-selection failures
- emitted-code invariant failures

These should not become generic IO errors without compiler context.

## 25. Generated Source Debuggability

The first backend should optimize for inspectability.

That means:

- stable generated file layout
- stable name mangling
- optional comments linking emitted items back to lowered symbol names
- optional source map / emitted metadata file
- deterministic output for the same lowered workspace

The backend should also support:

- explicit “emit Rust and stop”
- explicit “keep generated crate”

because backend bring-up will need inspection constantly.

## 26. Cargo Integration

The first build mode should use:

- generated `Cargo.toml`
- path dependency on local `fol-runtime`
- `cargo build`

Cargo first is the pragmatic choice because:

- `fol-runtime` is a Rust crate
- dependency setup is simpler
- debugging generated crates is easier

Direct `rustc` invocation can remain later optimization work.

## 27. Build Profile Policy

The backend should decide early how it builds generated crates.

Recommended first policy:

- debug builds by default during bring-up
- explicit release mode later
- deterministic crate name and target path

So the first backend contract should support:

- debug compile
- later release compile

without changing emission structure.

## 28. CLI Integration

The root CLI will need a backend step after lowering.

Later CLI surface will likely need:

- compile to binary
- emit generated Rust only
- keep or clean generated crate
- maybe print output binary path

But the backend plan should first stabilize the internal API.

CLI work should come after:

- crate foundation
- emission
- cargo build orchestration

## 29. Testing Strategy

Backend tests should mirror the earlier compiler stages:

- unit tests for naming/layout helpers
- crate-generation tests
- emission snapshot tests
- compile-smoke tests
- end-to-end CLI integration tests

The first backend should not rely only on “generated Rust compiles”.
It also needs:

- deterministic file-shape tests
- stable output content checks
- runtime behavior checks through produced binaries

## 30. Snapshot Strategy

The backend should add stable snapshot coverage for:

- generated `Cargo.toml`
- generated module tree
- generated Rust function/type layout
- emitted entry wrapper
- recoverable error emission shape

These can be text fixtures similar to `--dump-lowered`, but now for emitted Rust.

## 31. Behavior Test Strategy

The backend also needs executable behavior tests for:

- scalar return values
- record construction and field access
- container creation and indexing
- recoverable success/failure propagation
- `.len(...)`
- `.echo(...)`
- `panic(...)`
- multi-package import graphs
- same-package multi-file visibility
- namespace module emission

## 32. Determinism Requirements

The backend should be deterministic in:

- generated crate directory naming
- file ordering
- module ordering
- emitted declaration ordering
- symbol mangling
- dependency entries

Determinism matters for:

- debugging
- test snapshots
- future caching

## 33. Future LLVM Compatibility

Even though the first backend is Rust-based, the plan should not hard-wire
every abstraction to Rust syntax.

So the backend crate should keep these boundaries generic:

- backend config
- artifact model
- emitted symbol model
- target selection enum
- backend diagnostics

Rust-specific logic can live under:

- `backend::rust::*`

Later LLVM work should be able to reuse:

- the backend session shell
- target selection
- artifact reporting
- entry selection policy

without changing front-end or runtime ownership again.

## 34. Explicitly Out Of Scope For This Backend Milestone

This backend plan does **not** include:

- LLVM backend implementation
- optimization passes
- C ABI
- `core` / `std`
- link-time foreign artifact handling
- ownership/borrowing codegen
- V2 language semantics
- V3 systems semantics

The goal is:

- first runnable `V1` binaries

not full long-term compiler completion.

## 35. Phase 0: Contract Freeze

#### 0.1 done
- replace the closed runtime record in `PLAN.md` with this backend plan

#### 0.2 done
- freeze first-backend direction as Rust emission under the generic `fol-backend` crate boundary

#### 0.3 done
- freeze the rule that the backend consumes `LoweredWorkspace`, not AST, resolver, or typed-workspace inputs

#### 0.4 done
- freeze the rule that the first backend generates one Rust crate per lowered workspace, not one giant Rust file

#### 0.5 done
- freeze the rule that `fol-runtime` is the only support dependency for current `V1` execution semantics

## 36. Phase 1: Crate Foundation

#### 1.1 done
- create `fol-backend` as a workspace crate

#### 1.2 done
- add public API shell for backend entrypoints

#### 1.3 done
- add structured backend error types

#### 1.4 done
- add backend config model

#### 1.5 done
- add artifact/result model

#### 1.6 done
- add smoke tests for crate foundation and public API

## 37. Phase 2: Session And Workspace Intake

#### 2.1 done
- add backend session over `LoweredWorkspace`

#### 2.2 done
- retain entry package identity, package graph, and entry candidates inside backend session state

#### 2.3 done
- add stable workspace hashing / output-directory identity

#### 2.4 done
- add initial emitted source map / trace model for backend outputs

#### 2.5 done
- add tests for backend session identity and deterministic workspace hashing

## 38. Phase 3: Name Mangling And Layout

#### 3.1 done
- add stable name-mangling helpers for types, globals, routines, locals, and modules

#### 3.2 done
- add package-to-module layout planning

#### 3.3 done
- add namespace-to-module-file layout planning

#### 3.4 done
- add generated crate directory layout planner

#### 3.5 done
- add tests locking deterministic name mangling

#### 3.6 done
- add tests locking package/namespace layout planning

## 39. Phase 4: Rust Crate Skeleton Emission

#### 4.1 done
- emit `Cargo.toml` with path dependency on `fol-runtime`

#### 4.2 done
- emit crate root shell (`main.rs` or equivalent)

#### 4.3 done
- emit top-level package module shells

#### 4.4 done
- emit namespace module shells

#### 4.5 done
- add snapshot tests for generated crate skeleton shape

## 40. Phase 5: Type Emission

#### 5.1 done
- emit builtin/native type mappings through backend-owned Rust type renderers

#### 5.2 done
- emit runtime-backed type mappings for `str`, containers, shells, and recoverable results

#### 5.3 done
- emit record struct definitions

#### 5.4 done
- emit entry enum definitions

#### 5.5 done
- emit `FolRecord` implementations for backend-authored record types where needed

#### 5.6 done
- emit `FolEntry` implementations for backend-authored entry types where needed

#### 5.7 done
- add snapshot tests for emitted type shapes

## 41. Phase 6: Global And Routine Signature Emission

#### 6.1 done
- emit global declarations for straightforward current `V1` cases

#### 6.2 done
- emit routine signatures including receiver-qualified routines

#### 6.3 done
- emit recoverable routine return types as `FolRecover<T, E>`

#### 6.4 done
- emit local declarations and routine frame shells

#### 6.5
- add snapshot tests for globals and routine signatures

## 42. Phase 7: Core Instruction Emission

#### 7.1
- emit `Const`, `LoadLocal`, `StoreLocal`, `LoadGlobal`, and `StoreGlobal`

#### 7.2
- emit plain routine calls

#### 7.3
- emit field access

#### 7.4
- emit scalar intrinsic calls as native Rust operations

#### 7.5
- add snapshot tests for core instruction emission

## 43. Phase 8: Runtime-Shaped Instruction Emission

#### 8.1
- emit runtime-backed `.len(...)`

#### 8.2
- emit runtime-backed `.echo(...)`

#### 8.3
- emit `CheckRecoverable`

#### 8.4
- emit `UnwrapRecoverable`

#### 8.5
- emit `ExtractRecoverableError`

#### 8.6
- emit `ConstructOptional`

#### 8.7
- emit `ConstructError`

#### 8.8
- emit `UnwrapShell`

#### 8.9
- add snapshot tests for runtime-shaped instruction emission

## 44. Phase 9: Aggregate And Container Emission

#### 9.1
- emit record construction

#### 9.2
- emit entry construction

#### 9.3
- emit arrays

#### 9.4
- emit vectors and sequences

#### 9.5
- emit sets and maps

#### 9.6
- emit runtime-backed indexing

#### 9.7
- add snapshot tests for aggregate/container emission

## 45. Phase 10: Control-Flow And Terminator Emission

#### 10.1
- emit `Jump`

#### 10.2
- emit `Branch`

#### 10.3
- emit `Return`

#### 10.4
- emit `Report`

#### 10.5
- emit `Panic`

#### 10.6
- emit `Unreachable`

#### 10.7
- add snapshot tests for control-flow emission

## 46. Phase 11: Entry Wrapper And Process Outcome

#### 11.1
- select one backend-buildable entry routine

#### 11.2
- emit Rust `main()` wrapper for ordinary return routines

#### 11.3
- emit Rust `main()` wrapper for recoverable entry routines through `FolProcessOutcome`

#### 11.4
- emit exit-code and printable-failure handling through runtime helpers

#### 11.5
- add executable tests for entry success/failure behavior

## 47. Phase 12: Build Orchestration

#### 12.1
- add generated-crate writer

#### 12.2
- add build directory creation/cleanup policy

#### 12.3
- add Cargo invocation support

#### 12.4
- capture cargo failure diagnostics and surface them as backend diagnostics

#### 12.5
- add tests that generated crates compile successfully for current `V1` samples

## 48. Phase 13: Emit Mode And Debuggability

#### 13.1
- add “emit Rust only” backend mode

#### 13.2
- add “keep build dir” mode

#### 13.3
- add optional emitted manifest/source summary output

#### 13.4
- add snapshot tests for emitted full Rust crates

## 49. Phase 14: CLI Integration

#### 14.1
- wire `fol-backend` after lowering in the root CLI

#### 14.2
- add CLI flag for emit-only mode

#### 14.3
- add CLI flag for keeping generated backend artifacts

#### 14.4
- add CLI output for final artifact path

#### 14.5
- add end-to-end CLI tests for successful binary builds

#### 14.6
- add end-to-end CLI tests for backend diagnostics on emission/build failures

## 50. Phase 15: Behavioral End-To-End Coverage

#### 15.1
- add executable end-to-end tests for scalar programs

#### 15.2
- add executable end-to-end tests for records and entries

#### 15.3
- add executable end-to-end tests for containers and `.len(...)`

#### 15.4
- add executable end-to-end tests for `.echo(...)`

#### 15.5
- add executable end-to-end tests for recoverable success/failure propagation

#### 15.6
- add executable end-to-end tests for `check(...)`

#### 15.7
- add executable end-to-end tests for `expr || fallback`

#### 15.8
- add executable end-to-end tests across `loc`, `std`, and `pkg` package graphs

## 51. Phase 16: Backend Hardening

#### 16.1
- add backend verification for impossible lowered-to-target situations

#### 16.2
- add deterministic emission-order tests

#### 16.3
- add stable source-to-output traceability checks

#### 16.4
- audit all currently supported lowered `V1` surfaces and ensure they either emit or fail explicitly with backend diagnostics

## 52. Phase 17: Docs Closeout

#### 17.1
- update `README.md`

#### 17.2
- update `PROGRESS.md`

#### 17.3
- update the book where backend/runtime/build output behavior needs acknowledgement

#### 17.4
- rewrite `PLAN.md` into a backend completion record only after the first backend is real and validated

## 53. Definition Of Done

`fol-backend` is done for the first `V1` milestone only when all of the following are true:

- a `fol-backend` workspace crate exists
- it consumes `LoweredWorkspace`
- it generates a deterministic Rust crate for current lowered `V1` workspaces
- the generated crate depends on `fol-runtime`
- current `V1` lowered types, containers, shells, recoverable results, aggregates, and implemented intrinsics emit correctly
- current `V1` entry routines can compile into runnable binaries
- CLI integration can build binaries and emit generated Rust
- backend failures surface as structured diagnostics
- end-to-end executable tests are green across representative `loc`, `std`, and `pkg` graphs
- repo/docs are synced to the backend milestone
