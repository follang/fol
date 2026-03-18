# FOL Build System Plan: `lang/execution/fol-build`

Last updated: 2026-03-18

## No Backwards Compatibility

This is a new project. There is no installed user base. There are no external consumers.
When something is replaced, the old thing is deleted. No shims, no fallbacks, no parallel
implementations, no deprecation periods, no migration warnings. If the new way is chosen,
the old way does not exist.

## Status

Round 1 (build reset) is complete. All legacy `def root`/`def build` paths are deleted.
The canonical build entry is `pro[] build(graph: Graph): non` and it is enforced.

Round 2 (fol-build execution crate) is the current work.

Round 2 slices:

- [x] Slice 1. Create `lang/execution/fol-build` and move graph/api/semantic code out of `fol-package`
- [x] Slice 2. Move runtime types and evaluator into `fol-build`
- [x] Slice 3. Add `BuildStdlibScope` — resolver/typechecker injection surface
- [x] Slice 4. Wire the build stdlib into the resolver (file-bound isolation)
- [x] Slice 5. Wire the build stdlib into the typechecker
- [x] Slice 6. Replace AST-walking evaluator with real lowered-IR executor
- [x] Slice 7. Expand build API to Zig parity (modules, artifact.link, run args, path utils)
- [x] Slice 8. Full control flow in `build.fol` (when, loop, helper routines)
- [x] Slice 9. Frontend integration (full pipeline, -D options, named step selection)
- [x] Slice 10. New example fixtures and regression coverage

---

## Core Decision

FOL uses a Zig-style build model. `build.fol` is a normal FOL program. It goes through
the full compiler pipeline: stream → lexer → parser → resolver → typecheck → lower.
The only difference from a regular FOL program is at the final step — instead of emitting
Rust code via `fol-backend`, it is executed against a `BuildGraph` via `fol-build`.

The build system lives in `lang/execution/fol-build`. All build execution code belongs
there. `fol-package` keeps only entry validation and package metadata loading.

## Zig Reference

Zig's build entry:

```zig
pub fn build(b: *std.Build) void {
    const exe = b.addExecutable(.{ .name = "app", .root_source_file = .{ .path = "src/main.zig" } });
    b.installArtifact(exe);
}
```

FOL equivalent:

```fol
pro[] build(graph: Graph): non = {
    var target   = graph.standard_target();
    var optimize = graph.standard_optimize();

    var app = graph.add_exe({
        name     = "app",
        root     = "src/main.fol",
        target   = target,
        optimize = optimize,
    });

    graph.install(app);
}
```

FOL copies the architecture, not the syntax.

## What `build.fol` Is

`build.fol` is a **file-bound** FOL program. Regular FOL code is folder-bound: a folder
is a package, all `.fol` files in that folder share one package scope. `build.fol` is the
one exception: it is a single file that is its own complete compilation unit. It does not
share a namespace with sibling `.fol` files in the package folder.

Rules for `build.fol`:

- Goes through the full FOL pipeline (lex → parse → resolve → typecheck → lower → build executor)
- Does NOT include sibling `.fol` files in its scope
- Has one implicit import: the build stdlib (`fol/build`), providing `Graph` and all handle types
- CAN define helper `fun[]`/`pro[]`/`typ` declarations local to itself
- Those helpers are NOT exported to the package — they exist only during build evaluation
- Must declare exactly one `pro[] build(graph: Graph): non` as the entry point
- Additional `use` imports are allowed for FOL stdlib utilities

## Pipeline

```
build.fol
   │
   ▼ (same as any .fol file)
fol-stream → fol-lexer → fol-parser → fol-package (entry validation only)
                                            │
                                            ▼ (same as regular code)
                                      fol-resolver → fol-typecheck → fol-lower
                                                                          │
                                                                          ▼ (different backend)
                                                                      fol-build (executor)
                                                                          │
                                                                          ▼
                                                                      BuildGraph
                                                                          │
                                                                          ▼
                                                                    fol-frontend
                                                               (build/run/test/check)
```

