# build.fol

`build.fol` is a file-bound FOL compilation unit. It is the build specification
for a package.

## File-Bound vs Folder-Bound

Normal FOL packages are folder-bound: every `.fol` file in the package folder
shares one namespace. `build.fol` is the one exception.

Rules for `build.fol`:

- It is its own compilation unit — it does not see sibling `.fol` files
- It has one implicit build stdlib scope, exposing `.build()` and build-only
  handle methods
- It can define local helper `fun[]`, `pro[]`, and `typ` declarations
- Those local declarations are not exported to the package
- It must declare exactly one `pro[] build(): non` entry
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
pro[] build(): non = {
    var build = .build();
    var graph = build.graph();
    ...
}
```

- `pro[]` — procedure with no receivers
- no parameters
- return type `non`

The active build context is accessed explicitly through the ambient build-only
accessor:

```fol
.build()
```

`.build()` returns an opaque build-only handle. The handle type is not public
language surface and should not be named explicitly in source code. Graph access
is reached through `build.graph()`.

Missing entry, wrong signature, duplicate entries, the old injected graph
parameter form, or explicit `Graph` type syntax are compile errors.

## Local Helpers

`build.fol` can define helper functions visible only within itself:

```fol
fun[] make_lib(name: str, root: str): Artifact = {
    return .build().graph().add_static_lib({ name = name, root = root });
}

pro[] build(): non = {
    var build = .build();
    var graph = build.graph();
    var core = make_lib("core", "src/core/lib.fol");
    var io   = make_lib("io",   "src/io/lib.fol");
    var app  = graph.add_exe({ name = "app", root = "src/main.fol" });
    app.link(core);
    app.link(io);
    graph.install(app);
}
```

Helpers may call `.build()` ambiently, but they do not name a public build or
graph type in source.

## Package Control

`build.fol` is the only package control file.

Package metadata and direct dependencies are configured from inside
`pro[] build(): non` through the ambient build context.

## Capability Restrictions

The build executor enforces a capability model. Allowed operations:

- graph mutation (adding artifacts, steps, options)
- option reads (`.build().graph().standard_target()`, `.build().graph().standard_optimize()`, etc.)
- path joining and normalization (`.build().graph().path_from_root(...)`)
- basic string and container operations
- controlled file generation (`.build().graph().write_file(...)`, `.build().graph().copy_file(...)`)
- controlled process execution (`.build().graph().add_system_tool(...)`)

Forbidden operations (produce a compile or runtime error):

- arbitrary filesystem reads or writes
- network access
- wall clock access
- ambient environment variable access
- uncontrolled process execution

These restrictions ensure build graphs are deterministic and portable.
