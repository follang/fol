# FOL Backend Milestone

Last updated: 2026-03-16

This plan is now closed as a completion record.

`fol-backend` is implemented for the current `V1` boundary.

The active compiler chain is now:

`fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck -> fol-lower -> fol-runtime -> fol-backend`

## What Landed

- `fol-backend` now exists as a workspace crate with a public backend API,
  backend config, artifact model, session model, and structured backend errors.
- The first backend target is Rust emission, but the crate boundary remains
  generic enough for later targets.
- Backend input is `LoweredWorkspace`; backend does not consume parser,
  resolver, or typecheck structures directly.
- Generated output is one Rust crate per lowered workspace, not one giant file.
- Emitted crate layout is deterministic and grouped by lowered package and
  namespace.
- Backend-owned symbol mangling is stable for packages, globals, routines,
  locals, and types.
- Rust type emission now covers current builtin scalar types plus
  runtime-backed strings, containers, shells, and recoverable values.
- Backend now emits current `V1` records and entries as Rust structs and enums.
- Backend instruction emission now covers:
  - plain routine calls
  - field access
  - scalar intrinsic calls
  - `.len(...)`
  - `.echo(...)`
  - recoverable inspection and unwrap helpers
  - shell construction and unwrap
  - runtime-backed indexing
- Backend control-flow emission now covers:
  - `Jump`
  - `Branch`
  - `Return`
  - `Report`
  - `Panic`
  - `Unreachable`
- Backend can now write generated crates to disk, build them through Cargo, and
  return a compiled binary artifact.
- CLI integration is real:
  - backend runs after lowering when a buildable entry routine exists
  - `--emit-rust` is real
  - `--keep-build-dir` is real
  - final artifact paths are surfaced to the user
- Source-to-output traceability is real:
  - lowered source symbols can map into emitted Rust module paths
  - backend trace records keep emitted-path and package context

## Current Coverage

The backend milestone is now locked by green `make build` and `make test`
coverage for:

- declaration-only inputs
- scalar entry programs
- records and entries
- containers plus `.len(...)`
- `.echo(...)`
- recoverable success/failure propagation
- `check(...)`
- `expr || fallback`
- package graphs spanning `loc`, `std`, and installed `pkg`
- backend diagnostics for impossible lowered-to-target situations
- deterministic emission order
- generated crate snapshots and emitted artifact summaries

## Current Boundary

This milestone completes the first runnable backend for the current `V1`
subset only.

It does not claim:

- optimization
- LLVM
- C ABI
- ownership or pointer semantics
- later `V2` / `V3` language features
- multiple mature backend targets

## Status

- backend milestone: complete
- current `V1` compiler-to-binary path: real
- follow-on work: hardening, future targets, `core` / `std`, and later-version
  language features
