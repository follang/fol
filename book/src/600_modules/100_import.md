# Imports

An import declaration states that the source file containing the declaration depends on functionality of the imported package and enables access to exported identifiers of that package.

Syntax to import a library is:
```
use package_name: mod = { source }
```

There are two type of import declartions:
- system libraries
- local libraries

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
use warn std = {"fmt/log.warn"};

pro[] main: int = {
    warn("Last warning!...")
}
```
## Local libraries
To include a local package (example, package name `bender`), then we include the folder where it is, followed by the package name (folder is where files are located, package is the name defned with mod[])

```
use bend: loc = {"../folder/bender"};
```
Then to acces only a namespace:
```
use space: loc = {"../folder/bender/space"};
```

That second form is namespace import, not "single file import".
If `space` contains multiple `.fol` files in the same folder, they still belong to the same imported namespace.

## URL libraries
Libraries can be directly URL imported:

```
use space: url = {"https://github.com/follang/std"};
```