## `lang/execution/fol-build` File Structure

```
lang/execution/fol-build/
├── Cargo.toml
└── src/
    ├── lib.rs          re-exports: BuildSession, BuildGraph, BuildExecutionError
    ├── session.rs      BuildSession — top-level entry point for fol-frontend
    ├── executor.rs     BuildIrExecutor — executes lowered FOL IR against BuildApi
    ├── dispatch.rs     method call dispatch table (method name → BuildApi call)
    ├── context.rs      BuildExecutionContext — graph, package root, options, cli args
    ├── graph.rs        BuildGraph, all ID types, all edge types
    ├── api.rs          BuildApi — Rust-level graph mutation interface
    ├── stdlib.rs       BuildStdlibScope — resolver/typechecker injection surface
    ├── semantic.rs     BuildSemanticType, BuildSemanticMethodSignature, etc.
    ├── runtime.rs      BuildRuntimeValue, BuildRuntimeFrame, BuildRuntimeStmt
    ├── artifact.rs     BuildArtifactKind, artifact-level edges (link, import, add_generated)
    ├── step.rs         BuildStepKind, step planning, step attachments
    ├── dependency.rs   DependencyBuildHandle, DependencyBuildSurface
    ├── codegen.rs      CodegenRequest, SystemToolRequest
    ├── option.rs       BuildOptionKind, ResolvedBuildOptionSet, -D CLI parsing
    ├── native.rs       native artifact types
    └── error.rs        BuildExecutionError
```

## What Moves Out of `fol-package`

| Source (`fol-package`) | Destination (`fol-build`) |
|---|---|
| `build_graph.rs` | `graph.rs` |
| `build_api.rs` | `api.rs` |
| `build_semantic.rs` | `semantic.rs` |
| `build_runtime.rs` | `runtime.rs` |
| `build_eval.rs` | `eval.rs` → replaced by `executor.rs` |
| `build_step.rs` | `step.rs` |
| `build_artifact.rs` | `artifact.rs` |
| `build_dependency.rs` | `dependency.rs` |
| `build_codegen.rs` | `codegen.rs` |
| `build_option.rs` | `option.rs` |
| `build_native.rs` | `native.rs` |

`fol-package` keeps only:

- `build.rs` — `parse_package_build`, `PackageBuildDefinition`, `PackageBuildMode`
- `build_entry.rs` — `validate_parsed_build_entry`, `BuildEntrySignatureExpectation`
- `metadata.rs`, `session.rs`, `model.rs` — package metadata (unchanged)

## The Executor

The current `build_eval.rs` is an AST pattern matcher. It inspects the parsed AST of
`build.fol` and dispatches to `BuildApi` by recognizing statement shapes. It does not
execute FOL code — it reads FOL syntax.

The new executor in `executor.rs` receives **lowered FOL IR** from `fol-lower` and
executes it. It handles:

- `var` bindings — store result in a local frame
- method calls on `Graph` handle — dispatch to `BuildApi`
- method calls on artifact/step/run/install/dependency/generated-file handles — dispatch to handle methods
- `when` / `else` — conditional artifact and step registration
- `loop` — iterate to add multiple artifacts or steps
- helper `fun[]`/`pro[]` calls — execute helper bodies in a new frame
- method chaining — `.add_arg().add_file_arg().set_env()`

Capability model stays: arbitrary filesystem writes, network access, wall clock, and
uncontrolled process execution are forbidden during build evaluation.

## The Build Stdlib Scope

`stdlib.rs` produces a `BuildStdlibScope` that the resolver injects into `build.fol`
compilation. This is what makes `Graph`, `Artifact`, `Step`, etc. visible to the resolver
and typechecker as real types with real method signatures.

### Graph methods

