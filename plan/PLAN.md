# Add Cross-Compilation To The `rustc` Backend

Last updated: 2026-03-22

## Goal

Add explicit cross-compilation support to the current Rust backend so FOL
programs can be built for non-host targets through direct `rustc`.

The target outcome is:

- `fol code build --target <triple>` builds a binary for that target
- `build.fol` artifact definitions can declare a target
- backend runtime artifacts and binary outputs are target-scoped
- host and cross builds can coexist without clobbering each other
- normal builds keep using the same Rust emission pipeline

This is a build-target extension of the current backend, not a new backend.

## Non-goals

- Do not add an LLVM backend in this plan
- Do not embed `rustc` internals into `fol`
- Do not reintroduce Cargo as a normal artifact build driver
- Do not attempt to support arbitrary custom linker orchestration in the first
  milestone
- Do not make cross-target `run` or `test` silently emulate binaries
- Do not preserve ambiguous or legacy target spellings without an explicit
  mapping rule

## Current state

Today the backend build path is:

1. `build.fol` is evaluated into a build graph
2. the frontend selects the target artifact/root module
3. the compiler produces lowered FOL IR
4. `fol-backend` emits a temporary Rust crate
5. `fol-backend` compiles `fol-runtime` into an `.rlib` with direct `rustc`
6. `fol-backend` compiles the generated entry crate with direct `rustc`
7. the built binary is copied into the backend output `bin/` directory

Target-related state today:

- the build/evaluation layer already has a target concept
- CLI build options already parse `--target`
- artifact definitions already have target config fields
- the backend currently ignores target selection and always builds for the host
- runtime and binary outputs are not target-scoped
- `run` and `test` assume the built binary is executable on the current host

## Problem statement

Right now target selection exists in the higher-level build model, but that
information does not reach the actual `rustc` invocations.

That leaves three gaps:

1. target intent is accepted by parts of the system but not enforced
2. build outputs are not partitioned by target
3. run/test semantics are underspecified for non-host binaries

The backend must become the authority on target-aware native artifact builds.

## Product direction

The current backend language remains Rust.

This plan is specifically about:

- target-aware artifact builds
- target-aware output layout
- target-aware frontend diagnostics

This plan is not about:

- replacing Rust with LLVM
- changing the generated Rust source model
- changing `emit rust` into a target-specific source format

## Success criteria

This plan is successful when all of the following are true:

- the backend accepts an explicit machine target triple
- the same target is applied consistently to both `fol-runtime` and the
  generated entry crate
- output paths separate artifacts by target and profile
- host-target builds keep working exactly as before
- `emit rust` remains available and target-agnostic as source emission
- non-host `run` and `test` fail with clear diagnostics instead of attempting
  execution
- tests cover target mapping, output layout, diagnostics, and host behavior

## Design rules

1. One code generator only.
   Cross-compilation must use the same emitted Rust files as host builds.

2. One artifact builder only.
   Direct `rustc` remains the product build path for generated binaries.

3. Target choice must be explicit.
   The backend should receive a concrete host/default target decision, not
   infer target behavior in scattered places.

4. Target naming must be normalized.
   FOL-facing target values may differ from Rust target triples, but the mapping
   must be explicit and centralized.

5. Outputs must be target-scoped.
   Runtime libraries, generated binaries, and related build directories must not
   collide across targets.

6. Execution is not compilation.
   Building a non-host binary is allowed; running or testing it is a separate
   concern and should be rejected unless host-compatible.

7. No hidden Cargo fallback.
   Cross-compilation support must extend the rustc backend, not route around it.

## User-facing model

The intended user-facing shape is:

```bash
fol code build --target aarch64-unknown-linux-gnu
fol code build --target x86_64-pc-windows-gnu
fol code emit rust
```

And in `build.fol`:

```fol
var app = graph.add_exe({
    name = "app",
    root = "src/main.fol",
    target = "aarch64-unknown-linux-gnu"
});
```

Target precedence should be:

