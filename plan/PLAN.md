# Replace Cargo Artifact Builds With Direct `rustc`

Last updated: 2026-03-22

## Goal

Stop using Cargo as the executable build driver for generated FOL program
artifacts.

The short-term target is:

- keep the current Rust backend
- keep one Rust codegen pipeline
- compile generated binaries with direct `rustc`
- keep `emit rust` as the human/debug/export workflow

The long-term target is:

- keep backend interfaces clean enough that an LLVM backend can later be added
- avoid deepening Cargo coupling while the Rust backend still exists

## Non-goals

- Do not embed `rustc` internals into `fol`
- Do not build a new LLVM backend in this plan
- Do not maintain two different Rust code generators
- Do not attempt to reproduce Cargo's internal rustc command lines exactly
- Do not preserve Cargo as the product build authority once direct `rustc`
  parity is established

## Current state

Today a FOL package build works like this:

1. `build.fol` is evaluated into a build graph
2. the frontend selects the target artifact/root module
3. the compiler produces lowered FOL IR
4. `fol-backend` emits a temporary Rust crate
5. `fol-backend` runs `cargo build --manifest-path <generated>/Cargo.toml --release`
6. the built binary is copied into the backend output `bin/` directory

Important repo facts that make direct `rustc` feasible:

- generated artifact crates currently depend only on `fol-runtime`
- `fol-runtime` currently has no third-party Cargo dependencies
- there is no obvious proc-macro/build-script/native-library complexity in this
  artifact build path
- the generated Rust crate layout is already deterministic and backend-owned

## Product direction

The project should treat Rust emission as the current backend language, not as
the final architectural destination.

That means:

- near term: Rust backend, built directly with `rustc`
- medium term: backend build orchestration fully owned by `fol-backend`
- long term: optional LLVM backend beside or instead of the Rust backend

This separates two concerns cleanly:

- build driver choice: Cargo vs `rustc`
- backend choice: Rust vs LLVM

Replacing Cargo with `rustc` is a build-system/backend-integration task.
Replacing Rust with LLVM is a compiler-backend task.

## Success criteria

The plan is successful when all of the following are true:

- `fol code build` does not invoke Cargo for generated program artifacts
- `fol code run` and `fol code test` also avoid Cargo through the same artifact
  build path
- `fol code emit rust` still writes a valid Cargo-compatible crate to disk
- generated Rust source stays the single source of truth for the Rust backend
- backend tests validate that direct `rustc` builds run the same fixtures that
  previously passed under Cargo
- backend architecture is cleaner after the change, not more entangled

## Slice tracker

- [x] Slice 1: add explicit backend build path planning
- [x] Slice 2: isolate Cargo build driver behind a dedicated function boundary
- [x] Slice 3: add explicit backend build profile/config plumbing
- [x] Slice 4: add runtime source/build-root resolution helpers
- [x] Slice 5: compile `fol-runtime` to `.rlib` with direct `rustc`
- [x] Slice 6: compile generated crates with direct `rustc`
- [x] Slice 7: add rustc parity tests alongside Cargo-backed tests
- [x] Slice 8: introduce explicit artifact build modes for Cargo vs `rustc`
- [ ] Slice 9: switch normal artifact builds to direct `rustc`
- [ ] Slice 10: remove Cargo artifact-build dependency and tighten docs

## Design rules

1. One Rust code generator only.
   The same emitted Rust files must drive both `emit rust` and binary builds.

2. Backend-owned compilation.
   `fol-backend` should explicitly compile support/runtime artifacts and then
   compile the generated entry crate.

3. Cargo-compatible emission remains available.
   `emit rust` should keep producing a crate a human can inspect or build with
   Cargo if desired.

4. Product build path should not depend on Cargo.
   Cargo may exist as a temporary fallback or developer convenience during
   migration, but not as the final required path.

5. No fake "sync" with Cargo internals.
   "In sync" means same generated Rust and same observable behavior, not
   byte-for-byte reproduction of Cargo's private rustc flag selection.

## Proposed architecture

### Backend layers

Split the backend into three explicit responsibilities:

