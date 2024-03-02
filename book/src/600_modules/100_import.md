# Imports

An import declaration states that the source file containing the declaration depends on functionality of the imported package and enables access to exported identifiers of that package.

Syntax to import a library is:
```
use package_name: mod = { path }
```

There are two type of import declartions:
- system libraries
- local libraries

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

## URL libraries
Libraries can be directly URL imported:

```
use space: url = {"https://github.com/follang/std"};
```
