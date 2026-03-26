# Graph API

`build.graph()` is the public access point to the build graph in `build.fol`.
All graph construction goes through method calls on the returned handle.

This layer is intentionally narrower than the whole build surface:

- package metadata belongs on `build.meta({...})`
- direct dependencies belong on `build.add_dep({...})`
- future dependency queries and output handles sit above raw graph mutation and
  should not collapse back into ad hoc string-only graph APIs

The canonical shape is:

```fol
pro[] build(): non = {
    var build = .build();
    build.meta({ name = "app", version = "0.1.0" });
    var graph = build.graph();
    ...
}
```

## Artifacts

### `graph.add_exe`

Adds an executable artifact.

```fol
var app = graph.add_exe({
    name     = "app",
    root     = "src/main.fol",
    fol_model = "std",   // spell this explicitly for each artifact
    target   = target,    // optional
    optimize = optimize,  // optional
});
```

Returns an `Artifact` handle.

Required fields: `name`, `root`.
Optional fields: `fol_model`, `target`, `optimize`.

`root` is the path to the entry-point `.fol` source file relative to the
package root.

`fol_model` selects the runtime capability tier for the artifact:

- `core`
  no heap, no OS/runtime services
- `alloc`
  heap-backed facilities, still no OS/runtime services
- `std`
  hosted/runtime services on top of `alloc`

If omitted today, `fol_model` behaves like `std`, but the recommended style is
to spell it explicitly so the artifact contract is visible in `build.fol`.

The important boundary is semantic and runtime-facing:

- `core` artifacts must not use heap-backed `str`, `vec`, `seq`, `set`, or
  `map`
- `core` artifacts may still use arrays, records, routines, control flow, and
  `defer`
- `alloc` artifacts may use heap-backed runtime types but not hosted services
- `std` artifacts are the only artifacts that may use hosted services such as
  `.echo(...)` and ordinary host-executed `run` / `test`

Current implementation note:

- `core` already means “no heap and no OS/runtime services” at the language and
  runtime-contract level
- `core` artifacts are still emitted through the current Rust backend pipeline
- that means `core` is ready for semantic/runtime restriction work now, but it
  should not yet be read as “finished embedded backend support”

Mixed-model example:

```fol
pro[] build(): non = {
    var build = .build();
    build.meta({ name = "workspace_tools", version = "0.1.0" });
    var graph = build.graph();
    var corelib = graph.add_static_lib({
        name = "corelib",
        root = "core/lib.fol",
        fol_model = "core",
    });
    var alloclib = graph.add_static_lib({
        name = "alloclib",
        root = "alloc/lib.fol",
        fol_model = "alloc",
    });
    var tool = graph.add_exe({
        name = "tool",
        root = "app/main.fol",
        fol_model = "std",
    });

    graph.install(tool);
    graph.add_run(tool);
};
```

### `graph.add_static_lib`

Adds a static library artifact.

```fol
var core = graph.add_static_lib({ name = "core", root = "src/core/lib.fol" });
```

Returns an `Artifact` handle.

Library and test artifact config records follow the same optional fields:

- `fol_model`
- `target`
- `optimize`

### `graph.add_shared_lib`

Adds a shared (dynamic) library artifact.

```fol
var sdk = graph.add_shared_lib({ name = "sdk", root = "src/sdk/lib.fol" });
```

Returns an `Artifact` handle.

### `graph.add_test`

Adds a test artifact.

```fol
var tests = graph.add_test({ name = "tests", root = "src/tests.fol" });
```

Returns an `Artifact` handle.

### `graph.add_module`

Adds a standalone module that can be imported by other artifacts.

```fol
var utils = graph.add_module({ name = "utils", root = "src/utils.fol" });
```

Returns a `Module` handle.

## Installation and Runs

### `graph.install`

Marks an artifact for installation.

```fol
graph.install(app);
```

Returns an `Install` handle.

Install projections use the selected install prefix. Artifact installs resolve
under the prefix by kind:

- executables under `bin/`
- libraries under `lib/`

### `graph.install_file`

Installs either a source-file handle or a generated output handle.

```fol
var defaults = graph.file_from_root("config/defaults.toml");
graph.install_file({ name = "defaults", source = defaults });
```

```fol
var cfg = graph.write_file({
    name = "cfg",
    path = "config/generated.toml",
    contents = "ok",
});

graph.install_file({ name = "generated-cfg", source = cfg });
```

Returns an `Install` handle.

### `graph.install_dir`

Installs a source directory handle.

```fol
var assets = graph.dir_from_root("assets");
graph.install_dir({ name = "assets", source = assets });
```

Returns an `Install` handle.

The install prefix is selected by frontend/workspace configuration, not by the
package itself. Changing the prefix should move projected install destinations
without changing `build.fol`.

### `graph.add_run`

Registers an artifact as a run target. Binds the default `run` step when only
one executable exists and no explicit `run` step has been registered.