1. explicit CLI target
2. artifact target declared in `build.fol`
3. host default

That precedence should be implemented once and then threaded forward.

## Proposed architecture

### Target model

The backend needs a real machine-target concept, separate from the current
backend-language target enum.

Minimum viable shape:

- host target
- explicit target triple

For example:

```rust
pub enum BackendMachineTarget {
    Host,
    Triple(String),
}
```

or an equivalent normalized `Option<String>` field with explicit host handling.

### Rust target mapping

The build system may accept target names that are not already valid Rust target
triples.

Examples seen in the repo include values like:

- `x86_64-linux-gnu`
- `aarch64-macos-gnu`

Those are not necessarily valid `rustc --target` values. So the backend needs a
translation layer:

- FOL-facing target spelling
- normalized internal build target
- final Rust target triple used in `rustc`

This mapping must live in one place with tests.

### Target-aware output layout

Current output layout is not sufficient for cross builds because host and
cross-target artifacts would overwrite each other.

The backend should move to a target-aware layout such as:

- `.fol/build/debug/bin/<target>/<artifact>`
- `.fol/build/release/bin/<target>/<artifact>`
- `.fol/build/<profile>/fol-backend/runtime/<target>/<profile>/...`
- generated crate target outputs under target-scoped directories as needed

The exact shape can vary, but target and profile must both be encoded.

### rustc driver behavior

Both rustc invocations must receive the same target decision:

1. build `fol-runtime` with `--target <triple>`
2. build generated `src/main.rs` with `--target <triple>`

The backend remains responsible for:

- target triple selection
- output roots
- profile flags
- diagnostics when rustc or linker execution fails

## Frontend semantics

The frontend should distinguish clearly between:

- compile-only commands
- execution commands

Expected behavior:

- `build`: supports host and non-host targets
- `emit rust`: always supported
- `run`: host only in the first milestone
- `test`: host only in the first milestone unless it is explicitly redefined as
  compile-only for some cases

For non-host `run`/`test`, the frontend should fail fast with a diagnostic that
states the selected target and the current host.

## Slice tracker

- [x] Slice 1: define backend machine-target config and target normalization API
- [x] Slice 2: thread CLI/build target selection into frontend backend config
- [x] Slice 3: define FOL-target to Rust-target mapping rules with tests
- [x] Slice 4: make backend runtime and binary output paths target-scoped
- [ ] Slice 5: pass `--target` to rustc for runtime builds
- [ ] Slice 6: pass `--target` to rustc for generated crate builds
- [ ] Slice 7: reject non-host execution in `run` and `test` with clear diagnostics
- [ ] Slice 8: add backend and frontend tests for target layout and diagnostics
- [ ] Slice 9: document cross-compilation behavior in `book/`

## Phase 1: Define target ownership

Goal:
Make target selection a first-class backend input instead of an ignored build
option.

Tasks:

- add a machine-target field to backend config
- define a host/default representation
- define a normalization API for target inputs
- keep current host behavior unchanged when no target is specified

Expected result:

- backend APIs can carry target decisions explicitly

## Phase 2: Thread target selection through the frontend

Goal:
Carry target intent from CLI/build graph selection into backend invocation.

Tasks:

- inspect frontend command config sources for target values
- define precedence between CLI target and artifact target
- pass the chosen target into backend config
- keep `emit rust` target-agnostic unless future requirements change

Expected result:

- backend receives the target that the user actually requested

## Phase 3: Normalize target names for rustc

Goal:
Ensure the backend always knows which Rust target triple to hand to `rustc`.

Tasks:

- define accepted FOL-facing target spellings
- define mapping to canonical Rust target triples
- reject unsupported or ambiguous target names with clear diagnostics
- keep this logic centralized and tested

Expected result:

- `rustc` sees one normalized target triple

## Phase 4: Partition outputs by target

Goal:
Prevent artifact collisions between host and cross builds.

Tasks:

