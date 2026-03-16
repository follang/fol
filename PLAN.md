# FOL Frontend Milestone Complete

Last updated: 2026-03-16

This plan is closed.

`fol-frontend` is now implemented as the canonical user-facing workflow layer
for the current `V1` compiler stack.

## Completed Boundary

The current toolchain shape is now:

- `fol-stream`
- `fol-lexer`
- `fol-parser`
- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-lower`
- `fol-intrinsics`
- `fol-runtime`
- `fol-backend`
- `fol-frontend`

`fol-frontend` now owns:

- derive-based `clap` command parsing
- workspace and package discovery
- project and workspace scaffolding
- package preparation/fetch orchestration over `fol-package`
- `check`, `build`, `run`, `test`, `emit`, `clean`, and `completion`
- human/plain/json output
- color policy handling
- explicit artifact-root reporting
- frontend diagnostic guidance
- root-binary migration from the old direct compiler driver

## Command Surface

The current frontend command surface is real and test-backed:

- `init`
- `new`
- `work info`
- `work list`
- `fetch`
- `check`
- `build`
- `run`
- `test`
- `emit rust`
- `emit lowered`
- `clean`
- `completion`
- hidden `_complete`

## Hardening Result

The frontend milestone now has:

- visible aliases and grouped help
- output/color/profile precedence coverage
- workflow walkthrough coverage
- completion generation and dispatch coverage
- clean migration boundaries between frontend workflows and legacy direct flags
- explicit build, emit, package, and binary artifact reporting
- repo docs and book docs that describe `fol` as the user-facing tool

## What This Does Not Mean

This closeout does not claim:

- remote dependency fetching is complete
- package lockfile/version workflows are complete
- the root legacy compiler path has been fully removed
- later-version language work is done
- additional backends are done

Those are future milestones.

## Validation State At Closeout

The milestone was closed with:

- `make build` passing
- `make test` passing
- `46` unit tests passing
- `1616` integration tests passing

## Next Likely Work

The next useful work is no longer “create a frontend.” That is done.

The likely next fronts are:

- package/fetch/store expansion
- `core` / `std` library work
- backend expansion beyond the first Rust backend
- later-version language milestones
