# Bundled Std

FOL ships its standard library source with the toolchain.

Finalized design contract:

- public capability modes are only:
  - `core`
  - `memo`
- bundled standard-library package identity is:
  - `standard`
- the normal dependency alias in user projects is:
  - `std`
- source code should reach bundled std through the dependency system with `pkg`
  imports, for example:
  - `use std: pkg = {std};`

Normal build usage:

- users do not download `std` separately
- users add the bundled standard library explicitly in `build.fol`:

```fol
build.add_dep({
    alias = "std",
    source = "internal",
    target = "standard",
});
```

Implementation split:

- `core` and `memo` remain compiler/runtime capability layers in Rust
- `std` is the importable bundled library and should grow mostly in FOL

## What Ships With FOL

The FOL distribution should be read as three separate pieces:

- compiler and runtime substrate:
  - parser
  - resolver
  - typechecker
  - backend
  - runtime-owned `core` and `memo` capability support
- bundled library source:
  - `lang/library/std`
- optional external dependencies:
  - added through `.build().add_dep(...)`
  - bundled std uses the same dependency surface with `source = "internal"`

Dependency distinction:

- bundled std:
  - `source = "internal"`
  - `target = "standard"`
  - usually `alias = "std"`
- external packages:
  - `source = "loc" | "pkg" | "git"`
  - examples like `examples/std_logtiny_git` stay ordinary external dependencies
  - they do not replace or implicitly provide bundled std

Import rule:

- only `std` is imported from source code as a dependency alias
- `core` and `memo` are selected through `fol_model`, not imported

An explicit `--std-root <DIR>` override may still exist for development and testing, but it is not the normal user path.

## Bootstrap Surface

The bundled shipped std is intentionally small right now.

Current public bootstrap modules:

- `std.fmt`
- `std.fmt.math`
- `std.io`

Current bootstrap routines:

- `fmt::answer(): int`
- `fmt::double(int): int`
- `fmt::math::answer(): int`
- `io::echo_int(int): int`
- `io::echo_str(str): str`

`std.io` is intentionally narrow right now. It wraps the hosted `.echo(...)`
primitive instead of replacing it.

Current rule:

- `.echo(...)` remains the low-level hosted substrate
- `std.io` is the first bundled public wrapper over that substrate

That keeps the first shipped std honest:

- real FOL package
- real import path
- real hosted example coverage
- no fake placeholder `std.os` module yet

Canonical bootstrap example packages:

- `examples/std_bundled_fmt`
- `examples/std_bundled_io`
- `examples/std_explicit_pkg`

Current shipped public routines:

- `fmt::answer(): int`
- `fmt::double(int): int`
- `fmt::math::answer(): int`
- `io::echo_int(int): int`
- `io::echo_str(str): str`

Older hosted std examples should use bundled std modules when one already exists.
That means current echo-based examples should prefer `std.io` instead of calling
`.echo(...)` directly unless the example is explicitly about the primitive
substrate.

## Editing Bundled Std

Normal local iteration should edit:

- `lang/library/std`

Normal compiler and CLI flows should pick it up automatically without extra flags.

Use an explicit `--std-root <DIR>` override only when you deliberately want to:

- test an alternate std checkout
- isolate resolver/import behavior with a synthetic std tree
- compare bundled std against a temporary experimental root

That override is for development and tests. It is not the normal user workflow.