- update runtime artifact output paths to include target
- update final binary output paths to include target
- ensure build summaries and artifact reports surface the target-aware paths
- keep host-only builds easy to discover in the resulting layout

Expected result:

- host and cross builds can coexist safely

## Phase 5: Compile runtime support for the selected target

Goal:
Build `fol-runtime` for the same target as the final program.

Tasks:

- pass `--target <triple>` to the runtime rustc invocation
- make the runtime output path target-aware
- preserve profile behavior
- keep diagnostics clear when the target toolchain is unavailable

Expected result:

- runtime artifacts match the selected target

## Phase 6: Compile the generated crate for the selected target

Goal:
Build the generated entry crate for the selected target.

Tasks:

- pass `--target <triple>` to the generated crate rustc invocation
- use the target-matched runtime `.rlib`
- keep deterministic crate naming and output naming
- preserve host behavior when no target is specified

Expected result:

- the produced executable matches the selected target

## Phase 7: Define execution policy for non-host binaries

Goal:
Avoid pretending cross-built binaries are runnable on the current machine.

Tasks:

- detect when the selected target is not the current host target
- reject `run` with a clear message for non-host targets
- reject `test` with a clear message for non-host targets
- keep host-target execution behavior unchanged

Expected result:

- users get a correct diagnostic instead of an opaque OS execution failure

## Phase 8: Test the target-aware backend

Goal:
Prove that cross-compilation behavior is deliberate and stable.

Tasks:

- add unit tests for target normalization and mapping
- add backend tests for target-aware output layout
- add backend tests that rustc command construction includes target selection
- add frontend tests for target precedence and non-host run/test diagnostics
- keep real cross-toolchain requirements out of the default suite where
  possible

Expected result:

- the design is enforced by tests, not by convention

## Phase 9: Document the model

Goal:
Explain clearly how cross-compilation works for users and maintainers.

Tasks:

- update `book/` build documentation with target examples
- document the distinction between build and run for non-host targets
- document accepted target spellings and any mapping rules
- document output layout expectations

Expected result:

- the feature is usable without reading backend code

## Testing strategy

The first milestone should not require every development machine or CI worker
to have multiple cross toolchains installed.

Test layers should be:

1. pure unit tests
   - target parsing
   - target mapping
   - host-vs-non-host checks

2. backend integration tests
   - output layout
   - target-aware diagnostics
   - rustc command construction behavior where practical

3. frontend integration tests
   - CLI target precedence
   - non-host run/test rejection
   - build artifact summaries

4. optional environment-gated real cross-build tests later
   - only when a target toolchain is present

## Known risks

### Risk 1: FOL target names do not match Rust target triples

The build layer already uses target strings that may not be valid Rust triples.

Mitigation:

- centralize mapping logic
- reject unknown spellings explicitly
- keep tests for all accepted aliases

### Risk 2: Missing target toolchains

`rustc` may accept `--target`, but the standard library or linker for that
target may not be installed.

Mitigation:

- surface rustc/linker failures with the selected target in diagnostics
- avoid hard dependency on cross toolchains in the default test suite

### Risk 3: Output collisions

Without target-aware paths, repeated builds for different targets will overwrite
runtime artifacts or final binaries.

Mitigation:

- make target part of runtime and binary output layout from the start

### Risk 4: Confused run/test semantics

Users may assume that a successful cross-build implies local execution.

Mitigation:

- reject non-host run/test explicitly
- document the behavior in CLI and book docs

### Risk 5: Accidental backend coupling

Target logic could spread across frontend, build graph, and backend in ad hoc
ways.

Mitigation:

- centralize target normalization and rustc triple selection
- keep frontend responsible for choosing the target, backend responsible for
  compiling it

## Definition of done

This plan is complete when:

- target selection reaches the backend intentionally
- rustc runtime and entry-crate builds both honor the selected target
- output layout is target-safe
- host builds remain stable
- non-host run/test behavior is explicit and correct
- tests cover the core target model
- the book explains how to use the feature

LLVM remains separate future work and is not part of this plan.
