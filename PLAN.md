# FOL Runtime Completion

Last updated: 2026-03-15

`fol-runtime` is complete for the current `V1` compiler boundary.

The implemented `V1` chain is now:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck -> fol-lower -> fol-runtime`

What this milestone completed:

- a dedicated `fol-runtime` workspace crate
- stable scalar/runtime string policy
- runtime container families for current `V1`
- optional and error shell runtime types
- recoverable routine-result runtime ABI
- stable `.len(...)`, `.echo(...)`, `check(...)`, and top-level process-outcome helpers
- aggregate formatting hooks for backend-authored records and entries
- deterministic ordering/formatting guarantees for runtime-backed sets and maps
- backend-facing crate docs that define:
  - builtin mapping expectations
  - lowered-instruction/runtime boundaries
  - generated crate/import expectations
  - the first backend integration guide
- synced repo/book documentation for the runtime contract

Validation status at close:

- `make build`: passed
- `make test`: passed
- `18` unit tests passed
- `1567` integration tests passed

Definition of done at close:

- `fol-runtime` provides the full current `V1` support surface expected by the first backend
- current runtime-backed builtin/query/error/container behavior is explicit and tested
- current repo and book docs acknowledge the runtime contract
- the next milestone is no longer runtime design, but backend implementation

Next recommended milestone:

- create the first real `fol-backend` against `fol-lower` + `fol-runtime`
- generate a Rust crate from lowered workspaces
- compile that generated crate toward the first runnable `V1` binaries
