# Bundled `std`

This is the bundled FOL standard-library root.

Normal toolchain behavior should resolve:

- `use ...: std = {...}`

against this tree automatically when:

- `fol_model = "std"`

Bundled std is the normal path.

Use an explicit `--std-root <DIR>` override only for development and testing.

`core` and `mem` are not imported from here. They remain compiler/runtime capability modes.

## Growth Roadmap

`std` should start small and grow gradually.

Near-term intended families:

- `fmt`
- `io`
- `os`

The immediate goal is not a giant library rewrite. The goal is:

- ship a small real bundled std in FOL
- prove imports and hosted execution against that tree
- grow module families intentionally as the language surface matures