```
standard_target(config?)    → Target
standard_optimize(config?)  → Optimize
option(config)              → UserOption
add_exe(config)             → Artifact
add_static_lib(config)      → Artifact
add_shared_lib(config)      → Artifact
add_test(config)            → Artifact
add_module(config)          → Module          ← new
step(name, description?)    → Step
add_run(artifact)           → Run
install(artifact)           → Install
install_file(path)          → Install
install_dir(path)           → Install
write_file(config)          → GeneratedFile
copy_file(config)           → GeneratedFile
add_system_tool(config)     → GeneratedFile
add_codegen(config)         → GeneratedFile
dependency(alias, package)  → Dependency
path_from_root(subpath)     → str             ← new
build_root()                → str             ← new
install_prefix()            → str             ← new
```

### Artifact methods

```
link(dep_artifact)          → non             ← new (Zig: artifact.linkLibrary)
import(dep_module)          → non             ← new (Zig: artifact.root_module.addImport)
add_generated(gen_file)     → non             ← new
```

### Run methods

```
add_arg(value)              → Run             ← new, chainable (Zig: run.addArg)
add_file_arg(gen_file)      → Run             ← new, chainable (Zig: run.addFileArg)
add_dir_arg(path)           → Run             ← new, chainable
capture_stdout()            → GeneratedFile   ← new (Zig: run.captureStdOut)
set_env(key, value)         → Run             ← new, chainable (Zig: run.setEnvironmentVariable)
depend_on(step)             → Run             existing, chainable
```

### Step methods

```
depend_on(step)             → Step            existing, chainable
attach(gen_file)            → Step            ← new (attach generated file production to step)
```

### Install methods

```
depend_on(step)             → Install         existing, chainable
```

### Dependency methods

```
module(name)                → DependencyModule
artifact(name)              → DependencyArtifact
step(name)                  → DependencyStep
generated(name)             → DependencyGeneratedOutput
```

## -D CLI Options

Zig: `zig build -Dtarget=x86_64-linux-gnu -Doptimize=ReleaseFast -Denable-logs=true`

FOL: `fol code build -Dtarget=x86_64-linux-gnu -Doptimize=release-fast -Denable-logs=true`

`graph.standard_target()` reads `-Dtarget`. `graph.standard_optimize()` reads `-Doptimize`.
`graph.option({ name = "enable-logs", kind = bool, default = false })` reads `-Denable-logs`.

Option kinds: `bool`, `int`, `str`, `enum`, `path`, `target`, `optimize`.

## Step Selection

```sh
fol code build              # runs install steps (default)
fol code build docs         # runs the "docs" step
fol code run                # runs the default run step
fol code run --step serve   # runs the "serve" step
fol code test               # runs test steps
fol code check              # runs check steps
```

## Example: Full-Featured `build.fol`

```fol
fun[] make_lib(graph: Graph, name: str, root: str): Artifact = {
    return graph.add_static_lib({ name = name, root = root });
}

pro[] build(graph: Graph): non = {
    var target   = graph.standard_target();
    var optimize = graph.standard_optimize();
    var strip    = graph.option({ name = "strip", kind = bool, default = false });

    var core = make_lib(graph, "core", "src/core/lib.fol");
    var io   = make_lib(graph, "io",   "src/io/lib.fol");

    var app = graph.add_exe({
        name     = "app",
        root     = "src/main.fol",
        target   = target,
        optimize = optimize,
    });
    app.link(core);
    app.link(io);
    graph.install(app);

    var run = graph.add_run(app);
    run.add_arg("--config").add_file_arg(graph.path_from_root("config/default.toml"));

    when(target == "wasm32") {
        var wasm_step = graph.step("wasm-pack", "Package WASM output");
        var pack = graph.add_system_tool({
            tool    = "wasm-pack",
            args    = ["build", "--target", "web"],
            outputs = ["pkg/app.wasm"],
        });
        wasm_step.attach(pack);
    };

    var docs_step = graph.step("docs", "Generate documentation");
    var docs = graph.add_system_tool({
        tool    = "doc-gen",
        args    = ["src/", "--out", "docs/"],
        outputs = ["docs/index.html"],
    });
    docs_step.attach(docs);
}
```

