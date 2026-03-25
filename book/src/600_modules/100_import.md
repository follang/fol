# Imports

An import declaration states that the source file containing the declaration depends on functionality of the imported package and enables access to exported identifiers of that package.

Syntax to import a library is:
```
use alias: source_kind = { source }
```

Current source kinds are:
- `loc` for local directory imports
- `std` for standard-library directory imports
- `pkg` for installed external packages

## What `use` imports

`use` works against the source-layout model:

- the folder root is the package
- subfolders are namespaces inside that package
- a file is only a source file, not a separate module by itself

So a `use` target is normally:

- a whole package
- or one namespace inside that package

`use` does **not** mean "import another file from the same folder".
Files in the same package already share package scope.
Imports are for reaching another package or another namespace boundary.

Also note:

- `use` brings in exported functionality
- `hid` declarations remain file-only even inside the same package
- importing a package does not erase the original file boundary rules
- `use` is for consuming functionality, not for defining package dependencies

## Import kinds

### `loc`

`loc` is the simplest import kind:

- it points to a local directory
- that directory is scanned as a FOL package / namespace tree
- no `package.yaml` is required
- no `build.fol` is required

This makes `loc` useful for local workspace code, experiments, and monorepo-style sharing.

### `std`

`std` works like `loc`, except the directory is resolved from the toolchain's standard-library root.

So:

- `std` imports are directory-backed
- they are owned by the FOL toolchain
- they do not need user-managed package metadata in source code

### `pkg`

`pkg` is for formal external packages.

Unlike `loc` and `std`, a `pkg` import does not just point at an arbitrary source directory.
It points at an installed package root that must define its identity and build surface explicitly.
The package layer discovers that root first, and ordinary name resolution happens only after the package has been prepared.

For a `pkg` package root:

- `build.fol` is required
- `build.fol` stores package metadata, direct dependencies, and build logic

## Package Metadata And Build Files

Formal packages use one control file at the root:

- `build.fol`

### `build.fol`

`build.fol` is the package build entry file.

This file is responsible for:

- declaring package metadata
- declaring direct dependencies
- declaring package build logic
- declaring artifacts, steps, and generated outputs through the build API
- becoming the canonical package entrypoint for `fol code build/run/test/check`

`build.fol` is still an ordinary FOL file.
It is parsed with the same front-end as other `.fol` sources.
The difference is that the package layer evaluates one canonical build routine inside it.

Today that means:

- `fol code build/run/test/check` starts from `build.fol`
- the canonical entry is `pro[] build(): non`
- old `def root: loc = ...` and `def build(...)` forms are not the build model

So:

- `def` is still a general FOL declaration form
- `build.fol` is not a separate mini-language
- `build.fol` uses an ordinary routine entrypoint, like Zig's `build.zig`

That means:

- ordinary source `.fol` files use `use` to consume packages/namespaces
- `build.fol` uses `pro[] build(): non` plus `.build()` to configure package metadata,
  direct dependencies, and the build graph

So `use` and the build routine serve different jobs:

- `use` = consume functionality
- `pro[] build()` in `build.fol` = define package/build surface

## System libraries
This is how including other libraries works, for example include `fmt` module from standard library:
```
use fmt: std = {"fmt"};

pro main: ini = {
    fmt::log.warn("Last warning!...")
}
```
To use only the `log` namespace of `fmt` module:
```
use log: std = {"fmt/log"};

pro[] main: int = {
    log.warn("Last warning!...")
}
```
But let's say you only wanna use ONLY the `warn` functionality of `log` namespace from `fmt` module:
```
use warn: std = {"fmt/log"};

pro[] main: int = {
    warn("Last warning!...")
}
```
## Local libraries
To include a local package or namespace, point `loc` at the directory:

```
use bend: loc = {"../folder/bender"};
```
Then to acces only a namespace:
```
use space: loc = {"../folder/bender/space"};
```

That second form is namespace import, not "single file import".
If `space` contains multiple `.fol` files in the same folder, they still belong to the same imported namespace.

`loc` does not require `build.fol`.
But if the target directory already defines `build.fol` at its root, that directory is treated as a formal package root and should be imported through `pkg`, not `loc`.

## External packages
External packages are imported through `pkg`:

```
use space: pkg = {"space"};
```

`pkg` imports are different from `loc` and `std`:

- the imported root is an installed package root
- that root must contain `build.fol`
- `build.fol` is the package control file and currently defines package metadata,
  dependencies, exports, and build declarations that package loading depends on
- raw transport URLs do not appear in source code; package acquisition and installed-package preparation are separate from ordinary source resolution
