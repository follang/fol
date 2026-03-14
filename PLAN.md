# FOL Diagnostics Hardening Record

Last closed: 2026-03-14
Status: complete
Scope: `fol-diagnostics` plus the parser/package/resolver/CLI reporting surfaces that feed it

## Purpose

This file is no longer an active work plan.

It records that the diagnostics hardening phase was completed and summarizes the
compiler-facing reporting contract that now exists at head.

## Completed Outcome

- `fol-diagnostics` now uses a structured in-memory model instead of a thin
  one-location report shape.
- Diagnostics now carry:
- stable producer-owned codes
- one primary label
- zero or more secondary labels
- notes
- helps
- suggestions
- Human diagnostics now render source snippets, underline primary spans, and
  degrade cleanly when source text cannot be loaded.
- JSON diagnostics now preserve the same structured information directly instead
  of flattening to a lossy machine summary.
- Parser, package, and resolver now lower through a shared diagnostics-facing
  boundary instead of ad hoc producer-specific CLI handling.
- Duplicate, ambiguity, and package-control failures now preserve related sites
  as structured labels.
- Warning and info severities are now first-class and renderer-tested.
- CLI integration tests now lock rich human and JSON output shapes end to end.

## Validation Baseline

- `make build`: passed
- `make test`: passed
- current observed totals:
- `4` unit tests passed
- `1376` integration tests passed

## Phase Summary

- Phase `0`: contract reset complete
- Phase `1`: rich diagnostic model complete
- Phase `2`: human renderer hardening complete
- Phase `3`: JSON renderer hardening complete
- Phase `4`: stable producer-owned code mapping complete
- Phase `5`: shared producer lowering complete
- Phase `6`: multi-location related-label diagnostics complete
- Phase `7`: warning/info and guidance plumbing complete
- Phase `8`: CLI integration hardening complete
- Phase `9`: repo docs and book sync complete

All slices from `0.1` through `9.3` are complete.

## Current Contract

- Parser, package, and resolver failures retain exact primary locations where
  the underlying phase exposes them.
- Human-readable diagnostics show:
- severity
- code
- main message
- source snippet
- primary underline
- related labels
- notes and helps
- JSON diagnostics preserve the same rich structure with stable field names.
- The root CLI now lowers compiler glitches through one shared diagnostics
  adapter instead of keeping special-case per-producer extraction logic in the
  entrypoint.

## Definition Of Done

The diagnostics hardening milestone is complete for the current compiler
surface. Diagnostics are considered infrastructure-ready for the next semantic
phase.

Any further diagnostics work should be driven by new producer needs from later
phases such as type checking, not by unfinished parser/package/resolver
reporting gaps.

## Next Step

The next planned major phase is type checking.

See [PLAN_TYPE.md](./PLAN_TYPE.md) for the detailed `fol-typecheck` plan.
