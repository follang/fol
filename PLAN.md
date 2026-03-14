# V1 Typecheck Milestone

Status: complete

Last completed: 2026-03-14

## 1. Scope

This plan covered the first real `V1` semantic stage after package loading and
name resolution:

- `fol-typecheck`
- root CLI integration after resolver
- exact typecheck diagnostics through human and JSON outputs
- explicit unsupported boundaries for `V2` and `V3` surfaces

This was intentionally a `V1` milestone only. It did not aim to implement the
full language book at once.

## 2. Delivered

- Added `fol-typecheck` as a workspace crate.
- Lowered `ResolvedProgram` into typed semantic results.
- Installed canonical builtin `V1` types, including builtin `str`.
- Added normalized semantic type shapes and typed symbol/reference/node facts.
- Checked declaration signatures across bindings, aliases, records, entries,
  routine parameters, returns, and error types.
- Checked core `V1` expressions:
- literals
- plain and qualified identifiers
- assignments
- free calls
- method calls
- field/index/slice access
- record construction
- entry values
- array/vector/sequence/set/map literals
- Checked core control semantics:
- `return`
- `report`
- `when` branch agreement
- loop guard basics
- `never`-aware early exits such as `panic`, `return`, and `report`
- Froze the initial `V1` operator, coercion, and cast contract in code and tests.
- Made `V2` and `V3` surfaces fail explicitly instead of passing unchecked.
- Made ordinary source typechecking reject `build.fol` package-definition files.
- Wired the root CLI to run `fol-typecheck` after `fol-resolver`.
- Added end-to-end CLI tests for successful and failing typecheck runs,
  including JSON diagnostics.
- Synced repo status docs to the new stage boundary.

## 3. Validation Baseline

Latest completion baseline:

- `make build`: passed
- `make test`: passed
- unit tests: `5` passed
- integration tests: `1446` passed

## 4. Explicit Boundaries

This milestone does not mean the whole language is semantically implemented.

Still outside this completed plan:

- `V2` language semantics from [VERSIONS.md](./VERSIONS.md)
- `V3` systems and interop semantics from [VERSIONS.md](./VERSIONS.md)
- later `V1` compiler stages after typechecking
- backend, lowering, code generation, runtime, and binary production

Those surfaces should continue to fail explicitly or remain unimplemented until
their owning milestone exists.

## 5. Next Focus

The next compiler work should stay inside `V1` and move forward along the
compiler chain toward a binary-producing pipeline.

Use these files as the current reference point:

- [PROGRESS.md](./PROGRESS.md)
- [VERSIONS.md](./VERSIONS.md)
- the active test suite

Only after the `V1` compiler path is carried further should the project return
to `V2` and `V3` feature work.
