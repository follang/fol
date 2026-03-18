# Handle API

Graph methods return typed handles. Each handle type exposes its own set of
methods for configuring relationships and behavior.

## Artifact

The `Artifact` handle is returned by `add_exe`, `add_static_lib`,
`add_shared_lib`, and `add_test`.

### `artifact.link`

Links another artifact as a dependency. The linker will include it.

```fol
var core = graph.add_static_lib({ name = "core", root = "src/core/lib.fol" });
var app  = graph.add_exe({ name = "app", root = "src/main.fol" });
app.link(core);
```

Equivalent to Zig's `artifact.linkLibrary(dep)`.

### `artifact.import`

Imports a module into this artifact's compilation scope.

```fol
var utils = graph.add_module({ name = "utils", root = "src/utils.fol" });
var app   = graph.add_exe({ name = "app", root = "src/main.fol" });
app.import(utils);
```

Equivalent to Zig's `artifact.root_module.addImport(name, module)`.

### `artifact.add_generated`

Declares that this artifact depends on a generated file being produced before
it can compile.

```fol
var schema = graph.add_codegen({
    kind   = "schema",
    input  = "schema/api.yaml",
    output = "gen/api.fol",
});
var app = graph.add_exe({ name = "app", root = "src/main.fol" });
app.add_generated(schema);
```

---

## Run

The `Run` handle is returned by `graph.add_run`. All `Run` methods are
chainable and return `Run`.

### `run.add_arg`

Appends a literal string argument to the run command.

```fol
var run = graph.add_run(app);
run.add_arg("--config").add_arg("config/default.toml");
```

### `run.add_file_arg`

Appends a generated file as a path argument.

```fol
var cfg = graph.copy_file({
    name   = "config",
    source = "config/defaults.toml",
    dest   = "gen/config.toml",
});
var run = graph.add_run(app);
run.add_file_arg(cfg);
```

Equivalent to Zig's `run.addFileArg(file)`.

### `run.add_dir_arg`

Appends a directory path as an argument.

```fol
var run = graph.add_run(app);
run.add_dir_arg("assets/");
```

### `run.capture_stdout`

Captures the standard output of the run command as a generated file.

```fol
var run    = graph.add_run(generator_tool);
var output = run.capture_stdout();
app.add_generated(output);
```

Returns a `GeneratedFile` handle. Equivalent to Zig's `run.captureStdOut()`.

### `run.set_env`

Sets an environment variable for the run command.

```fol
var run = graph.add_run(app);
run.set_env("LOG_LEVEL", "debug");
```

### `run.depend_on`

Makes this run step depend on another step completing first.

```fol
var codegen = graph.step("codegen");
var run     = graph.add_run(app);
run.depend_on(codegen);
```

---

## Step

The `Step` handle is returned by `graph.step`. All `Step` methods are
chainable and return `Step`.

### `step.depend_on`

Makes this step depend on another step.

```fol
var compile = graph.step("compile");
var bundle  = graph.step("bundle");
bundle.depend_on(compile);
```

### `step.attach`

Attaches a generated file production to this step. When the step runs, the
attached generated file is produced first.

```fol
var strip_tool = graph.add_system_tool({
    tool   = "strip",
    output = "gen/app.stripped",
});
var strip_step = graph.step("strip");
strip_step.attach(strip_tool);
```

---

## Install

The `Install` handle is returned by `graph.install`, `graph.install_file`,
and `graph.install_dir`.

### `install.depend_on`

Makes this install step depend on another step.

```fol
var check   = graph.step("check");
var install = graph.install(app);
install.depend_on(check);
```

---

## Dependency

The `Dependency` handle is returned by `graph.dependency`. It exposes the
public surface of another package.

### `dependency.module`

Resolves a named module from the dependency.

```fol
var dep    = graph.dependency("mylib", "local:../mylib");
var module = dep.module("core");
app.import(module);
```

### `dependency.artifact`

Resolves a named artifact from the dependency.

```fol
var dep = graph.dependency("mylib", "local:../mylib");
var lib = dep.artifact("mylib-static");
app.link(lib);
```

### `dependency.step`

Resolves a named step from the dependency.

```fol
var dep  = graph.dependency("mylib", "local:../mylib");
var step = dep.step("codegen");
app_step.depend_on(step);
```

### `dependency.generated`

Resolves a named generated output from the dependency.

```fol
var dep   = graph.dependency("mylib", "local:../mylib");
var types = dep.generated("types.fol");
app.add_generated(types);
```
