# Runtime Models

This document is the canonical runtime-tier matrix for FOL.

`fol_model` is selected per artifact in `build.fol`.

It is not:

- a source-file pragma
- an import convention
- a compatibility mode

It is:

- a semantic capability boundary
- a backend/runtime linkage boundary

## Tiers

### `core`

Meaning:

- no heap
- no OS/runtime services

Allowed language surface:

- scalars: `int`, `flt`, `bol`, `chr`
- arrays: `arr[...]`
- records, entries, aliases
- routines and method sugar
- control flow
- `defer`
- `opt[...]`, `err[...]`
- array `.len(...)`
- `panic(...)`

Forbidden surface:

- `str`
- `vec[...]`
- `seq[...]`
- `set[...]`
- `map[...]`
- `.echo(...)`
- hosted process-entry assumptions

### `alloc`

Meaning:

- heap-backed runtime facilities
- still no hosted OS/runtime services

Adds:

- `str`
- `vec[...]`
- `seq[...]`
- `set[...]`
- `map[...]`
- dynamic/string `.len(...)`

Still forbidden:

- `.echo(...)`
- hosted `run` / `test` execution semantics
- process/console/filesystem/network services

### `std`

Meaning:

- hosted runtime services on top of `alloc`

Adds:

- `.echo(...)`
- hosted process outcome behavior
- ordinary host-executed `run` / `test`
- future OS/runtime services

## Current implementation status

Already enforced semantically:

- `.echo(...)` requires `std`
- `str`, `vec`, `seq`, `set`, and `map` are rejected in `core`
- array `.len(...)` stays valid in `core`
- dynamic/string `.len(...)` requires `alloc` or `std`
- routed `run` / `test` reject non-`std` artifacts

Still in progress:

- turning `fol-runtime` into the one model crate with internal
  `core` / `alloc` / `std` ownership
- backend linkage by runtime tier
- deleting the old unsplit runtime ownership path

## Build example

```fol
pro[] build(graph: Graph): non = {
    var corelib = graph.add_static_lib({
        name = "corelib",
        root = "src/core/lib.fol",
        fol_model = "core",
    });

    var heaplib = graph.add_static_lib({
        name = "heaplib",
        root = "src/alloc/lib.fol",
        fol_model = "alloc",
    });

    var tool = graph.add_exe({
        name = "tool",
        root = "src/main.fol",
        fol_model = "std",
    });
}
```
