# Bundled `std`

This is the bundled FOL standard-library root.

Normal projects should declare:

```fol
build.add_dep({
    alias = "std",
    source = "internal",
    target = "standard",
});
```

and then import bundled std through the dependency alias, for example:

```fol
use std: pkg = {std};
```

Bundled std is the normal path.

Use an explicit `--std-root <DIR>` override only for development and testing.

`core` and `memo` are not imported from here. They remain compiler/runtime capability modes.

External dependencies stay separate from bundled std.

- bundled std:
  - `source = "internal"`
  - `target = "standard"`
  - normally `alias = "std"`
- external packages:
  - `source = "loc" | "pkg" | "git"`
  - example: `examples/std_logtiny_git`

## Bootstrap Scope

`std` should start small and grow gradually.

The current bundled bootstrap surface is intentionally tiny:

- `std.fmt.answer(): int`
- `std.fmt.double(int): int`
- `std.fmt.math.answer(): int`
- `std.io.echo_int(int): int`
- `std.io.echo_str(str): str`

That is enough to prove:

- the toolchain ships a real importable `std`
- bundled std resolves without extra dependency setup
- FOL-authored std modules compile and run under `fol_model = "memo"`

`std.io` is currently just a thin FOL wrapper over the hosted `.echo(...)`
substrate.

`std.os` is still deferred until it has one honest user-facing API.

## Shipped Surface Summary

Current shipped bundled modules:

- `std.fmt`
- `std.fmt.math`
- `std.io`

Current shipped public routines:

- `fmt::answer(): int`
- `fmt::double(int): int`
- `fmt::math::answer(): int`
- `io::echo_int(int): int`
- `io::echo_str(str): str`

Canonical bootstrap examples:

- `examples/std_bundled_fmt`
- `examples/std_bundled_io`
- `examples/std_explicit_pkg`

Anything outside that list should not be documented as already shipped.

## Growth Rule

When bundled `std` gains a new public name:

- add or update a real example package
- add or update CLI/integration coverage
- add or update LSP/tree-sitter coverage
- update this README and `docs/bundled-std.md`
- update the relevant book pages
