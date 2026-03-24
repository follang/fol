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

- runtime crate split
- backend linking by runtime tier
- runtime code movement into `core` / `alloc` / `std`
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

## Epoch 2: Create Runtime Crate Skeletons

Goal:
Introduce the physical runtime split without yet moving every implementation.

### Slice Tracker

- [x] Slice 4. Add new crates:
  - `lang/execution/fol-core`
  - `lang/execution/fol-alloc`
  - `lang/execution/fol-std`
  with minimal `Cargo.toml` and `lib.rs` surfaces.
- [ ] Slice 5. Wire the workspace manifests so the new crates build in the
  workspace and are visible to the backend build path.
- [ ] Slice 6. Define crate dependency direction and enforce it in code:
  - `fol-core` depends on nothing from `fol-alloc` or `fol-std`
  - `fol-alloc` may depend on `fol-core`
  - `fol-std` may depend on `fol-core` and `fol-alloc`
- [ ] Slice 7. Add smoke tests proving the new crates compile and the workspace
  build remains green.

### Exit criteria

- The three runtime crates exist and build.
- The dependency direction is explicit and tested.

## Epoch 3: Backend Learns Runtime Tier Linking

Goal:
Make backend emission reflect `fol_model` structurally, not only semantically.

### Slice Tracker

- [ ] Slice 8. Add backend crate-set selection by `BackendFolModel` with a small
  internal abstraction such as `BackendRuntimeTier` or `BackendRuntimeCrateSet`.
- [ ] Slice 9. Update emitted Rust crate generation so `core`, `alloc`, and
  `std` artifacts link different runtime crates.
- [ ] Slice 10. Add backend trace metadata tests to prove emitted artifacts
  record the selected runtime crate set.
- [ ] Slice 11. Add frontend integration tests proving:
  - `fol_model = core` emits against `fol-core`
  - `fol_model = alloc` emits against `fol-core + fol-alloc`
  - `fol_model = std` emits against `fol-core + fol-alloc + fol-std`

### Exit criteria

- Backend linkage differs by model.
- This is visible in emitted source or trace metadata and locked by tests.

## Epoch 4: Move Process And Console Services Into `std`

Goal:
Remove hosted process/runtime assumptions from the shared runtime surface.

### Slice Tracker

- [ ] Slice 12. Move `.echo(...)` implementation ownership into `fol-std`.
- [ ] Slice 13. Move process outcome and executable entry helpers into
  `fol-std`.
- [ ] Slice 14. Make backend-generated `std` artifacts use `fol-std` for main
  entry and hosted execution support.
- [ ] Slice 15. Remove the old shared runtime ownership for those services
  instead of keeping fallback exports.
- [ ] Slice 16. Add backend and app-level example tests for hosted `std`
  execution after the move.

### Exit criteria

- Console and process behavior live in `std`.
- No shared fallback path remains for those services.

## Epoch 5: Move Heap Types Into `alloc`

Goal:
Make heap-backed runtime data structures physically belong to `alloc`.

### Slice Tracker

- [ ] Slice 17. Move string runtime support into `fol-alloc`.
- [ ] Slice 18. Move `vec` and `seq` runtime support into `fol-alloc`.
- [ ] Slice 19. Move `set` and `map` runtime support into `fol-alloc`, or, if
  one of them is not yet stable enough, explicitly defer it in the docs and
  keep the plan honest.
- [ ] Slice 20. Update backend emission so `alloc` and `std` artifacts import
  those types from `fol-alloc`.
- [ ] Slice 21. Delete the old monolithic ownership of those heap-backed types.
- [ ] Slice 22. Add end-to-end fixtures for:
  - `alloc` artifact using `str`
  - `alloc` artifact using `seq`
  - `std` artifact using `str` + `.echo(...)`
  - `core` artifact still rejecting the same surfaces

### Exit criteria

- Heap-backed runtime types physically live in `alloc`.
- Backend emission for `alloc` and `std` points to `fol-alloc`.

## Epoch 6: Establish `core` As A Real No-Heap Tier

Goal:
Make `core` useful and honest for embedded-first work.

### Slice Tracker

- [ ] Slice 23. Audit the backend-emitted `core` crate root and remove accidental
  imports of hosted or heap-backed support.
- [ ] Slice 24. Add explicit backend tests that `core` artifacts emit without
  `fol-alloc` or `fol-std`.
- [ ] Slice 25. Add example artifacts for `core` that use only:
  - scalars
  - arrays
  - records
  - control flow
  - `defer`
- [ ] Slice 26. Add negative example fixtures for forbidden `core` surfaces:
  - `str`
  - `seq`
  - `vec`
  - `set`
  - `map`
  - `.echo(...)`
- [ ] Slice 27. Document the exact current embedded meaning of `core`:
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

- [ ] Slice 28. Improve frontend summaries and emitted metadata so build output
  shows the selected `fol_model`.
- [ ] Slice 29. Add build-route tests for mixed-model workspaces:
  - `core` static lib
  - `alloc` helper lib
  - `std` host tool
- [ ] Slice 30. Add scaffold support or examples that generate clear
  `fol_model` usage in `build.fol`.
- [ ] Slice 31. Add validation diagnostics for inconsistent build intent where
  relevant, for example if a route expects host execution but the artifact model
  is non-`std`.

### Exit criteria

- The build UX makes model selection obvious.
- Mixed-model workspaces are tested.

## Epoch 8: Remove The Old Monolithic Runtime Surface

Goal:
Finish the transition instead of keeping parallel ownership.

### Slice Tracker

- [ ] Slice 32. Delete or radically shrink the old `fol-runtime` crate so it no
  longer owns the split runtime surfaces.
- [ ] Slice 33. Remove old backend references to the monolithic runtime path.
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
- the old monolithic runtime path is gone
- docs, examples, tests, and backend output all match that reality