## Example Fixtures to Add

| Fixture | Demonstrates |
|---|---|
| `examples/exe_with_options/` | target + optimize + boolean option |
| `examples/multi_lib/` | multiple libs, helper function, `artifact.link` |
| `examples/codegen/` | `add_codegen` + `artifact.add_generated` |
| `examples/workspace/` | multi-package, `dependency` + `artifact.import` |
| `examples/custom_steps/` | named steps + `step.attach` |
| `examples/run_args/` | `run.add_arg`, `run.add_file_arg`, `run.capture_stdout` |
| `examples/conditional/` | `when` selecting platform-specific artifacts |

## Slices

### Slice 1 — Create `fol-build`, move graph/api/semantic

Create `lang/execution/fol-build/Cargo.toml` and `src/lib.rs`.
Move `build_graph.rs`, `build_api.rs`, `build_semantic.rs` into the new crate unchanged.
Update `fol-package` to depend on `fol-build`.

Exit criteria: `cargo build` passes, all tests pass.

### Slice 2 — Move runtime types and evaluator

Move `build_runtime.rs`, `build_eval.rs`, `build_step.rs`, `build_artifact.rs`,
`build_dependency.rs`, `build_codegen.rs`, `build_option.rs`, `build_native.rs`.
Create `error.rs`, `context.rs`, `session.rs` with stub implementations.

`BuildExecutionContext`:
```rust
pub struct BuildExecutionContext {
    pub graph: BuildGraph,
    pub package_root: PathBuf,
    pub install_prefix: Option<PathBuf>,
    pub resolved_options: ResolvedBuildOptionSet,
    pub cli_args: Vec<(String, String)>,
}
```

`BuildSession`:
```rust
pub struct BuildSession { context: BuildExecutionContext }
impl BuildSession {
    pub fn new(package_root: PathBuf, cli_args: Vec<(String, String)>) -> Self
    pub fn execute(&mut self, lowered: &LoweredBuildProgram) -> Result<BuildGraph, BuildExecutionError>
    pub fn run_step(&self, step_name: &str) -> Result<(), BuildExecutionError>
}
```

Exit criteria: `fol-package` contains only entry validation and metadata. `cargo build` passes.

### Slice 3 — `BuildStdlibScope`

Create `stdlib.rs`. Produce `BuildStdlibScope::canonical()` covering all Graph methods,
all handle methods, all new methods listed above. Every entry has: receiver type, method
name, parameter list (required/optional/variadic), return type.

Unit tests: every method signature has correct receiver, return type, required params.

Exit criteria: `BuildStdlibScope::canonical()` is complete and tested.

### Slice 4 — Wire stdlib into resolver (file-bound)

When the resolver encounters a file flagged as `ParsedSourceUnitKind::Build`:
- Do not walk sibling `.fol` files
- Inject `BuildStdlibScope::canonical()` as the ambient scope
- `Graph` resolves to `BuildSemanticType::graph()`
- Method calls on build handles resolve via `BuildSemanticMethodSignature` dispatch

Exit criteria:
- `build.fol` with wrong method name produces a resolver error listing available methods
- Sibling `.fol` files are not visible to `build.fol`
- Helper `fun[]` declarations in `build.fol` are visible to the build entry

### Slice 5 — Wire stdlib into typechecker

All build types are recognized. Method call argument types are validated. Record literals
passed to build API calls are structurally validated (required fields present, unknown
fields rejected). Return types of method calls match what the stdlib scope declares.

Exit criteria:
- `var target = graph.standard_target()` typechecks
- `graph.add_exe({ name = "app", root = "src/main.fol", target = target })` typechecks
- Missing required field `root` in `add_exe` → typecheck error
- Passing `Artifact` where `Step` is expected → typecheck error

### Slice 6 — Real lowered-IR executor

