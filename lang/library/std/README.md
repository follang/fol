# Bundled `std`

This is the bundled FOL standard-library root.

Normal toolchain behavior should resolve:

- `use ...: std = {...}`

against this tree automatically when:

- `fol_model = "std"`

Bundled std is the normal path.

Use an explicit `--std-root <DIR>` override only for development and testing.

`core` and `mem` are not imported from here. They remain compiler/runtime capability modes.

## Bootstrap Scope

`std` should start small and grow gradually.

The current bundled bootstrap surface is intentionally tiny:

- `std.fmt.answer(): int`
- `std.fmt.double(int): int`
- `std.fmt.math.answer(): int`

That is enough to prove:

- the toolchain ships a real importable `std`
- bundled std resolves without extra dependency setup
- FOL-authored std modules compile and run under `fol_model = "std"`

`std.io` and `std.os` are deferred until they contain honest user-facing APIs.
