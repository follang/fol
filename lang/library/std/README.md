# Bundled `std`

This is the bundled FOL standard-library root.

Normal toolchain behavior should resolve:

- `use ...: std = {...}`

against this tree automatically when:

- `fol_model = "std"`

`core` and `mem` are not imported from here. They remain compiler/runtime capability modes.
