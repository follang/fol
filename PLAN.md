# FOL Intrinsics Milestone

Last updated: 2026-03-15

This file is now a completion record for the `fol-intrinsics` milestone.

## Outcome

`fol-intrinsics` is implemented as the shared compiler-owned intrinsic registry
for the current `V1` compiler pipeline.

The active compiler chain is now:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck -> fol-intrinsics -> fol-lower`

`fol-intrinsics` is shared infrastructure rather than a separate execution
stage, but it is now the semantic source of truth for intrinsic identity and
classification across typecheck, lowering, diagnostics, CLI behavior, and book
documentation.

## What Landed

### Registry Foundation

The crate now owns:

- canonical intrinsic ids
- canonical spellings and aliases
- intrinsic surfaces:
  - dot-root calls
  - keyword intrinsics
  - operator-alias intrinsics
- intrinsic categories
- milestone/roadmap classification
- backend role classification
- lowering lookup helpers
- structured intrinsic diagnostics

### Implemented `V1` Intrinsics

The currently implemented `V1` intrinsic subset is:

- `.eq(...)`
- `.nq(...)`
- `.lt(...)`
- `.gt(...)`
- `.ge(...)`
- `.le(...)`
- `.not(...)`
- `.len(...)`
- `.echo(...)`
- `check(...)`
- `panic(...)`

These are now wired through the shared registry and are no longer treated as
scattered ad hoc compiler special cases.

### Explicitly Deferred Intrinsics

The registry also records deferred or future-only surfaces so the compiler can
report explicit milestone-boundary diagnostics instead of generic unsupported
errors.

That includes:

- current deferred `V1` candidates such as `as`, `cast`, `.cap(...)`,
  `.is_empty(...)`, `.low(...)`, `.high(...)`
- later `V2` and `V3` intrinsic families
- roadmap/library-oriented placeholders that should likely live in `core` or
  `std` instead of as compiler intrinsics

### Typecheck And Lowering Integration

Typecheck now routes intrinsic ownership and validation through the registry
for:

- comparisons
- boolean `.not(...)`
- length query `.len(...)`
- diagnostic `.echo(...)`
- keyword intrinsics `check(...)` and `panic(...)`
- explicit milestone-boundary failures for deferred intrinsic families

Lowering now routes intrinsic lowering through registry-backed decisions and
retains explicit intrinsic identity in lowered output.

The lowered renderer and verifier now make intrinsic behavior inspectable and
defensible:

- lowered dumps show canonical intrinsic names
- lowered dumps show backend roles
- verifier rejects impossible intrinsic instruction shapes

## Documentation State

The book and repo docs were updated to match the implemented intrinsic
contract.

Synced documentation includes:

- `README.md`
- `PROGRESS.md`
- `book/src/300_meta/100_buildin.md`
- cross-references in the type, routines, and sugar chapters

The book now distinguishes:

- compiler intrinsics
- ordinary `core` / `std` library APIs
- implemented `V1` intrinsic surfaces
- deferred `V1` / `V2` / `V3` surfaces

## Validation Baseline

Latest validation for this milestone:

- `make build` passed
- `make test` passed
- `18` unit tests passed
- `1567` integration tests passed

## Status

This milestone is complete for the current `V1` compiler boundary.

The next major milestone should build on this registry rather than reintroduce
intrinsic-specific logic in later stages.
