# PLAN: Runtime Split for `fol-model`

Last updated: 2026-03-24

## Intent

This plan replaces the current compiler-only `fol_model` enforcement with a
real architecture split:

- `core`
  no heap, no OS
- `alloc`
  heap yes, OS no
- `std`
  heap yes, OS yes

The end state is not:

- one large runtime with conditional checks
- frontend-only policy
- backend-only policy
- import-name conventions

The end state is:

- `build.fol` chooses `fol_model` per artifact
- typecheck enforces the language surface for that artifact
- backend links the correct runtime crate set
- the runtime implementation is physically split
- docs match the real behavior

## Rules

- Every slice must stay commit-sized.
- Every slice that changes behavior must include tests in the same commit.
- After each slice:
  - run `make build`
  - run `make test`
  - if both pass:
    - mark the slice complete here
    - commit it
- No compatibility layer for the old monolithic runtime model once the new path
  is chosen for a subsystem.

## Current baseline

Already complete before this plan:

- `build.fol` accepts `fol_model`
- frontend carries `fol_model`
- typecheck gates:
  - `.echo(...)` behind `std`
  - heap-backed type surfaces out of `core`
  - dynamic `.len(...)` out of `core`
- routed `run` / `test` reject non-`std` execution

Not complete:

- single-crate runtime model ownership inside `fol-runtime`
- backend linking by runtime tier
- runtime code movement into `core` / `alloc` / `std` modules inside `fol-runtime`
- crate-level ownership of string/container/runtime/process services
- docs for the full split

## Epoch 1: Freeze The Model Contract

Goal:
Lock down the contract before moving code.

### Slice Tracker

- [x] Slice 1. Rewrite the book and version docs so `core`, `alloc`, and `std`
  are described as runtime tiers, and explicitly state:
  - `str` is not `core`
  - dynamic containers are not `core`
  - `.echo(...)` is `std`
  - process-entry behavior is `std`
- [x] Slice 2. Add a single canonical feature matrix document, probably
  `docs/runtime-models.md`, that maps language features, intrinsics, and runtime
  services into `core`, `alloc`, and `std`.
- [x] Slice 3. Add tests that lock the intended language boundary text into CLI
  and structured diagnostics for:
  - `str` in `core`
  - `.echo(...)` in `alloc`
  - dynamic `.len(...)` in `core`

### Exit criteria

- The intended split is documented in one place and referenced by the book.
- Diagnostics reflect the language model consistently.

## Epoch 2: Turn `fol-runtime` Into The Model Crate

Goal:
Keep one runtime crate and make the model ownership explicit inside it.

### Slice Tracker

- [x] Slice 4. Remove the abandoned multi-crate split and keep `fol-runtime` as
  the only runtime crate.
- [x] Slice 5. Add explicit `core`, `alloc`, and `std` module boundaries inside
  `fol-runtime`, with minimal marker APIs and unit tests.
- [x] Slice 6. Define dependency direction inside `fol-runtime` modules and
  enforce it in code:
  - `core` module owns the no-heap, no-OS base
  - `alloc` module may build on `core`
  - `std` module may build on `core` and `alloc`
- [x] Slice 7. Add smoke tests proving `fol-runtime` exposes the model modules
  and the workspace build remains green.

### Exit criteria

- `fol-runtime` is the single model crate.
- The internal module direction is explicit and tested.

## Epoch 3: Backend Learns Runtime Tier Linking

Goal:
Make backend emission reflect `fol_model` structurally, not only semantically.

### Slice Tracker

- [x] Slice 8. Add backend runtime-tier selection by `BackendFolModel` with a
  small internal abstraction such as `BackendRuntimeTier`.
- [x] Slice 9. Update emitted Rust crate generation so `core`, `alloc`, and
  `std` artifacts import different `fol-runtime` model modules while still
  linking one runtime crate.
- [x] Slice 10. Add backend trace metadata tests to prove emitted artifacts
  record the selected runtime tier and emitted runtime module surface.
- [x] Slice 11. Add frontend integration tests proving:
  - `fol_model = core` emits against `fol-runtime::core`
  - `fol_model = alloc` emits against `fol-runtime::alloc`
  - `fol_model = std` emits against `fol-runtime::std`

### Exit criteria

- Backend-emitted runtime usage differs by model.
- This is visible in emitted source or trace metadata and locked by tests.

## Epoch 4: Move Process And Console Services Into `std`

Goal:
Remove hosted process/runtime assumptions from the shared runtime surface.

### Slice Tracker

- [x] Slice 12. Move `.echo(...)` implementation ownership into the `std`
  module inside `fol-runtime`.
- [x] Slice 13. Move process outcome and executable entry helpers into
- the `std` module inside `fol-runtime`.
- [x] Slice 14. Make backend-generated `std` artifacts use `fol-runtime::std` for main
  entry and hosted execution support.
- [x] Slice 15. Remove the old shared runtime ownership for those services
  instead of keeping fallback exports.
- [x] Slice 16. Add backend and app-level example tests for hosted `std`
  execution after the move.

### Exit criteria

- Console and process behavior live in `fol-runtime::std`.
- No shared fallback path remains for those services.

## Epoch 5: Move Heap Types Into `alloc`

Goal:
Make heap-backed runtime data structures physically belong to `alloc`.

### Slice Tracker

