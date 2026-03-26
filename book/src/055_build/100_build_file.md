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

## Ambient Build API

The canonical build shape is:

```fol
pro[] build(): non = {
    var build = .build();

    build.meta({
        name = "app",
        version = "0.1.0",
        kind = "exe",
    });

    build.add_dep({
        alias = "json",
        source = "pkg",
        target = "json",
    });

    var graph = build.graph();
    var app = graph.add_exe({
        name = "app",
        root = "src/main.fol",
    });
    graph.install(app);
    graph.add_run(app);
}
```

The public layering is:

- `.build()` for package-level build context
- `build.meta({...})` for package metadata
- `build.add_dep({...})` for one direct dependency
- `build.graph()` for artifact and step graph work

The public surface includes:

- dependency handles returned from `build.add_dep({...})`
- unified output handles for generated and copied files
- explicit dependency build arguments
- a cleaner install-prefix model

Build/cache internals and installed outputs are separate:

- build root for emitted and intermediate build artifacts
- cache roots for reusable local state
- install prefix for user-visible installed outputs

Do not put package metadata directly on the graph handle.

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

### `build.meta({...})`

`build.meta({...})` configures package metadata for the current package.

Typical fields belong here:

- `name`
- `version`
- `kind`
- `description`
- `license`

This is package identity and package description data.
It is not graph mutation.

### `build.add_dep({...})`

`build.add_dep({...})` registers one direct dependency of the current package.

Typical fields belong here:

- `alias`
- `source`
- `target`
- `mode`
- `args`

For example:

```fol
build.add_dep({
    alias = "shared",
    source = "loc",
    target = "../shared",
});

build.add_dep({
    alias = "json",
    source = "pkg",
    target = "json",
    mode = "lazy",
});
```

Supported dependency modes:

- `eager`
- `lazy`
- `on-demand`

The public mode is preserved as declared. Some deeper dependency preparation is
still earlier-stage internally today, so treat the current runtime behavior as
honest-but-not-yet-maximally-lazy.

Forwarded dependency args stay explicit:

```fol
var graph = build.graph();
var target = graph.standard_target();
var optimize = graph.standard_optimize();
var fast = graph.option({ name = "use_fast_parser", kind = "bool", default = true });

build.add_dep({
    alias = "json",
    source = "pkg",
    target = "json",
    mode = "lazy",
    args = {
        target = target,
        optimize = optimize,
        use_fast_parser = fast,
        jobs = 4,
        flavor = "strict",
    },
});
```

This declares direct dependencies only.
Transitive dependencies stay declared in each dependency package's own
`build.fol`.

Nothing is forwarded implicitly from the parent build. If a dependency should
see `target`, `optimize`, or a package-specific option, pass it explicitly in
`args`.

### `build.graph()`

`build.graph()` returns the opaque graph handle used for artifact and step
construction.

Graph-only work belongs here:

- `add_exe`
- `add_static_lib`
- `install`
- `add_run`
- `standard_target`

Named steps may also carry an optional description:

```fol
var docs = graph.step("docs", "Generate documentation");
```
- `standard_optimize`
- `write_file`

That split keeps the build surface clear:

- package metadata through `build.meta`
- direct dependencies through `build.add_dep`
- artifact graph mutation through `build.graph()`

## Capability Restrictions

The build executor enforces a capability model. Allowed operations:

- graph mutation (adding artifacts, steps, options)
- option reads (`.build().graph().standard_target()`, `.build().graph().standard_optimize()`, etc.)
- source-path handles (`.build().graph().file_from_root(...)`, `.build().graph().dir_from_root(...)`)
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
