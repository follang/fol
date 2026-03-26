# PLAN

## Goal

Harden the `core` / `alloc` / `std` runtime-model split so it behaves like a real semantic and runtime boundary across:

- build evaluation
- package/session loading
- typecheck
- lowering/backend emission
- routed frontend execution
- editor/LSP
- examples, docs, and negative fixtures

This plan is about hardening, not adding a fourth mode or expanding the language surface.

## Current Scan Summary

The scan shows the model split is already implemented and documented, but the remaining risk is mostly in boundary enforcement and regression coverage.

High-signal existing enforcement already present:

- typecheck capability gates in:
  - `lang/compiler/fol-typecheck/src/decls.rs`
  - `lang/compiler/fol-typecheck/src/exprs/calls.rs`
  - `test/typecheck/test_typecheck_containers_and_shells.rs`
- routed workspace execution gates in:
  - `lang/tooling/fol-frontend/src/build_route/mod.rs`
  - `lang/tooling/fol-frontend/src/build_route/tests/exec.rs`
- backend runtime-tier selection in:
  - `lang/execution/fol-backend/src/config.rs`
  - `lang/execution/fol-backend/src/emit/tests.rs`
- runtime module boundaries in:
  - `lang/execution/fol-runtime/src/core.rs`
  - `lang/execution/fol-runtime/src/alloc.rs`
  - `lang/execution/fol-runtime/src/std.rs`
- editor model-aware semantics in:
  - `lang/tooling/fol-editor/src/lsp/semantic.rs`
  - `lang/tooling/fol-editor/src/lsp/tests/lifecycle.rs`
  - `lang/tooling/fol-editor/src/lsp/tests/completion.rs`
  - `lang/tooling/fol-editor/src/lsp/tests/completion_namespaced.rs`
- docs/examples baseline in:
  - `docs/runtime-models.md`
  - `examples/core_*`
  - `examples/alloc_*`
  - `examples/std_*`
  - `examples/mixed_models_workspace`

Remaining weak spots from the scan:

- transitive dependency model-boundary coverage is still thinner than direct-package coverage
- mixed-workspace route selection coverage is stronger for success paths than for subtle failure paths
- emitted Rust/runtime import auditing exists, but not for enough mixed dependency/model combinations
- example coverage is mostly positive-path; negative example packages for model misuse are still light
- editor model-awareness exists, but ambiguous package contexts and cross-artifact file mapping need stronger regression tests
- docs are mostly aligned, but they still need explicit “allowed / forbidden / indirect boundary” examples per tier

## Rules For This Plan

For each slice:

- add tests in the same commit as the fix/feature
- run `make build`
- then run `make test`
- only commit if both pass
- mark the slice complete in `PLAN.md` in the same commit

No compatibility paths. If a new stricter model rule is chosen, the old looser path is deleted.

---

## Epoch 1: Freeze The Runtime-Model Contract

### Slice 1
Status: complete

Audit `docs/runtime-models.md`, `book/src/055_build/200_graph_api.md`, and relevant book pages so they all say the same thing about:

- `core`
- `alloc`
- `std`
- `.echo(...)`
- dynamic `.len(...)`
- routed `run` / `test`

Completion criteria:

- docs use the same contract wording
- no doc still implies `std` is the baseline default in spirit

### Slice 2
Add a compiler-side test matrix that locks the canonical capability facts in one place.

Completion criteria:

- one test file or module checks the authoritative capability matrix for:
  - heap
  - OS/runtime
  - strings
  - dynamic containers
  - `.echo(...)`
  - array `.len(...)`
  - dynamic `.len(...)`

### Slice 3
Add explicit documentation examples for allowed/forbidden surfaces per tier.

Completion criteria:

- docs include one positive and one forbidden example for each of:
  - `core`
  - `alloc`
  - `std`

### Slice 4
Add a top-level integration test that asserts the docs’ example package list stays in sync with the actual example directories.

Completion criteria:

- if a listed model example is missing, tests fail
- if a package is renamed and docs drift, tests fail

---

## Epoch 2: Harden Direct Model Boundary Rejections

### Slice 5
Expand direct compile-fail fixtures for `core` rejecting heap-backed types.

Completion criteria:

- negative fixtures cover:
  - explicit `str`
  - inferred string literal
  - `vec`
  - `seq`
  - `set`
  - `map`
  - mixed declarations in one file

### Slice 6
Expand direct compile-fail fixtures for `core` rejecting heap-backed expressions, not just declarations.

Completion criteria:

- negative fixtures cover expression-level uses such as:
  - returning string literals
  - binding dynamic containers in routine bodies
  - dynamic `.len(...)`

### Slice 7
Expand direct compile-fail fixtures for `alloc` rejecting `.echo(...)` and hosted assumptions.

Completion criteria:

- negative fixtures cover:
  - `.echo(...)`
  - routed run/test refusal for alloc artifacts where relevant