- [x] Slice 17. Move string runtime support into `fol-runtime::alloc`.
- [x] Slice 18. Move `vec` and `seq` runtime support into `fol-runtime::alloc`.
- [x] Slice 19. Move `set` and `map` runtime support into `fol-runtime::alloc`, or, if
  one of them is not yet stable enough, explicitly defer it in the docs and
  keep the plan honest.
- [x] Slice 20. Update backend emission so `alloc` and `std` artifacts import
  those types from the `alloc` module in `fol-runtime`.
- [x] Slice 21. Delete the old unsplit ownership path for those heap-backed
  types inside `fol-runtime`.
- [x] Slice 22. Add end-to-end fixtures for:
  - `alloc` artifact using `str`
  - `alloc` artifact using `seq`
  - `std` artifact using `str` + `.echo(...)`
  - `core` artifact still rejecting the same surfaces

### Exit criteria

- Heap-backed runtime types physically live in `fol-runtime::alloc`.
- Backend emission for `alloc` and `std` points to that module.

## Epoch 6: Establish `core` As A Real No-Heap Tier

Goal:
Make `core` useful and honest for embedded-first work.

### Slice Tracker

- [x] Slice 23. Audit the backend-emitted `core` crate root and remove accidental
  imports of hosted or heap-backed support.
- [x] Slice 24. Add explicit backend tests that `core` artifacts emit without
  importing alloc/std runtime modules.
- [x] Slice 25. Add example artifacts for `core` that use only:
  - scalars
  - arrays
  - records
  - control flow
  - `defer`
- [x] Slice 26. Add negative example fixtures for forbidden `core` surfaces:
  - `str`
  - `seq`
  - `vec`
  - `set`
  - `map`
  - `.echo(...)`
- [x] Slice 27. Document the exact current embedded meaning of `core`:
  “no heap and no OS at language/runtime level, still emitted through the
  current Rust backend pipeline.”

### Exit criteria

- `core` has a tested positive surface.
- `core` has a tested negative surface.
- Docs do not overclaim embedded backend maturity.

## Epoch 7: Tighten Frontend And Build-System UX

Goal:
Make the model visible and obvious at the artifact/build level.

### Slice Tracker

- [x] Slice 28. Improve frontend summaries and emitted metadata so build output
  shows the selected `fol_model`.
- [x] Slice 29. Add build-route tests for mixed-model workspaces:
  - `core` static lib
  - `alloc` helper lib
  - `std` host tool
- [x] Slice 30. Add scaffold support or examples that generate clear
  `fol_model` usage in `build.fol`.
- [x] Slice 31. Add validation diagnostics for inconsistent build intent where
  relevant, for example if a route expects host execution but the artifact model
  is non-`std`.

### Exit criteria

- The build UX makes model selection obvious.
- Mixed-model workspaces are tested.

## Epoch 8: Remove The Old Unsplit Runtime Surface

Goal:
Finish the transition instead of keeping parallel ownership.

### Slice Tracker

- [x] Slice 32. Delete or radically shrink the old unsplit `fol-runtime` surface
  so the crate becomes the model crate rather than a monolithic dump.
- [x] Slice 33. Remove old backend references to the unsplit runtime path.
- [ ] Slice 34. Remove stale tests that assume one hosted runtime surface.
- [ ] Slice 35. Add a final regression pass across backend emission, frontend
  routing, example apps, and docs.

### Exit criteria

- There is no parallel runtime implementation path left.
- Runtime ownership is unambiguous.

## Epoch 9: Post-Split Hardening

Goal:
Prove the split holds under normal project use.

### Slice Tracker

- [ ] Slice 36. Add full example packages:
  - `examples/core-blink-shape`
  - `examples/alloc-containers`
  - `examples/std-cli`
- [ ] Slice 37. Add CLI integration tests compiling and emitting each example.
- [ ] Slice 38. Add one workspace example mixing all three models in one build
  graph.
- [ ] Slice 39. Add developer docs for how to choose a model and what each tier
  guarantees.
- [ ] Slice 40. Do a final language/docs audit so no chapter still implies the
  old unsplit hosted runtime story.

### Exit criteria

- Users can see and run concrete examples for each model.
- Documentation no longer conflicts with implementation.

## Recommended execution order

Do not reorder casually.

Recommended order:

1. Epoch 1
2. Epoch 2
3. Epoch 3
4. Epoch 4
5. Epoch 5
6. Epoch 6
7. Epoch 7
8. Epoch 8
9. Epoch 9

This order matters because:

- docs and contract should settle before moving code
- backend linkage must exist before runtime movement is safe
- hosted services should move before deleting the old runtime
- `core` must be tested as a real tier before cleanup is declared done

## High-risk points

These are the places most likely to break during implementation:

- backend emitted Rust import paths
- executable entrypoint ownership
- string/container runtime assumptions hidden in lowering or backend helpers
- tests that assume all runnable artifacts are `std`
- accidental parallel exports from both old and new runtime crates

## Definition of done

This plan is only done when all of the following are true:

- `fol_model` changes both semantics and linked runtime structure
- `core` artifacts do not pull heap or OS runtime code
- `alloc` artifacts can use heap-backed types without `std`
- `std` artifacts own hosted execution/runtime services
- the old unsplit runtime path is gone
- docs, examples, tests, and backend output all match that reality
