# Bundled FOL Libraries

This tree contains FOL library source that ships with the toolchain.

Current intent:

- `std` lives here as bundled source
- users should not download `std` separately for normal usage
- `core` and `memo` remain compiler/runtime capability modes, not importable libraries

The normal bundled standard-library root is:

- `lang/library/std`
