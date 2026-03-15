# FOL V1 Error Handling Milestone

Status: complete  
Last updated: 2026-03-15

This milestone closed the remaining `V1` gap between:

- declared routine error types
- `report expr`
- call-site handling
- lowered IR
- CLI behavior
- user-facing docs

The current compiler now treats recoverable errors as a real `V1` feature
through the implemented chain:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck -> fol-lower`

## Completed Contract

The current `V1` contract is:

- routines declare errors as `ResultType / ErrorType`
- `report expr` is typechecked against the declared error type
- plain use of an error-aware call propagates only through a surrounding routine
  with a compatible declared error type
- `check(expr)` is the `V1` inspection surface for recoverable routine calls
- `expr || fallback` is the `V1` recovery surface
- `fallback` may provide a value, `report`, or `panic`
- postfix `!` remains scoped to `opt[...]` and `err[...]` shell values
- routine call results with `/ ErrorType` are not treated as `err[...]` shells

## Completed Lowered Boundary

Lowering now preserves one explicit recoverable-call ABI contract:

- `TaggedResultObject`
- boolean tag
- success slot
- error slot

Lowered IR now carries explicit recoverable-error operations for:

- inspection
- success extraction
- error extraction
- propagation
- panic fallback paths

The CLI and deterministic lowered dumps now lock that behavior end to end.

## Repaired Regressions

The reopened hardening pass closed the concrete regressions found from real CLI
probes:

- routine parameters now lower through the correct routine-owned scope
- typed non-empty container literals lower through the correct family-specific
  paths
- all-exit `when` expressions no longer try to synthesize dead join values

## Validation Baseline

Latest green validation for this milestone:

- `make build` passed
- `make test` passed
- `8` unit tests passed
- `1537` integration tests passed

## Docs State

The milestone docs are now aligned:

- [README.md](./README.md)
- [PROGRESS.md](./PROGRESS.md)
- [book/src/650_errors/200_recover.md](./book/src/650_errors/200_recover.md)
- [book/src/700_sugar/200_pipes.md](./book/src/700_sugar/200_pipes.md)

## Next Step

Recoverable errors are complete for the current `V1` boundary. The next major
compiler milestone should return to the first real backend that consumes lowered
IR and carries the current `V1` subset toward executable artifact generation.
