# FOL Package Plan

Status: complete
Last rebuilt: 2026-03-13
Scope: `fol-package` migration and the minimal resolver/CLI/doc work needed to make package loading a dedicated crate boundary

## Completion Summary

The `fol-package` migration is complete.

Delivered:

- `fol-package` exists as a workspace crate between parsed source packages and `fol-resolver`
- `package.yaml` metadata parsing lives in `fol-package`
- `build.fol` parsing and package-definition extraction live in `fol-package`
- `loc`, `std`, and installed `pkg` loading flow through `fol-package`
- package-session caching, cycle handling, and shared dependency dedupe live in `fol-package`
- prepared export mounts are computed in `fol-package` and consumed by resolver
- resolver no longer owns package control-file parsing or package root loading
- the CLI now prepares entry packages through `fol-package` before resolution
- installed-package locators have an explicit model
- future git-like locator forms fail with explicit placeholder diagnostics
- future C ABI package records have inert placeholder modeling without active semantics

## Final Contract

- `loc`: manifest-free local directory import
- `std`: manifest-free toolchain directory import for the current stdlib phase
- `pkg`: installed package import requiring `package.yaml` and `build.fol`
- `package.yaml`: metadata only
- `build.fol`: ordinary FOL syntax whose recognized top-level definitions provide package dependency/export meaning
- direct file imports remain invalid
- stray `package.fol` files are ignored

## Validation Baseline

Latest green validation for this completed plan:

- `make build` passed
- `make test` passed
- `2` unit tests passed
- `1363` integration tests passed

## Slice Status

Completed slices: `33 / 33`
Plan progress: `100%`

All phases completed:

- crate and boundary reset
- metadata and build parsing migration
- package session foundation
- dependency and export graph ownership
- resolver integration
- CLI and public API migration
- locator and distribution groundwork
- C ABI groundwork
- docs and closeout

## Next Boundary

This plan is closed.

The active next phase is post-resolution semantic work, especially whole-program
type checking. Future package acquisition and real C ABI activation remain later
`fol-package` follow-up work, not open blockers for the completed package-loading
boundary.