Replace the AST-walking evaluator in `build_eval.rs` with `executor.rs`. The executor
receives lowered FOL IR from `fol-lower` and executes it. Delete `build_eval.rs`.

Supported in executor:
- `var` bindings
- method calls on `Graph` and all handle types
- `when` / `else`
- `loop`
- helper `fun[]`/`pro[]` calls (recursive frame execution)
- method chaining

Exit criteria:
- All current build fixtures produce the same `BuildGraph` as before
- `build.fol` with `when` executes conditionally
- `build.fol` with a helper `fun[]` executes the helper correctly
- The old AST evaluator is deleted

### Slice 7 — Expand build API (Zig parity)

Add all new methods listed in the stdlib section. Each new method needs:
- Graph IR representation (new edge type if needed)
- `BuildApi` method in `api.rs`
- Entry in `BuildStdlibScope` (resolver + typechecker sees it)
- Dispatch case in `dispatch.rs` (executor routes to it)
- Unit test in `fol-build`
- At least one fixture using it

New graph IR additions:
- `BuildRunConfig` — stores args, env vars, capture target for run steps
- `ArtifactLinkEdge` — artifact depends on another artifact
- `ModuleImportEdge` — artifact imports a module
- `StepAttachment` — step owns a generated file production

### Slice 8 — Full control flow in `build.fol`

`when`, `else`, `loop` work in `build.fol`. Helper routines defined in `build.fol` can
be called from the build entry and from other helpers.

Exit criteria:
- Fixture `examples/conditional/` conditionally adds a wasm artifact via `when`
- Fixture `examples/multi_lib/` uses a helper `fun[] make_lib(...)` called from `build`
- A loop over a sequence in `build.fol` adds multiple artifacts correctly

### Slice 9 — Frontend integration

`fol-frontend` uses `BuildSession` from `fol-build` instead of calling `fol-package`'s
evaluator. `build.fol` is compiled through the full pipeline before execution.

CLI option parsing: `-Dname=value` pairs are passed to `BuildSession::new` as `cli_args`.
Named step selection: `fol code build <step>` selects the named step from the graph.

Exit criteria:
- `fol code build` works end-to-end with the new pipeline
- `fol code run` works
- `fol code test` works
- `fol code build -Dtarget=x86_64-linux-gnu` passes the option into `standard_target()`
- `fol code build docs` executes the "docs" step

### Slice 10 — Fixtures and regression coverage

Add all fixtures listed in the fixtures table. Add integration tests in `test/app/build/`:
- `test_conditional_artifact` — `when` selects different artifacts
- `test_helper_routine` — helper function used from `build`
- `test_run_args` — `add_arg`, `add_file_arg`, `capture_stdout`
- `test_artifact_link` — `artifact.link(dep.artifact(...))`
- `test_module_import` — `artifact.import(dep.module(...))`
- `test_codegen_artifact` — `add_codegen` + `artifact.add_generated`
- `test_custom_step` — named step + `step.attach`
- `test_d_options` — `graph.option(...)` + CLI `-D` flags
- `test_path_utils` — `path_from_root`, `build_root`

Negative tests:
- Wrong method name on `Artifact` → resolver error
- `artifact.link` with wrong handle type → typecheck error
- `when` condition is non-bool → typecheck error
- `build.fol` with no canonical entry → clear error
- `build.fol` with two canonical entries → clear error

Exit criteria: all fixtures pass, all negative tests produce correct errors.

## Success Definition

Done when all of this is true:

- `lang/execution/fol-build` contains all build execution code
- `fol-package` contains only entry validation and package metadata
- `build.fol` goes through the full compiler pipeline before execution
- `when`, `loop`, and helper routines work in `build.fol`
- All new API methods (link, import, run args, path utils) are implemented and tested
- `-D` CLI options work end-to-end
- Named step selection works
- All fixtures in `examples/` demonstrate a distinct build capability
- All integration tests pass
