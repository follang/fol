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

Recommended style:

- always spell `fol_model` explicitly in `build.fol`
- treat `core`, `alloc`, and `std` as contract choices, not convenience labels
- do not treat `std` as the informal baseline just because the current backend
  emits hosted Rust

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

Allowed example:

```fol
fun[] checksum(values: arr[int, 3]): int = {
    return .len(values) + values[0];
};
```

Forbidden example:

```fol
fun[] label(): str = {
    return "core-nope";
};
```

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

Allowed example:

```fol
fun[] label(prefix: str, extras: ... str): str = {
    return prefix + extras[0];
};
```

Forbidden example:

```fol
fun[] main(): int = {
    .echo("alloc-nope");
    return 0;
};
```

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

Allowed example:

```fol
fun[] main(): int = {
    var shown: int = .echo(9);
    return 9;
};
```

Forbidden example:

```fol
ali CounterPtr: ptr[int];
```

`std` widens runtime capability, but it does not opt into later V3/V4 language
surfaces automatically.

## Quick selection rule

- pick `core` first if the artifact can stay array-only and no-heap
- move to `alloc` only when you actually need `str` or dynamic containers
- move to `std` only when you actually need hosted runtime behavior

The intent is to keep capability growth explicit. `std` is not the semantic
baseline for every artifact just because the current backend is hosted Rust.

## Choose your model

Use `core` when the artifact can stay array-only and fixed-shape:

```fol
var graph = .build().graph();
graph.add_static_lib({
    name = "math",
    root = "src/lib.fol",
    fol_model = "core",
});
```

Move to `alloc` when the artifact itself genuinely needs heap-backed strings or
dynamic containers:

```fol
var graph = .build().graph();
graph.add_static_lib({
    name = "text",
    root = "src/lib.fol",
    fol_model = "alloc",
});
```

Move to `std` only when the artifact itself needs hosted behavior such as
`.echo(...)` or routed host execution:

```fol
var graph = .build().graph();
graph.add_exe({
    name = "tool",
    root = "src/main.fol",
    fol_model = "std",
});
```

Direct boundary reminder:

- a `core` artifact must not declare `str`, `seq`, `vec`, `set`, or `map`
- an `alloc` artifact must not call `.echo(...)`

Transitive boundary reminder:

- a `core` artifact still cannot consume heap-backed API from an `alloc`
  dependency
- an `alloc` artifact still cannot consume hosted-only API from a `std`
  dependency
- a `std` artifact may consume both `core` and `alloc` dependencies in one
  graph

## Transitive boundary rule

Capability legality is checked at the consuming artifact boundary, not only at
the dependency's own artifact boundary.

That means:

- a `core` artifact cannot consume heap-backed API from an `alloc` package just
  because the dependency itself was declared with `fol_model = "alloc"`
- a `core` or `alloc` artifact cannot reach `.echo(...)` indirectly through an
  imported `std` package
- a `std` artifact may consume `core` and `alloc` packages in the same graph

The consuming artifact model always wins.

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

## Runtime export contract

The backend should treat the three runtime modules as intentionally different
public surfaces.

- `fol_runtime::core`
  - no heap-backed types
  - no hosted hooks like `.echo(...)`
  - no hosted process-outcome helpers
- `fol_runtime::alloc`
  - heap-backed strings and dynamic containers
  - still no hosted hooks like `.echo(...)`
  - still no hosted process-outcome helpers
- `fol_runtime::std`
  - hosted hooks such as `.echo(...)`
  - hosted process-outcome helpers
  - alloc-tier heap types re-exported for host artifacts

Backend authors should not import a wider tier than the lowered artifact
actually requires. `core` emission should stay `core`-only. `alloc` emission
must not silently widen to `std`. `std` is the only tier that may rely on
hosted runtime entry and console hooks.

## Editor note

The editor should follow the same model split.

The intended contract is:

- LSP semantic diagnostics should come from the real compiler pipeline
- `fol_model` should affect editor diagnostics and completion the same way it
  affects `fol code build`
- tree-sitter grammar and structural capture layout stay hand-authored
- repetitive editor name lists are compiler-derived, not manually copied

This means adding a language feature should not require a second semantic
implementation in `fol-editor`. Only syntax-structure changes should normally
need targeted tree-sitter or editor UX updates.

## Build example

```fol
pro[] build(): non = {
    var build = .build();
    build.meta({ name = "mixed_models_workspace", version = "0.1.0" });
    var graph = build.graph();
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
- `examples/core_surface_showcase`
- `examples/alloc_defaults`
- `examples/alloc_containers`
- `examples/alloc_collections`
- `examples/alloc_surface_showcase`
- `examples/std_cli`
- `examples/std_bundled_fmt`
- `examples/std_echo_min`
- `examples/std_logtiny_git`
- `examples/std_named_calls`
- `examples/std_surface_showcase`
- `examples/mixed_models_workspace`

Negative example packages:

- `examples/fail_core_heap_reject`
- `examples/fail_alloc_echo`
- `examples/fail_core_alloc_boundary`
