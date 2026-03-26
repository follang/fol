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

## What Ships With FOL

The FOL distribution should be read as three separate pieces:

- compiler and runtime substrate:
  - parser
  - resolver
  - typechecker
  - backend
  - runtime-owned `core` and `mem` capability support
- bundled library source:
  - `lang/library/std`
- optional external dependencies:
  - added through `.build().add_dep(...)`
  - not required for normal `std` usage

Import rule:

- only `std` is imported from source code
- `core` and `mem` are selected through `fol_model`, not imported

An explicit `--std-root <DIR>` override may still exist for development and testing, but it is not the normal user path.

## Bootstrap Surface

The bundled shipped std is intentionally small right now.

Current public bootstrap modules:

- `std.fmt`
- `std.fmt.math`

Current bootstrap routines:

- `fmt::answer(): int`
- `fmt::double(int): int`
- `fmt::math::answer(): int`

That keeps the first shipped std honest:

- real FOL package
- real import path
- real hosted example coverage
- no fake placeholder `std.io` or `std.os` modules yet

## Editing Bundled Std

Normal local iteration should edit:

- `lang/library/std`

Normal compiler and CLI flows should pick it up automatically without extra flags.

Use an explicit `--std-root <DIR>` override only when you deliberately want to:

- test an alternate std checkout
- isolate resolver/import behavior with a synthetic std tree
- compare bundled std against a temporary experimental root

That override is for development and tests. It is not the normal user workflow.
