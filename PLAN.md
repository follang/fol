# FOL Lowering Hardening Completion Record

Last updated: 2026-03-15

This file records the completed hardening pass that reopened `fol-lower` after
real CLI probes found gaps in the claimed `V1` lowering boundary.

## 0. Final Outcome

The reopened lowering blockers are closed.

For the current `V1` language boundary in [`VERSIONS.md`](./VERSIONS.md),
`fol-lower` should now be treated as implemented again.

The active compiler chain is:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck -> fol-lower`

## 1. What Was Repaired

### 1.1 Routine Parameter Symbol Lowering

- Lowering no longer loses ordinary routine parameters when multiple routines in
  the same package or source unit reuse the same parameter names.
- Routine parameter lookup now uses the routine-owned syntax scope instead of a
  broader enclosing scope that could ambiguously match descendants.
- True missing-parameter situations now fail explicitly with lowering-owned
  diagnostics instead of silently skipping parameter locals.

### 1.2 Typed Container Literal Lowering

- Typed non-empty `arr`, `vec`, `seq`, `set`, and `map` literals now lower
  through their typechecked container family instead of falling into the old
  empty-container fallback path.
- Exact lowered instruction-shape assertions now lock those repaired container
  families instead of only checking that the CLI succeeds.

### 1.3 `when` Control-Flow Lowering

- Statement `when` lowering no longer fabricates unreachable continuation blocks
  when every branch exits through `return`, `report`, or other terminating flow.
- The repaired CFG shape is locked by dedicated lowering tests.

### 1.4 End-To-End Proof

- A real multi-surface `V1` program using records, routine parameters, typed
  non-empty containers, loops, and early-return `when` control flow now compiles
  successfully through the root CLI.
- The same fixture now has `--dump-lowered` coverage so its lowered workspace
  shape remains inspectable and stable.
- The earlier failing sample families now have direct CLI regression tests:
  parameter-heavy lowering, typed container literals, and all-exit `when`
  control flow.

## 2. Validation Baseline

Latest green validation for this completed hardening pass:

- `make build`
- `make test`
- `8` unit tests passed
- `1513` integration tests passed

## 3. Documentation Sync

The hardening closeout is reflected in:

- [`README.md`](./README.md)
- [`PROGRESS.md`](./PROGRESS.md)

Those files now describe the repaired lowering boundary as real and backed by
end-to-end coverage.

## 4. What Comes Next

The next major compiler milestone should be the first backend that consumes the
lowered `V1` IR and carries a valid `V1` program toward binary production.

This plan is complete. Any new lowering work should be treated as either:

- backend-driven extension work, or
- a new bug-specific hardening pass triggered by a concrete repro
