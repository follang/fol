# PLAN: `fol-model` + `core` / `alloc` / `std`

Last updated: 2026-03-24

## Slice Tracker

- [x] Slice 1. Land the documentation checkpoint for `fol-model`, `core`,
  `alloc`, and `std`, and rewrite this plan as a commit-by-commit tracker.
- [x] Slice 2. Add a shared `BuildArtifactFolModel` enum and artifact target
  config support in `fol-build`, with unit tests.
- [x] Slice 3. Extend build semantic artifact config shapes so `fol_model` is a
  first-class artifact field, with semantic tests.
- [ ] Slice 4. Parse `fol_model` from `build.fol` artifact records in the build
  executor and carry it through evaluated build programs.
- [ ] Slice 5. Add positive build-source tests for `fol_model` on executable,
  library, and test artifacts.
- [ ] Slice 6. Add explicit invalid-`fol_model` diagnostics in build
  evaluation, with negative tests.
- [ ] Slice 7. Carry `fol_model` into frontend/backend compile configuration and
  emitted trace metadata.
- [ ] Slice 8. Add the initial semantic capability model in the compiler and
  wire artifact `fol_model` into it.
- [ ] Slice 9. Gate std-only intrinsics first: move `.echo(...)` behind
  `fol-model = std`, with diagnostics and tests.
- [ ] Slice 10. Gate heap-backed families first: reject `str`, `vec`, `seq`,
  `set`, and `map` in `core`, with diagnostics and tests.

## Purpose

This plan proposes a fundamental runtime and build-model split for FOL:

- `core`
  no heap, no OS
- `alloc`
  adds heap-backed facilities, still no OS
- `std`
  adds OS/runtime services on top of `alloc`

The compiler-facing control point should be a build-artifact setting in
`build.fol`, using the concept name:

- `fol-model`

This is not a parser-only change.
It requires coordinated work across:

- `build.fol` semantic API
- package/build graph IR
- frontend compile config
- typecheck capability gating
- lowering/runtime contracts
- backend crate emission/linking
- book/version docs

## Core decision

### `fol-model` belongs in `build.fol`

The selected model is an artifact capability contract, not a local source-file
preference.

It should live beside:

- `target`
- `optimize`
- artifact kind
- build graph linkage

It should not be modeled as:

- a source-file pragma
- a package import convention alone
- a backend-only switch

The intended shape is per artifact, not only per package:

```fol
pro[] build(graph: Graph): non = {
    graph.add_static_lib({
        name = "corelib",
        root = "src/lib.fol",
        fol_model = "core",
        target = "thumbv7em-none-eabi",
    });

    graph.add_exe({
        name = "tool",
        root = "tools/main.fol",
        fol_model = "std",
    });
}
```

## Main architectural rule

The model must be enforced semantically, not only linked structurally.

That means:

- the build graph records the selected model
- the frontend passes it into semantic compilation
- typecheck rejects surfaces unavailable in that model
- lowering/backend only see already-valid model-constrained programs
- backend links only the support crates required by that model

If the compiler only changes the linked runtime crate set but still accepts
forbidden source surfaces, the split is incomplete.

## Desired model contract

### `core`

Allowed:

- builtin scalars: `int`, `flt`, `bol`, `chr`
- fixed arrays: `arr[...]`
- aliases, records, entries
- routines, methods-as-sugar, calls
- control flow
- `defer`
- `opt[...]` and `err[...]`
- recoverable routine ABI shape
- scalar intrinsics:
  - `.eq`, `.nq`, `.lt`, `.gt`, `.ge`, `.le`
  - `.not`
- `panic` as language control flow

Forbidden:

- heap-backed string storage
- `vec[...]`
- `seq[...]`
- `set[...]`
- `map[...]`
- OS/process/console services
- `.echo(...)`
- hosted process-entry assumptions

### `alloc`

Adds:

- heap-backed `str`
- `vec[...]`
- `seq[...]`
- likely `set[...]`
- likely `map[...]`
- dynamic-container/string `.len(...)`
- heap-aware formatting helpers that do not require OS

Still forbidden:

- console output
- filesystem
- networking
- host process assumptions

### `std`

Adds:

- console I/O
- process outcome conventions
- host/runtime integration
- future filesystem/network/time services
- `.echo(...)`

## Current repo reality

This repository already has the right high-level seam:

- compiler crates
- intrinsic registry
- runtime crate
- backend crate

Relevant current boundaries:

- runtime support is centralized in `lang/execution/fol-runtime`
- backend codegen is centralized in `lang/execution/fol-backend`
- `std` is currently only a resolver/package-root concept
- the backend currently always emits hosted Rust artifacts

Current constraints that matter:

