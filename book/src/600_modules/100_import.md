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

For a `pkg` package root:

- `package.yaml` is required
- `build.fol` is required
- `package.yaml` stores metadata only
- `build.fol` declares dependencies and exports

## Package Metadata And Build Files

Formal packages use two files at the root:

- `package.yaml`
- `build.fol`

### `package.yaml`

`package.yaml` is metadata only.
It is intentionally not a normal `.fol` source file.

Typical metadata belongs here:

- package name
- version
- package kind
- human-oriented description/license/author data

What does **not** belong here:

- `use`
- dependency edges
- export wiring
- build logic

### `build.fol`

`build.fol` is where the package is assembled.

This file is responsible for:

- declaring package dependencies
- declaring which directories/namespaces are exported
- defining the package surface that consumers import

And this is where `def` belongs.

That means:

- ordinary source `.fol` files use `use` to consume packages/namespaces
- `build.fol` uses `def` to define package dependencies and exports

So `use` and `def` serve different jobs:

- `use` = consume functionality
- `def` in `build.fol` = define package graph / exported surface

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

`loc` does not require `package.yaml` or `build.fol`.

## External packages
External packages are imported through `pkg`:

```
use space: pkg = {"space"};
```

`pkg` imports are different from `loc` and `std`:

- the imported root is an installed package root
- that root must contain `package.yaml`
- that root must contain `build.fol`
- `package.yaml` provides metadata only
- `build.fol` defines dependencies and exports
- raw transport URLs do not appear in source code; package acquisition is separate
