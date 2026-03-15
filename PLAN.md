# FOL Lowering Hardening Plan

Last updated: 2026-03-15

This file reopens the lowering milestone.

The previous closeout record was too optimistic. Real CLI probes against the
current `V1` surface exposed lowering regressions that conflict with the
language boundary in [`VERSIONS.md`](./VERSIONS.md).

`fol-lower` is substantial and mostly implemented, but it should not be treated
as fully complete for `V1` again until the concrete blockers below are fixed and
locked with end-to-end tests.

## 0. Why This Plan Was Reopened

The reopening is based on fresh real CLI probes, not on hypothetical concerns.

These failures were reproduced directly from `target/debug/fol` on temporary
fixtures:

### 0.1 Lowering Loses Some Resolved Value Symbols

Observed error:

- `LoweringInvalidInput: value symbol 'flag' does not map to a lowered local or global definition`

This happened while lowering a valid `V1` routine that used a parameter in
control-flow and imported-method contexts.

That means some resolved/typechecked value symbols still fail to lower through
the current symbol-to-local/global translation path.

### 0.2 Container Literals Still Fail In Real CLI Paths

Observed error:

- `LoweringUnsupported: empty linear container literals require an expected container type in lowered V1`

This was triggered by non-empty typed `V1` container literals such as:

- `var names: seq[str] = {"Ada", "Lin"}`
- `var counts: map[str, int] = {{"ada", 1}, {"lin", 2}}`

That is a real contract violation because `VERSIONS.md` places a practical
container subset in `V1`, and the lowering milestone claimed arrays, vectors,
sequences, sets, and maps were already lowered.

### 0.3 Value-Producing `when` Is Not End-To-End Stable Yet

Observed error:

- `LoweringInvalidInput: value-producing when did not retain a lowered join value`

That means one of the core `V1` control-flow/value-lowering surfaces still has a
real join-state bug in live CLI execution.

## 1. Scope Of This Reopened Plan

This is not a new lowering phase.
It is a hardening pass over the existing `V1` lowering milestone.

It is responsible for:

- fixing the reproduced `V1` lowering regressions
- adding exact library-level and CLI-level tests for them
- removing the mismatch between claimed `V1` support and actual lowering behavior
- re-closing the lowering milestone only after the fixes are proven end to end

It is not responsible for:

- new `V2` or `V3` language features
- backend work
- LLVM
- C backend generation
- ownership / borrowing
- C ABI

## 2. Hard Definition Of Done

This plan is done only when all of the following are true:

- routine parameters and other ordinary resolved value symbols lower reliably in
  all current `V1` expression/control-flow contexts that the typechecker accepts
- typed non-empty `V1` container literals lower successfully through the real CLI
- value-producing `when` lowers successfully through the real CLI
- the failing sample families are locked by integration tests, not just unit tests
- [`README.md`](./README.md), [`PROGRESS.md`](./PROGRESS.md), and this file no
  longer overstate lowering support

## 3. Execution Strategy

Fix the regressions in the same order they were observed in real use:

1. value-symbol lowering gaps
2. container literal lowering
3. value-producing `when`
4. end-to-end confirmation and docs

Each fix must land with:

- the code change
- the exact test for that surface
- `make build`
- `make test`

## 4. Implementation Slices

### Phase 0. Repro And Fixture Lock-In

- `0.1` `done` Add focused lowering-library repro tests for routine-parameter symbol lowering through control-flow-heavy bodies.
- `0.2` `done` Add focused lowering-library repro tests for non-empty `seq` and `map` literal lowering in typed `V1` contexts.
- `0.3` `done` Add focused lowering-library repro tests for value-producing `when` lowering with explicit join values.
- `0.4` `done` Add one end-to-end CLI repro fixture that combines ordinary globals, records, routine parameters, containers, loops, and value-producing `when`.

### Phase 1. Value Symbol Hardening

- `1.1` `pending` Audit lowered symbol lookup for routine parameters, local bindings, and imported mounted symbols to find where current lookup still misses valid lowered locals.
- `1.2` `pending` Fix the lowering path so ordinary routine parameters always map to lowered locals anywhere current `V1` typing can reference them.
- `1.3` `pending` Add negative guards so true missing-symbol situations still report explicit lowering errors instead of silently aliasing the wrong symbol.

### Phase 2. Container Literal Hardening

- `2.1` `done` Trace why typed non-empty `seq` / `map` literals currently fall into the “empty linear container” lowering error.
- `2.2` `done` Fix linear-container lowering so typed `arr` / `vec` / `seq` literals lower correctly in binding, return, and call-argument contexts.
- `2.3` `done` Fix `set` / `map` lowering so typed key/value aggregates lower correctly in binding, return, and index-lookup contexts.
- `2.4` `done` Add exact lowering-shape assertions for the repaired container instructions instead of only checking that the CLI succeeds.

### Phase 3. Value `when` Join Hardening

- `3.1` `done` Audit join-local allocation and branch-result wiring for value-producing `when`.
- `3.2` `done` Fix value-producing `when` so all successful typed branches retain one lowered join destination.
- `3.3` `pending` Add exact lowered block/instruction assertions for the repaired value-producing `when` path.

### Phase 4. End-To-End Proof

- `4.1` `pending` Add CLI success coverage for a real multi-surface `V1` program that includes records, routine parameters, non-empty containers, loops, and value-producing `when`.
- `4.2` `pending` Add `--dump-lowered` snapshot coverage for that same real `V1` program so the lowered shape is inspectable and stable.
- `4.3` `pending` Re-run the earlier failing sample families and lock them as regression tests.

### Phase 5. Documentation Closeout

- `5.1` `pending` Update [`README.md`](./README.md) and [`PROGRESS.md`](./PROGRESS.md) only after the repaired surfaces are actually green end to end.
- `5.2` `pending` Rewrite this file into a true completion record only after the repaired `V1` lowering boundary is real again.

## 5. What Should Happen After This Plan

Only after these hardening slices are complete should the project treat
`fol-lower` as fully done for `V1` and move cleanly to the first backend plan.