1. `FolStr` is `String`
2. `FolSeq` and `FolVec` are `Vec<T>`
3. `.echo(...)` uses `println!`
4. process-entry policy lives in the runtime
5. the current build/compile flow does not carry a runtime capability tier

Therefore the current implementation is effectively:

- one hosted heap-capable runtime
- one hosted executable backend

## Non-goals

This plan does not attempt to solve immediately:

- ownership/borrowing
- pointers
- async or concurrency runtime
- alternate non-Rust backends
- embedded linker scripts or HAL integration
- full `no_std` Rust backend emission details

Those can follow after the model split exists.

## Naming decision

### Concept name

Use:

- `fol-model`

### Build field spelling

Prefer the existing build-record key style that best matches the build parser and
build semantic API.

Candidate spellings:

- `fol_model`
- `fol-model`

Implementation should choose the spelling that best matches current build field
rules, but the concept should remain “FOL model”.

## Language surface classification

### Move to `core`

- scalar operators and comparisons
- aliases
- records and entries
- methods as receiver sugar
- arrays
- option/error shells
- recoverable-call control flow
- `defer`

### Move to `alloc`

- `str`
- `vec`
- `seq`
- `set`
- `map`
- dynamic `.len(...)`

### Move to `std`

- `.echo(...)`
- process-exit mapping
- host-side runtime hooks
- future stdlib import content that requires OS/runtime services

## Why `str` should be `alloc`, not `core`

Given the current implementation strategy, `str` is inherently heap-backed.

Trying to keep `str` in `core` would force one of these:

- a fake no-heap `core` that still depends on heap-backed strings
- an immediate redesign toward borrowed/fixed string semantics
- early ownership/lifetime complexity

The clean choice is:

- `str` belongs to `alloc`

That keeps `core` honestly heap-free.

## Why `.echo(...)` should be `std`, not `core`

Today `.echo(...)` is runtime-backed and implemented with `println!`.

That means it currently implies:

- output device
- process/host environment
- formatting allocation in practice

So `.echo(...)` should be treated as:

- `std`

not as a universal core intrinsic.

## Why `panic` can remain in `core`

`panic` is language control flow first.

Its backend behavior may differ by model:

- `std`: printable host panic/failure path
- `core`: trap/halt/backend-defined abort path without OS assumptions

So `panic` may remain in `core`, but its lowering/backend contract must stop
assuming hosted process behavior.

## Required architecture changes

## Phase 0: Documentation and invariants

Goal:

- define the split before changing code

Work:

- update book sections on intrinsics/runtime/containers
- update versioning docs to state that `core/alloc/std` is a build/runtime
  capability model
- explicitly document:
  - `str` is `alloc`
  - `.echo(...)` is `std`
  - arrays/scalars/records/routines are `core`

Acceptance:

- docs no longer describe the current runtime as a single undifferentiated
  support layer
- docs no longer imply that heap-backed and hosted facilities are always
  available

## Phase 1: Build graph model field

Goal:

- make `fol-model` a real artifact property in build evaluation

Work:

- extend build semantic shapes for artifact config records
- add `fol-model` validation
- accepted values:
  - `core`
  - `alloc`
  - `std`
- artifact IR must carry the chosen model
- default decision must be explicit:
  choose either:
  - default `std` for now
  - or require explicit model in early rollout

Decision:

- recommended temporary default: `std`
  because it preserves current behavior while the split is staged

Acceptance:

- `graph.add_exe/add_static_lib/...` can record a model
- diagnostics for invalid model values are explicit

## Phase 2: Frontend and backend config plumbing

Goal:

- pass selected model through the compile pipeline

Work:

- frontend compile requests must carry artifact model
- lowering/backend sessions must know the selected model
- emitted backend traces should record the model

Acceptance:

- backend build artifacts know which model they are compiling
- model is visible in debug/trace output

## Phase 3: Typecheck capability gating

Goal:

- enforce model boundaries before codegen

Work:

- introduce a semantic capability context:
  - heap allowed?
  - OS/std allowed?
- gate type families:
  - `str`, `vec`, `seq`, `set`, `map`
- gate intrinsics:
  - `.echo`
  - dynamic `.len`
- gate std imports if necessary

Diagnostics must say both:

- what was used
- why the current `fol-model` forbids it

Examples:

- “`str` requires `fol-model = alloc` or `std`”
- “`.echo(...)` requires `fol-model = std`”

Acceptance:

- `core` artifacts reject heap/std surfaces during typecheck
- `alloc` artifacts reject std-only surfaces during typecheck

## Phase 4: Runtime crate split

Goal:

- reflect model boundaries in code layout and linkage

Work:

- split `fol-runtime` into:
  - `fol-core`
  - `fol-alloc`
  - `fol-std`

Recommended dependency direction:

- `fol-core`
  no dependency on `alloc` or `std`
- `fol-alloc`
  depends on `fol-core`