1. Rust emission
   Input: lowered workspace
   Output: emitted Rust file set and crate layout metadata

2. Rust artifact preparation
   Input: emitted file set
   Output: materialized crate root on disk, runtime build directory, output dir

3. Rust compilation driver
   Input: crate roots, runtime crate source, target/profile config
   Output: native executable path

### Backend modes

Introduce a more explicit backend artifact/build model:

- `EmitSource`
  Writes the generated Rust crate and returns its root

- `BuildArtifactWithRustc`
  Writes the generated Rust crate, builds runtime support with `rustc`, then
  builds the final artifact with `rustc`

- optional temporary `BuildArtifactWithCargo`
  Exists only during migration/testing and is deleted once parity is proven

### Single source of truth

The emitted Rust crate contents remain authoritative:

- `src/main.rs`
- `src/packages/...`
- the crate name/layout chosen by the backend
- runtime import contract through `fol_runtime`

`Cargo.toml` is output metadata for Cargo-facing workflows, not the build graph
authority for product builds.

## Phase 1: Refactor backend build responsibilities

Goal:
Make the current Cargo path structurally replaceable before changing behavior.

Tasks:

- extract "emit generated files to disk" from "compile generated crate"
- define a dedicated backend build context with:
  - output root
  - build root
  - bin root
  - generated crate root
  - runtime build root
  - selected profile
  - optional target triple
- isolate current Cargo invocation behind an internal build-driver function
- keep all external behavior unchanged in this phase

Expected result:

- the backend can materialize the generated crate without immediately deciding
  how it is compiled
- the compilation driver becomes a pluggable internal detail

## Phase 2: Add direct `rustc` compilation for `fol-runtime`

Goal:
Compile `fol-runtime` explicitly as a backend-owned support artifact.

Tasks:

- add a backend function to compile `lang/execution/fol-runtime/src/lib.rs`
  into an `.rlib`
- choose a deterministic backend-owned output path, for example:
  - `<output>/fol-backend/runtime/<profile>/libfol_runtime.rlib`
- pass at minimum:
  - `--crate-name fol_runtime`
  - `--crate-type rlib`
  - `--edition=2021`
  - optimization/profile flags
  - `--out-dir <runtime-output-dir>`
- make runtime source discovery explicit instead of Cargo-manifest-driven
- preserve the existing runtime path override behavior where sensible, but
  redefine it in terms of source/build input rather than Cargo path dependency

Expected result:

- the backend can produce a reusable `fol-runtime` library artifact without
  invoking Cargo

## Phase 3: Add direct `rustc` compilation for generated binaries

Goal:
Compile the emitted Rust entry crate directly with `rustc`.

Tasks:

- compile generated `src/main.rs` with:
  - `--crate-name <generated-name>`
  - `--edition=2021`
  - `--extern fol_runtime=<path-to-built-runtime-rlib>`
  - profile flags
  - output path under backend `bin/`
- ensure module resolution works from the generated crate root
- ensure crate and output naming remain deterministic
- return the final native binary path through the existing backend artifact API

Expected result:

- the backend can produce a working native executable using only `rustc` plus
  the system linker/toolchain

## Phase 4: Define explicit profile and target policy

Goal:
Replace Cargo's implicit defaults with backend-owned policy.

Tasks:

- define profile mappings:
  - debug -> no/low optimization, debuginfo policy as chosen
  - release -> optimized build policy
- define the initial target support contract:
  - first milestone: host-only builds are acceptable
  - later milestone: explicit `--target <triple>` support
- thread selected target/profile from frontend build options into backend build
  driver config
- reject unsupported target configurations with backend-owned diagnostics

Expected result:

- `fol-backend` is the authority on how build profile and target affect Rust
  compilation

## Phase 5: Switch product artifact builds to `rustc`

Goal:
Make direct `rustc` the default artifact build path.

Tasks:

- change backend build mode used by:
  - `fol code build`
  - `fol code run`
  - `fol code test`
  - direct-file build/run paths
- keep `emit rust` unchanged as source emission
- keep temporary Cargo fallback only behind a test-only or internal option if
  still needed during migration

Expected result:

- normal user-facing binary builds no longer invoke Cargo

