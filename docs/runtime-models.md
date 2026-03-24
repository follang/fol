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

Choose `core` when:

- the artifact must avoid heap allocation completely
- the artifact should be valid for embedded-first targets
- arrays and plain records are enough
- console/process services are not part of the contract

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

Choose `alloc` when:

- the artifact needs strings or dynamic containers
- the artifact still should not depend on hosted OS/runtime services
- you want to keep heap usage explicit in `build.fol`

### `std`

Meaning:

- hosted runtime services on top of `alloc`

Adds:

- `.echo(...)`
- hosted process outcome behavior
- ordinary host-executed `run` / `test`
- future OS/runtime services

Choose `std` when:

- the artifact is a normal host tool or CLI
- the artifact needs `.echo(...)`
- the artifact is expected to run through `fol code run` or routed `test`

## Quick selection rule

- pick `core` first if the artifact can stay array-only and no-heap
- move to `alloc` only when you actually need `str` or dynamic containers
- move to `std` only when you actually need hosted runtime behavior

The intent is to keep capability growth explicit. `std` is not the semantic
baseline for every artifact just because the current backend is hosted Rust.

## Guarantees by model

| Model   | Heap | Hosted runtime | Typical artifact shape |
|---------|------|----------------|------------------------|
| `core`  | no   | no             | embedded logic, fixed-shape libs |
| `alloc` | yes  | no             | heap utilities, container-heavy libs |
| `std`   | yes  | yes            | CLIs, host tools, integration executables |

## Current implementation status

Implemented today:

- `.echo(...)` requires `std`
- `str`, `vec`, `seq`, `set`, and `map` are rejected in `core`
- array `.len(...)` stays valid in `core`
- dynamic/string `.len(...)` requires `alloc` or `std`
- routed `run` / `test` reject non-`std` artifacts
- emitted Rust imports the matching `fol_runtime::{core,alloc,std}` module
- `fol-runtime` is the single runtime crate with internal `core` / `alloc` /
  `std` ownership

## Editor note

The editor should follow the same model split.

The intended contract is:

- LSP semantic diagnostics should come from the real compiler pipeline
- `fol_model` should affect editor diagnostics and completion the same way it
  affects `fol code build`
- tree-sitter grammar stays hand-authored
- repetitive editor name lists should be compiler-derived instead of manually
  copied

This means adding a language feature should not require a second semantic
implementation in `fol-editor`, but syntax assets and editor UX may still need
targeted updates where structure changes.

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

## Example packages

- `examples/core_blink_shape`
- `examples/core_defer`
- `examples/core_records`
- `examples/alloc_defaults`
- `examples/alloc_containers`
- `examples/alloc_collections`
- `examples/std_cli`
- `examples/std_echo_min`
- `examples/std_named_calls`
- `examples/mixed_models_workspace`