- `fol-std`
  depends on `fol-core` and `fol-alloc`

Likely placement:

- scalar/value/shell/recoverable ABI in `fol-core`
- `FolStr`, `FolSeq`, `FolVec`, `FolSet`, `FolMap` in `fol-alloc`
- `.echo`, process outcome helpers, hosted hooks in `fol-std`

Acceptance:

- no hosted console/process helpers remain in `fol-core`
- no heap-backed data structures remain in `fol-core`

## Phase 5: Backend emission by model

Goal:

- link only the support crates required by the selected model

Work:

- `core` artifacts import/link `fol-core`
- `alloc` artifacts import/link `fol-core` + `fol-alloc`
- `std` artifacts import/link `fol-core` + `fol-alloc` + `fol-std`

This may require:

- backend prelude split
- generated imports changing by model
- crate skeleton generation changing by model

Acceptance:

- emitted crates do not depend on hosted std support when building `core`
- emitted crates do not pull heap support when building `core`

## Phase 6: Runtime hook redesign

Goal:

- remove hidden hosted assumptions from core execution

Work:

- decouple `panic` backend strategy from process/std policy
- move process outcome helpers fully into `std`
- make `core` entry/library emission possible without process-exit semantics

Acceptance:

- library-style or freestanding emission does not require hosted process helpers

## Phase 7: Import/root model alignment

Goal:

- make package loading consistent with capability tiers

Work:

- decide whether `core`, `alloc`, and `std` are:
  - implicit compiler-provided libraries
  - canonical package roots
  - or a mixture

Recommended direction:

- `core`, `alloc`, and `std` should be standard library tiers selected by model,
  not user-installed `pkg` packages

Compiler behavior:

- `core` available everywhere
- `alloc` available only for `alloc/std`
- `std` available only for `std`

Acceptance:

- import resolution and model gating agree

## Phase 8: Embedded-first backend mode

Goal:

- make `core` artifacts meaningful for embedded/no-heap bring-up

Initial target:

- compile `core` libraries and simple `core` executables without heap/std usage

This phase should come after the semantic/runtime split, not before it.

Acceptance:

- a small `core` artifact can compile without `fol-alloc` or `fol-std`
- backend output does not include hosted runtime dependencies for `core`

## Required design decisions before implementation

### 1. Default model

Options:

- default to `std`
- require explicit `fol-model`

Recommendation:

- short term: default to `std`
- later: consider requiring explicit `fol-model` for embedded-oriented targets

### 2. `set` and `map`

Question:

- do they belong in `alloc`?

Recommendation:

- yes

Reason:

- current implementation is dynamic/runtime-backed
- no clear no-heap representation exists in the current design

### 3. `str`

Question:

- should `str` remain core?

Recommendation:

- no
- `str` belongs in `alloc`

### 4. `.len(...)`

Split:

- array `len` can remain `core`
- string/dynamic-container `len` requires `alloc`

This likely means the intrinsic remains registry-owned but its admissible
operand families depend on model.

### 5. Source-level override

Recommendation:

- do not add a source-file `#![no_std]` equivalent
- keep the contract in `build.fol`

## Risks

### Risk 1: semantic/runtime mismatch

If typecheck gates too little, backend/link failures will become the first place
the model is enforced.

That is unacceptable.

Mitigation:

- capability gating must happen in typecheck

### Risk 2: fake `core`

If `core` still pulls `String`, `Vec`, or `println!` indirectly, the split is
not real.

Mitigation:

- hard crate separation
- backend linkage tests per model

### Risk 3: docs drift

The book already mixes language intent with current runtime assumptions.

Mitigation:

- document the tier split before large code churn

### Risk 4: too much at once

This touches build, semantic analysis, runtime, and backend.

Mitigation:

- land in phases
- keep `std` as the temporary default while the lower tiers are brought up

## Suggested implementation order

1. docs and version docs
2. build graph `fol-model`
3. compiler config plumbing
4. typecheck gating
5. runtime crate split
6. backend linkage/import split
7. import/root model cleanup
8. embedded-first examples/tests

## Acceptance criteria for the whole project

This work is complete only when all of these are true:

- `build.fol` can declare a `fol-model` per artifact
- the chosen model reaches typecheck and backend
- `core` artifacts reject heap/std-only language surfaces
- `alloc` artifacts reject std-only surfaces
- runtime support is physically split into `fol-core`, `fol-alloc`, `fol-std`
- backend artifacts link only the required support crates
- docs state clearly which surfaces belong to which model
- at least one test-backed `core` artifact builds without heap/std support

## Recommended immediate next step after review

After review, start with a design-only slice:

1. add the book/version documentation for the model split
2. add `fol-model` to build semantic config/IR only
3. do not yet split runtime crates in the same slice

That gives one clean checkpoint before deeper semantic and runtime changes.