## Phase 6: Establish parity tests

Goal:
Prove that direct `rustc` is behaviorally equivalent to the prior Cargo path
for the current backend scope.

Tasks:

- add backend integration tests that:
  - build a fixture with direct `rustc`
  - run the produced binary
  - assert expected output/exit status
- during migration, add comparison tests that run the same emitted program
  through both Cargo and `rustc`
- validate:
  - scalar programs
  - recoverable entrypoints
  - runtime helpers such as `.echo` and `.len`
  - multi-package workspace graphs
  - container/runtime-backed values
- validate emitted source still remains Cargo-buildable for `emit rust`

Expected result:

- removal of Cargo is justified by test evidence instead of assumption

## Phase 7: Remove Cargo from artifact build mode

Goal:
Delete the old artifact-build path once direct `rustc` is proven.

Tasks:

- remove `cargo build` invocation from `fol-backend`
- delete temporary dual-path logic
- simplify backend diagnostics around build failures to talk in terms of
  explicit `rustc` and linker execution
- keep `Cargo.toml` generation only if still useful for `emit rust`

Expected result:

- the product build path is simpler and fully backend-owned

## Phase 8: Clean up runtime and emission contracts

Goal:
Use the migration to make later backend work easier.

Tasks:

- make runtime-path discovery explicit and backend-owned
- document the runtime ABI expected by emitted Rust code
- document the exact emitted crate invariants relied on by direct `rustc`
- avoid sneaking Cargo assumptions back into generated-crate layout

Expected result:

- future backend work starts from a cleaner contract boundary

## Known risks

### Risk 1: Linker/platform differences

Cargo currently hides some toolchain selection and linker behavior.

Mitigation:

- support host-only builds first
- keep diagnostics precise when `rustc` or linker execution fails
- defer full cross-target support until the host path is stable

### Risk 2: Runtime artifact management

Building `fol-runtime` manually means the backend owns caching and output paths.

Mitigation:

- start with a simple deterministic rebuild policy
- add caching only after correctness is established

### Risk 3: Hidden future dependencies

If `fol-runtime` later gains external crates, the direct build path becomes more
complex.

Mitigation:

- treat `fol-runtime` dependency growth as an architectural decision
- if the runtime must stay directly buildable with `rustc`, keep its dependency
  surface intentionally minimal

### Risk 4: False sense of completion

Replacing Cargo with `rustc` does not make the backend "LLVM-based" or "native"
in the architectural sense.

Mitigation:

- document clearly that this is a build-driver change, not a non-Rust backend

## LLVM follow-up plan

LLVM is a separate roadmap and should only begin after the `rustc` migration is
stable.

The intended sequence is:

1. own the artifact build path with `rustc`
2. stabilize backend/runtime contracts
3. define a backend-agnostic artifact interface
4. prototype an LLVM backend beside the Rust backend
5. compare semantics and runtime ABI behavior
6. decide whether Rust emission remains as:
   - a reference backend
   - an `emit rust` debug/export backend
   - or a removable transitional backend

## Explicit recommendation

Do this work now:

- Cargo artifact build removal
- direct `rustc` build driver
- backend contract cleanup

Do not do this work in the same project slice:

- embedded `rustc`
- compiler-internal `rustc_driver` integration
- LLVM backend implementation

## Execution order

Recommended order:

1. Phase 1: refactor backend build responsibilities
2. Phase 2: build `fol-runtime` with direct `rustc`
3. Phase 3: build generated artifacts with direct `rustc`
4. Phase 4: make profile/target policy explicit
5. Phase 6: add parity tests while Cargo path still exists
6. Phase 5: switch product artifact builds to `rustc`
7. Phase 7: delete Cargo artifact build mode
8. Phase 8: clean up contracts and docs
9. only then start LLVM follow-up work

## Definition of done

This plan is complete when:

- generated FOL program binaries are built through direct `rustc`
- `emit rust` still produces a valid inspectable Cargo-compatible crate
- Cargo is no longer required for normal program binary builds
- the backend build contract is clearer than before
- the codebase is in a better position to add an LLVM backend later without
  first undoing Rust/Cargo build coupling
