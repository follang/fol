# Build System

FOL uses a Zig-style build model. The build specification is a normal FOL
program called `build.fol`. It goes through the full compiler pipeline and
is executed against a build graph IR instead of emitting backend code.

The build system lives in `lang/execution/fol-build`. It handles:

- graph IR construction for artifacts, steps, options, modules, and generated files
- full control flow in build programs (when, loop, helper routines)
- `-D` CLI option passing into the build program
- named step selection at the command line

## Entry Point

Every buildable package must have a `build.fol` at its root with exactly one
canonical entry:

```fol
pro[] build(): non = {
    ...
}
```

The active build graph is accessed explicitly through the build-only ambient
accessor:

```fol
.graph()
```

There is no injected `graph` parameter anymore. `.graph()` returns an opaque
build-only handle. Users do not name its type explicitly. All build operations
still go through methods on that returned handle and on the handles it returns.

## Minimal Example

```fol
pro[] build(): non = {
    var graph = .graph();
    var app = graph.add_exe({ name = "app", root = "src/main.fol" });
    graph.install(app);
    graph.add_run(app);
}
```

This registers an executable, marks it for installation, and binds a default
run step.

## What `fol-build` Owns

- `graph.rs` — build graph IR (steps, artifacts, modules, options, generated files)
- `api.rs` — Rust-level graph mutation interface
- `semantic.rs` — method signatures and type info for the resolver and typechecker
- `stdlib.rs` — `BuildStdlibScope`: the ambient scope injected into `build.fol`
- `executor.rs` — executes the lowered FOL IR against the build graph
- `eval.rs` — evaluate a `build.fol` from source; entry point for `fol-package`
- `option.rs` — build option kinds, target triples, optimize modes
- `runtime.rs` — runtime representation of artifacts, generated files, step bindings
- `step.rs` — step planning, ordering, cache keys, execution reports
- `codegen.rs` — system tool and codegen request types
- `artifact.rs` — artifact pipeline definitions and output types
- `dependency.rs` — inter-package dependency surfaces

Use this section for:

- understanding the shape of `build.fol`
- the full graph API reference
- control flow available inside `build.fol`
- build options and `-D` flags
- artifact types, modules, and generated files
