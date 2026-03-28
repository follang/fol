# fol-editor Baseline Repair Plan

This plan is only for repairing the current `fol-editor` baseline so the
remaining V1 slices can be executed safely.

It is not a feature-growth plan.
It is not V2 work.
It is not a compatibility plan.

The goal is simple:

- make the current `fol-editor` navigation/editor baseline green
- remove stale tests that no longer match compiler-backed truth
- repair the shared editor seams that block further slice batches

Success criteria for this repair plan:

- `cargo test -p fol-editor navigation -- --nocapture` passes
- `make build` passes
- `make test` passes
- the repo is clean except for unrelated user-owned changes

## Epoch 1: Reconfirm The Baseline

### Slice 1 (complete)
Re-run the targeted failing navigation/editor baseline and pin the exact red set
in notes/tests.

Completion criteria:

- the failing test inventory is re-confirmed on the committed baseline
- failures are grouped by root cause, not treated as unrelated noise

### Slice 2 (complete)
Audit the remaining open slices in the previous editor plan and mark which are
blocked by baseline faults rather than missing feature work.

Completion criteria:

- a short mapping exists from failing tests to blocking root causes

## Epoch 2: Workspace And Overlay Repair

### Slice 3 (complete)
Repair editor overlay/materialization behavior for multi-package and workspace
roots.

Target behavior:

- local/workspace imports resolve against the copied analysis tree correctly
- sibling packages are available during overlay analysis
- the analyzed document remains traceable back to the real source path

Completion criteria:

- workspace-symbol and local-workspace navigation tests stop failing due to
  missing copied roots

### Slice 4 (complete)
Repair path normalization between overlay paths and source paths.

Target behavior:

- definition/references/rename/symbol results point back to real source files
- editor does not leak temp overlay paths to LSP consumers

Completion criteria:

- workspace/member navigation results use source paths consistently

## Epoch 3: Navigation Lookup Repair

### Slice 5 (complete)
Relax editor-side navigation lookup so it is not limited to exact resolver
reference node hits when a symbol can still be identified safely.

Target behavior:

- definition/references/rename work when the cursor lands on a declaration
- same-package namespaced use sites resolve more reliably
- imported namespace navigation becomes less brittle

Completion criteria:

- failing same-file and same-package navigation tests turn green

### Slice 6 (complete)
Repair same-file local reference inclusion/exclusion behavior.

Target behavior:

- include-declaration and exclude-declaration paths both behave correctly
- local references do not lose the declaration location

Completion criteria:

- local reference tests are green

### Slice 7 (complete)
Repair current-package multi-file rename lookup.

Target behavior:

- same-package top-level rename finds the declaration and usage files
- build-entry rename still rejects cleanly at the current safe boundary

Completion criteria:

- top-level rename tests and same-package rename tests are green

## Epoch 4: Local Origin Repair

### Slice 8 (complete)
Repair missing declaration-origin data for local bindings and parameters.

Target behavior:

- rename/reference flows can find declaration locations for locals/parameters
- solution may be compiler-backed origin propagation or a narrow editor fallback,
  but it must stay honest and deterministic

Completion criteria:

- local binding rename test is green
- parameter rename test is green

### Slice 9 (complete)
Audit local-origin repair across other supported local classes.

Target behavior:

- label/destructure/capture/loop-binder classes do not regress silently

Completion criteria:

- tests or explicit audit notes cover the currently supported local classes

## Epoch 5: Signature Help Repair

### Slice 10 (complete)
Repair plain-call signature help.

Completion criteria:

- plain routine-call signature help test is green

### Slice 11 (complete)
Repair qualified-call signature help.

Completion criteria:

- qualified namespaced call signature help test is green

### Slice 12 (complete)
Repair build-file signature help.

Completion criteria:

- build-file helper-call signature help test is green

## Epoch 6: Quick Fix Truth Alignment

### Slice 13 (complete)
Re-audit unresolved-name quick-fix expectations against the actual diagnostic
suggestion path.

Target behavior:

- if compiler-backed suggestions exist, editor surfaces them
- if they do not exist, tests stop pretending they do

Completion criteria:

- unresolved-name quick-fix tests match real compiler-backed truth

### Slice 14 (complete)
Repair requested-diagnostic-context filtering for code actions.

Completion criteria:

- code actions only appear when the requested diagnostic matches

### Slice 15 (complete)
Re-audit typecheck-only no-action expectations.

Completion criteria:

- tests prove action-free behavior for typecheck diagnostics without exact
  replacements
- stale assumptions are deleted

## Epoch 7: Diagnostics And Wording Repair

### Slice 16 (complete)
Repair future-version boundary diagnostic expectations.

Target behavior:

- tests match the actual current compiler/editor wording and related-info shape
- no stale requirement for a `V2` literal if the real diagnostic changed

Completion criteria:

- future-boundary editor test is green and honest

### Slice 17 (complete)
Sweep the remaining navigation tests for stale current-contract wording.

Completion criteria:

- tests refer to the real current V1 contract only

## Epoch 8: Close The Baseline

### Slice 18 (complete)
Run the full targeted editor navigation suite and ensure it is green.

Completion criteria:

- `cargo test -p fol-editor navigation -- --nocapture` passes

### Slice 19 (complete)
Run the repo gate.

Completion criteria:

- `make build` passes
- `make test` passes

### Slice 20 (complete)
Commit the repair batch and mark this repair plan complete.

Completion criteria:

- committed with one conventional-commit title only
- `PLAN.md` fully marked complete
- worktree left clean except unrelated user-owned changes