### Slice 8
Add direct positive fixtures proving `core` still allows its intended minimal surface.

Completion criteria:

- positive examples cover:
  - arrays
  - records
  - entries
  - methods
  - `defer`
  - shells
  - array `.len(...)`

### Slice 9
Add direct positive fixtures proving `alloc` allows heap-only surfaces without accidentally requiring `std`.

Completion criteria:

- positive examples cover:
  - `str`
  - `seq`
  - `vec`
  - defaults/variadics if used there
  - dynamic `.len(...)`
  - no `.echo(...)`

### Slice 10
Add direct positive fixtures proving `std` remains minimal and honest.

Completion criteria:

- at least one `std` example is only `.echo(...)`
- at least one `std` example uses alloc-tier types plus hosted behavior

---

## Epoch 3: Transitive Dependency Model Boundaries

### Slice 11
Add tests where a `core` artifact depends on a `core` library and stays valid.

Completion criteria:

- success-path transitive `core -> core` example exists

### Slice 12
Add tests where a `core` artifact depends on an `alloc` library exporting heap-backed API and must fail.

Completion criteria:

- failure is caught before backend emission
- diagnostic points at the heap-backed boundary clearly

### Slice 13
Add tests where a `core` artifact depends on a `std` library or hosted-only export and must fail.

Completion criteria:

- diagnostic is explicit about `std`-only surface or hosted boundary

### Slice 14
Add tests where an `alloc` artifact depends on another `alloc` library and stays valid.

Completion criteria:

- success-path `alloc -> alloc` example exists

### Slice 15
Add tests where an `alloc` artifact indirectly reaches `.echo(...)` through a dependency and must fail.

Completion criteria:

- failure proves `.echo(...)` does not leak through imported packages

### Slice 16
Add tests where a `std` artifact consumes `core` and `alloc` dependencies in one graph and succeeds.

Completion criteria:

- positive mixed dependency graph exists and runs

### Slice 17
Add tests where transitive dependency exports preserve model identity in prepared package metadata.

Completion criteria:

- prepared package/build-route metadata exposes enough to assert transitive model facts in tests

### Slice 18
Document the transitive-boundary rule explicitly.

Completion criteria:

- docs say capability legality is checked at the consuming artifact boundary, not just where a dependency was compiled

---

## Epoch 4: Workspace Routing And Mixed-Artifact Hardening

### Slice 19
Add routed workspace tests for ambiguous mixed-model packages where `run` should fail because selection is not uniquely `std`.

Completion criteria:

- routed `run` diagnostics mention resolved models

### Slice 20
Add routed workspace tests for step-selected execution crossing model boundaries.

Completion criteria:

- selecting a non-`std` step for `run`/`test` fails with explicit step/model details

### Slice 21
Add routed build tests for mixed-model workspaces that include all three models and multiple packages.

Completion criteria:

- build summary exposes all three models stably

### Slice 22
Add routed check/build/test regression tests where package members have same root but different artifact models.

Completion criteria:

- selection logic stays correct for multi-artifact package roots

### Slice 23
Add frontend `work`/summary tests that surface model distribution across the workspace.

Completion criteria:

- `work` surfaces expose model information clearly enough for debugging

### Slice 24
Tighten routed diagnostics to distinguish:

- not runnable because model is wrong
- not runnable because selection is ambiguous
- not runnable because no entry exists

Completion criteria:

- regression tests pin all three diagnostic classes

---

## Epoch 5: Backend And Emitted Rust Audits

### Slice 25
Expand backend emission tests for direct `core`, `alloc`, and `std` crates to assert imports stay model-pure.

Completion criteria:

- `core` emitted modules import only `fol_runtime::core` plus non-tier support modules
- `alloc` emitted modules do not import `fol_runtime::std`

### Slice 26
Add mixed-workspace emitted-source tests where different artifacts in one workspace emit different runtime module imports.

Completion criteria:

- emitted source audit covers one workspace with all three models

### Slice 27
Add emitted-source tests for cross-package dependency consumption with different models.

Completion criteria:

- a `std` consumer of `alloc` still emits correct imports
- a `core` illegal consumer fails before emission

### Slice 28
Audit instruction rendering tests for hidden alloc/std leakage into core rendering paths.

Completion criteria:

- backend instruction tests explicitly assert no alloc/std helper names leak into core-only emission

### Slice 29
Add backend tests covering runtime helper selection for `.len(...)` and recoverable hooks by model.

Completion criteria:

- array `.len(...)` in `core` stays legal
- dynamic `.len(...)` path is only rendered for alloc/std-legal lowered programs

### Slice 30
Add a top-level regression test that walks emitted Rust trees and fails if model-forbidden runtime imports appear.

Completion criteria:

- helper test scans emitted files instead of only one hand-picked file

---

## Epoch 6: Runtime Module Boundary Hardening

### Slice 31
Audit `fol-runtime` public exports so `core`, `alloc`, and `std` module surfaces stay intentionally different.

