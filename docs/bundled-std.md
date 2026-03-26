# Bundled Std

FOL ships its standard library source with the toolchain.

Normal usage:

- `std` is resolved from the bundled tree at `lang/library/std`
- users do not add `std` as a dependency
- users do not download `std` separately

Model rules:

- `fol_model = "core"`: `use std ...` is forbidden
- `fol_model = "mem"`: `use std ...` is forbidden
- `fol_model = "std"`: `use std ...` is allowed

Implementation split:

- `core` and `mem` remain compiler/runtime capability layers in Rust
- `std` is the importable bundled library and should grow mostly in FOL

An explicit std-root override may still exist for development and testing, but it is not the normal user path.

## Editing Bundled Std

Normal local iteration should edit:

- `lang/library/std`

Normal compiler and CLI flows should pick it up automatically without extra flags.

Use an explicit std-root override only when you deliberately want to:

- test an alternate std checkout
- isolate resolver/import behavior with a synthetic std tree
- compare bundled std against a temporary experimental root

That override is for development and tests. It is not the normal user workflow.
