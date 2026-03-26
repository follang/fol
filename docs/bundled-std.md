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