Completion criteria:

- tests pin that `core` does not re-export heap types
- tests pin that `alloc` does not expose hosted hooks like `echo`

### Slice 32
Add runtime tests that assert tier capability flags and exported helper families stay aligned.

Completion criteria:

- tests cover `HAS_HEAP`, `HAS_OS`, module names, and allowed helper exports

### Slice 33
Add a no-accidental-reexport test for tier modules.

Completion criteria:

- if `alloc` or `std` symbols are accidentally re-exported through `core`, tests fail

### Slice 34
Document the runtime-tier export contract for backend authors.

Completion criteria:

- docs say what each runtime module may import/use and what it must not expose

---

## Epoch 7: Editor And LSP Model Hardening

### Slice 35
Add LSP diagnostics tests for transitive model-boundary failures in real workspaces, not only open-file direct errors.

Completion criteria:

- editor surfaces transitive boundary diagnostics from the real compiler pipeline

### Slice 36
Add LSP completion tests for ambiguous multi-artifact packages.

Completion criteria:

- completion is filtered correctly when a file belongs to:
  - one `core` artifact
  - one `alloc` artifact
  - one `std` artifact
  - ambiguous mixed package with no unique artifact mapping

### Slice 37
Add hover/semantic snapshot tests for mixed-model workspaces with routed files under different packages.

Completion criteria:

- active model tracking does not bleed across packages or artifacts

### Slice 38
Add tree-sitter/editor integration tests ensuring build-file model declarations stay discoverable and stable.

Completion criteria:

- editor tests cover `fol_model = "core" | "alloc" | "std"` in build files

### Slice 39
Add editor docs that explain model-aware diagnostics and completion behavior.

Completion criteria:

- `docs/editor-sync.md` and runtime-model docs agree on editor behavior

---

## Epoch 8: Example Packages And Negative Example Suites

### Slice 40
Add a standalone negative `core` misuse example package.

Completion criteria:

- package intentionally fails because of heap-backed surface
- integration harness asserts the exact failure class

### Slice 41
Add a standalone negative `alloc` misuse example package.

Completion criteria:

- package intentionally fails because of `.echo(...)` or hosted-only surface

### Slice 42
Add a standalone mixed dependency-boundary failure example.

Completion criteria:

- `core` consumer of `alloc` or `std` boundary is represented as a real package set

### Slice 43
Add one fuller positive example package per tier.

Completion criteria:

- `core` full example uses several allowed surfaces
- `alloc` full example uses heap features without hosted runtime
- `std` full example uses hosted runtime honestly

### Slice 44
Add integration tests that build all positive model examples and assert their summaries/runtime imports.

Completion criteria:

- one test suite walks all positive model examples

### Slice 45
Add integration tests that expect failure for all negative model examples.

Completion criteria:

- one test suite walks all negative model examples

---

## Epoch 9: Documentation And Book Alignment

### Slice 46
Audit the build book for every `fol_model` mention and make the contract explicit wherever examples could imply more than is actually allowed.

Completion criteria:

- no build-book example accidentally uses a forbidden surface for its declared model

### Slice 47
Audit routine/container/sugar book chapters for examples that silently assume `std`.

Completion criteria:

- hosted examples either declare `std` context or are rewritten to be model-neutral

### Slice 48
Add a concise “choose your model” guide with transitive-boundary examples.

Completion criteria:

- docs show when to move from `core` to `alloc`, and from `alloc` to `std`
- docs include one direct-dependency and one transitive-dependency example

---

## Epoch 10: Final Hardening And Audit Closure

### Slice 49
Add one top-level regression suite that combines:

- direct legality
- transitive legality
- routed execution legality
- emitted runtime import legality

Completion criteria:

- suite is broad enough to catch cross-layer drift in one place

### Slice 50
Audit all example directories and remove checked-in `.fol` generated artifacts that should not live in source examples.

Completion criteria:

- examples stay source-only unless a checked-in generated directory is intentional and documented

### Slice 51
Audit model-related diagnostics for consistency of wording.

Completion criteria:

- diagnostics consistently use:
  - `fol_model = core`
  - `fol_model = alloc`
  - `fol_model = std`
- no mixed old wording remains

### Slice 52
Final repo-wide scan for stale assumptions.

Completion criteria:

- scan docs, examples, tests, frontend, backend, runtime, and editor for stale wording or model drift
- mark plan complete only after the scan is clean

---

## Success Criteria

This plan is complete when all of the following are true:

- direct and transitive model-boundary failures are locked by tests
- routed `run` / `test` behavior is explicit and stable across mixed workspaces
- emitted Rust imports are audited for model purity in more than one happy path
- editor diagnostics and completion are model-aware even in mixed and ambiguous package layouts
- positive and negative example packages exist for the model split
- docs and book pages describe the same runtime-model contract everywhere
- the repo has no stale assumption that `std` is the informal baseline
