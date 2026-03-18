# build.fol

`build.fol` is a file-bound FOL compilation unit. It is the build specification
for a package.

## File-Bound vs Folder-Bound

Normal FOL packages are folder-bound: every `.fol` file in the package folder
shares one namespace. `build.fol` is the one exception.

Rules for `build.fol`:

- It is its own compilation unit — it does not see sibling `.fol` files
- It has one implicit import: the build stdlib (`fol/build`), providing `Graph`
  and all handle types
- It can define local helper `fun[]`, `pro[]`, and `typ` declarations
- Those local declarations are not exported to the package
- It must declare exactly one `pro[] build(graph: Graph): non` entry
- Additional `use` imports from the FOL stdlib are allowed

## Compilation Pipeline

`build.fol` goes through the full FOL compiler pipeline:

```
build.fol
    │
    ▼  stream → lexer → parser
    │
    ▼  fol-resolver   (build stdlib injected as ambient scope)
    │
    ▼  fol-typecheck  (handle types and method signatures validated)
    │
    ▼  fol-lower      (lowered IR produced)
    │
    ▼  fol-build executor  (IR executed against BuildGraph)
    │
    ▼  BuildGraph
```

The compiler rejects `build.fol` files that reference sibling source files,
use filesystem or network APIs, or contain more than one canonical entry.

## Canonical Entry

The entry must match exactly:

```fol
pro[] build(graph: Graph): non = {
    ...
}
```

- `pro[]` — procedure with no receivers
- parameter name `graph`, type `Graph`
- return type `non`

Missing entry, wrong signature, or duplicate entries are compile errors.

## Local Helpers

`build.fol` can define helper functions visible only within itself:

```fol
fun[] make_lib(graph: Graph, name: str, root: str): Artifact = {
    return graph.add_static_lib({ name = name, root = root });
}

pro[] build(graph: Graph): non = {
    var core = make_lib(graph, "core", "src/core/lib.fol");
    var io   = make_lib(graph, "io",   "src/io/lib.fol");
    var app  = graph.add_exe({ name = "app", root = "src/main.fol" });
    app.link(core);
    app.link(io);
    graph.install(app);
}
```

## package.yaml

Every package needs a `package.yaml` alongside `build.fol`:

```yaml
name: my-app
version: 1.0.0
```

The build system reads `name` and `version` from the manifest. Dependencies
declared in the manifest are made available to `build.fol` via
`graph.dependency(...)`.

## Capability Restrictions

The build executor enforces a capability model. Allowed operations:

- graph mutation (adding artifacts, steps, options)
- option reads (`graph.standard_target()`, `graph.standard_optimize()`, etc.)
- path joining and normalization (`graph.path_from_root(...)`)
- basic string and container operations
- controlled file generation (`graph.write_file(...)`, `graph.copy_file(...)`)
- controlled process execution (`graph.add_system_tool(...)`)

Forbidden operations (produce a compile or runtime error):

- arbitrary filesystem reads or writes
- network access
- wall clock access
- ambient environment variable access
- uncontrolled process execution

These restrictions ensure build graphs are deterministic and portable.