```fol
var run = graph.add_run(app);
```

Returns a `Run` handle. See [Handle API](./300_handle_api.md) for `Run` methods.

## Steps

### `graph.step`

Creates a named custom step.

```fol
var docs = graph.step("docs");
var docs = graph.step("docs", "Generate documentation");
```

Returns a `Step` handle. See [Handle API](./300_handle_api.md) for `Step` methods.

Named steps are selectable on the command line:

```text
fol code build docs
fol code build --step docs
```

When a description is present, frontend step planning and unknown-step
diagnostics surface it as part of the available step catalog.

## Current Execution Semantics

Step execution is still serial today. The build graph keeps deterministic step
ordering and explicit dependency edges, but it does not claim parallel
execution yet.

Current reporting distinguishes:

- requested
- executed
- skipped-from-cache
- skipped-by-foreign-run-policy
- produced outputs

That reporting is intended for frontend summaries and tests. Produced outputs
now participate in step cache-key semantics, so generated-file changes can
invalidate dependent steps predictably.

## Options

### `graph.standard_target`

Reads the `-Dtarget` option. Returns a `Target` handle.

```fol
var target = graph.standard_target();
```

The value is provided at build time via `-Dtarget=x86_64-linux-gnu`. If no
value is provided, `target` resolves to the host target.

An optional config record is accepted:

```fol
var target = graph.standard_target({ default = "x86_64-linux-gnu" });
```

### `graph.standard_optimize`

Reads the `-Doptimize` option. Returns an `Optimize` handle.

```fol
var optimize = graph.standard_optimize();
```

The value is provided via `-Doptimize=release-fast`. Defaults to `debug` if
not set.

Valid values: `debug`, `release-safe`, `release-fast`, `release-small`.

### `graph.option`

Declares a named user option readable via `-D`.

```fol
var root_opt = graph.option({
    name    = "root",
    kind    = "path",
    default = "src/main.fol",
});
```

Returns a `UserOption` handle.

Required fields: `name`, `kind`.
Optional field: `default`.

Option kinds:

| Kind       | Description                    | CLI Example            |
|------------|--------------------------------|------------------------|
| `bool`     | Boolean flag                   | `-Dstrip=true`         |
| `int`      | Integer value                  | `-Djobs=4`             |
| `str`      | Arbitrary string               | `-Dprefix=/usr/local`  |
| `enum`     | One of a fixed set of strings  | `-Dbackend=llvm`       |
| `path`     | File or directory path         | `-Droot=src/main.fol`  |
| `target`   | Target triple                  | `-Dtarget=x86_64-linux-gnu` |
| `optimize` | Optimization mode              | `-Doptimize=release-fast`   |

## Generated Files

### `graph.write_file`

Declares a file to be written with static contents at build time.

```fol
var header = graph.write_file({
    name     = "version.h",
    path     = "gen/version.h",
    contents = "#define VERSION 1\n",
});
```

Returns a `GeneratedFile` handle.

### `graph.copy_file`

Declares a file to be copied from a source-file handle.

```fol
var template = graph.file_from_root("config/template.toml");
var cfg = graph.copy_file({
    name   = "config",
    source = template,
    dest   = "gen/config.toml",
});
```

Returns a `GeneratedFile` handle.

### `graph.add_system_tool`

Declares a system tool invocation that produces a file.

```fol
var packed = graph.add_system_tool({
    tool   = "wasm-pack",
    args   = ["build", "--target", "web"],
    output = "gen/app.wasm",
});
```

Returns a `GeneratedFile` handle.

The generated file is keyed by the output path. Use this handle with
`step.attach(...)` or `artifact.add_generated(...)`.

### `graph.add_codegen`

Declares a FOL codegen step.

```fol
var schema = graph.add_codegen({
    kind   = "schema",
    input  = "schema/api.yaml",
    output = "gen/api.fol",
});
```

Returns a `GeneratedFile` handle.

Codegen kinds: `fol-to-fol`, `schema`, `asset-preprocess`.

## Path Utilities

### `graph.file_from_root`

Returns a source-file handle rooted under the package root.

```fol
var cfg = graph.file_from_root("config/default.toml");
```

Useful when passing source files into `copy_file`, `install_file`, or
`run.add_file_arg`.

### `graph.dir_from_root`

Returns a source-dir handle rooted under the package root.

```fol
var assets = graph.dir_from_root("assets");
```

### `graph.build_root`

Returns the package root directory as an absolute path string.

```fol
var root = graph.build_root();
```

### `graph.install_prefix`

Returns the installation prefix. Defaults to the workspace install directory.

```fol
var prefix = graph.install_prefix();
```

## Dependencies

Direct dependencies are declared on `build.add_dep({...})`, not on `graph`.

See [Handle API](./300_handle_api.md) for querying modules, artifacts, steps,
and generated outputs from a `Dependency` handle.
