# FOL Build Plan: `build.fol` As The Real Entry Point

Last updated: 2026-03-17

Status at head:

- the implementation roadmap below is complete
- the repo now has the build-graph model, draft build API, evaluator/model
  layers, routed workspace command plumbing, docs refresh, and checked-in
  examples
- the product end state is **not** fully complete yet:
  - modern graph-backed `build.fol` execution is still not the live default
  - workspace `fol code build/run/test/check` still execute only
    compatibility-only packages and reject modern/hybrid entries explicitly
  - the checked-in `examples/` tree reflects the current compatibility surface,
    not the final graph-driven user experience

This plan replaces the previous editor-focused milestone.

The next active milestone is to turn `build.fol` from a very small package-control
file into the real build entry point for FOL projects, in the same broad spirit
that `build.zig` is the build entry point for Zig projects.

The goal is not to copy Zig token-for-token. The goal is to achieve the same
capability level through FOL syntax and FOL semantics.

## Why This Plan Exists

After scanning the current repo state:

- `build.fol` already exists and is mandatory for formal `pkg` roots.
- today it is only extracted for:
  - package dependencies via top-level `def name: pkg = "..."`,
  - exported roots via top-level `def name: loc = "..."`,
  - inert native-artifact placeholders like `header`, `object`,
    `static_lib`, and `shared_lib`.
- `fol-package` treats `build.fol` as ordinary FOL syntax, but only a narrow
  subset of top-level `def` declarations has package meaning during package
  preparation.
- `fol code check/build/run/test` currently drive compilation directly through
  frontend/backend commands, not through a real user-authored build graph.
- the current system has package loading, dependency fetching, lockfiles, CLI
  workflows, and backend artifact generation, but it does **not** yet have:
  - a first-class build graph,
  - user-declared build steps,
  - target/optimize option plumbing through `build.fol`,
  - generated-file steps,
  - install/run/test/docs steps declared in `build.fol`,
  - explicit artifact dependencies between build nodes,
  - a standard build API surface comparable to Zig’s build system.

## What Was Scanned Before Writing This Plan

Repo context reviewed:

- [`README.md`](../README.md)
- [`book/src/600_modules/100_import.md`](../book/src/600_modules/100_import.md)
- [`book/src/600_modules/200_blocks.md`](../book/src/600_modules/200_blocks.md)
- [`book/src/050_tooling/200_tool_commands.md`](../book/src/050_tooling/200_tool_commands.md)
- [`plan/PROGRESS.md`](./PROGRESS.md)
- [`lang/compiler/fol-package/src/build.rs`](../lang/compiler/fol-package/src/build.rs)
- [`lang/compiler/fol-package/src/model.rs`](../lang/compiler/fol-package/src/model.rs)
- [`lang/compiler/fol-package/src/session.rs`](../lang/compiler/fol-package/src/session.rs)
- parser and resolver tests that exercise current `build.fol` extraction and
  package/export semantics

External reference reviewed:

- Zig build-system documentation from the official Zig docs/site, with focus on:
  - `build.zig` as the build entry point
  - step graphs
  - artifact creation
  - standard target/optimization options
  - run/test/install/doc steps
  - dependency wiring
  - generated files and custom steps

## Zig Model We Want To Match

The build system we want from FOL should match Zig at the capability level in
these ways:

1. One build entry file per project root.
2. The entry file constructs a build graph rather than acting as inert metadata.
3. Artifacts are explicit nodes.
4. Build actions form named steps and dependencies.
5. Target and optimization settings are standard first-class options.
6. User options are part of the build graph, not ad-hoc CLI hacks.
7. Running, testing, installing, documenting, codegen, and custom actions are
   all expressible as graph nodes.
8. Dependencies can contribute modules/artifacts/steps into the consuming build.
9. Generated files and external tools can feed later build stages.
10. The CLI executes a requested step from that graph rather than bypassing it.

## FOL-Specific Constraints

FOL should not become “Zig with different keywords”.

The FOL version should preserve these constraints:

1. `build.fol` stays valid FOL syntax.
2. We do not introduce a separate build-only parser if ordinary FOL parsing can
   support the model.
3. The build API should look like ordinary FOL declarations/calls, not a YAML
   document or shell DSL.
4. Build evaluation must be deterministic and side-effect controlled.
5. Package metadata remains in `package.yaml`; `build.fol` owns build graph and
   export/build behavior.
6. The repo must preserve the current `loc/std/pkg` package model.

## Desired End State

At the end of this plan:

- `build.fol` is the canonical build entry point for package roots.
- `fol code build/run/test/check` resolve and execute build graph steps from
  `build.fol`.
- build declarations can create executable/library/test/doc/codegen/install/run
  nodes and wire dependencies between them.
- package exports move from “special extracted `def loc` records only” toward a
  coherent build-graph concept, while preserving the package import contract.
- the current narrow V1 package-control behavior still works during migration,
  then becomes a subset of the richer build model instead of a separate mode.

## Non-Goals For This Milestone

This plan does **not** require all future package-manager ambitions up front.

Specifically out of scope unless needed by a later phase:

- remote registries beyond current fetch/update/store roots
- cross-language package publishing UX
- full C/C++ toolchain abstraction parity on day one
- distributed build execution
- IDE build-graph visualization
- user-defined build graph plugins loaded from arbitrary native code

## Current Baseline

The repo already has these build-adjacent pieces:

### Package Control

- formal `pkg` roots require `package.yaml` and `build.fol`
- `build.fol` extraction currently recognizes:
  - dependency defs of type `pkg`
  - export defs of type `loc`
  - reserved native-artifact placeholder defs
- prepared export mounts are computed from build-declared export roots

### CLI Workflow

- `fol work ...` covers project/workspace setup and inspection
- `fol pack fetch/update` covers dependency materialization and lockfile handling
- `fol code check/build/run/test` covers compile pipeline driving

### Compiler Pipeline

- package loading
- resolver workspaces
- typechecking
- lowering
- backend artifact generation

### Missing Build-System Pieces

- no graph object
- no named user-defined steps
- no standard `target` / `optimize` build options
- no first-class “add executable/library/test” API
- no installed-artifact step model
- no generated file step model
- no build-time options/modules interface
- no package dependency handoff into build graph nodes

## Proposed FOL Build Surface

This plan assumes `build.fol` grows into a standard-library-backed build API.

The likely direction is:

```fol
use build: std = {"fol/build"};

def build(graph: build::Graph): void = {
    var target = graph.standard_target();
    var optimize = graph.standard_optimize();

    var app = graph.add_exe({
        name = "demo",
        root = "src/main.fol",
        target = target,
        optimize = optimize,
    });

    graph.install(app);

    var run_app = graph.add_run(app);
    graph.step("run", "Run the demo").depend_on(run_app);
}
```

The exact syntax is not locked by this plan. The capability set is.

## Concept Mapping: Zig To FOL

### Entry file

- Zig: `build.zig`
- FOL target: `build.fol`

### Build graph root

- Zig: `std.Build`
- FOL target: `fol/build::Graph` or equivalent

### Artifact creation

- Zig: `addExecutable`, `addStaticLibrary`, `addSharedLibrary`, `addTest`
- FOL target:
  - `graph.add_exe(...)`
  - `graph.add_static_lib(...)`
  - `graph.add_shared_lib(...)`
  - `graph.add_test(...)`

### Standard options

- Zig: `standardTargetOptions`, `standardOptimizeOption`
- FOL target:
  - `graph.standard_target()`
  - `graph.standard_optimize()`

### Step graph

- Zig: named/custom steps with dependencies
- FOL target:
  - `graph.step(name, description)`
  - `.depend_on(...)`

### Running artifacts

- Zig: `addRunArtifact`
- FOL target:
  - `graph.add_run(artifact)`

### Install surface

- Zig: install artifact/file/dir steps
- FOL target:
  - `graph.install(artifact)`
  - `graph.install_file(...)`
  - `graph.install_dir(...)`

### Generated files / custom tools

- Zig: generated files, system commands, custom steps
- FOL target:
  - `graph.add_write_file(...)`
  - `graph.add_codegen(...)`
  - `graph.add_system_tool(...)`
  - `graph.add_step(...)`

### Dependency handoff

- Zig: dependency packages expose modules/artifacts to the consumer build
- FOL target:
  - dependency roots expose build graph surfaces, importable modules, artifacts,
    and possibly named steps

## Architecture Decisions We Must Lock Early

These are not optional cleanup items. They are gating design choices.

### 1. What Executes `build.fol`?

Options:

1. Dedicated interpreter over parsed/typechecked/lowered build subsets.
2. Compile `build.fol` into an internal IR/VM and execute that.
3. Reuse backend/runtime execution and run build scripts as normal FOL code.

Planned direction:

- prefer a dedicated build evaluator over full general runtime execution
- keep the evaluator deterministic
- whitelist build APIs and I/O surfaces explicitly
- avoid making arbitrary ordinary runtime behavior the build contract too early

### 2. Is `build.fol` Just One Function Or A Broader Surface?

Options:

1. one required `def build(...)`
2. extracted top-level declarations plus one orchestrating entry
3. multiple named top-level build entry hooks

Planned direction:

- keep one canonical entry surface for graph construction
- allow helper defs/types/functions in the file
- preserve current extraction compatibility during migration

### 3. How Do Current Export Defs Migrate?

Current:

- `def root: loc = "src"`
- `def fmt: loc = "src/fmt"`

We need to decide whether exports become:

1. a compatibility shim on top of the new graph
2. a dedicated package-surface section in the build API
3. ordinary graph nodes marked for package export

Planned direction:

- keep current `def ...: loc = "..."` export declarations as compatibility input
  in Phase 1
- introduce a richer build-native export API later
- migrate resolver/package preparation onto the richer model only after both
  surfaces coexist

### 4. What Is The First-Class Unit Of Dependency Exposure?

Candidates:

- source roots
- modules
- named export surfaces
- artifacts
- generated outputs
- step handles

Planned direction:

- model dependency exposure as a structured build package handle that can provide:
  - modules/source roots,
  - artifacts,
  - named steps,
  - generated outputs,
  - build options metadata

## Milestones

## Phase 0: Repo Baseline And Vocabulary Lock

Purpose:

- stop mixing “package metadata”, “package exports”, “artifact building”, and
  “build graph” as if they were the same thing

Work:

1. Define repo-wide terms:
   - package root
   - build root
   - build entry point
   - module/source root
   - artifact
   - install root
   - step
   - dependency package
   - generated file
2. Document current-state `build.fol` behavior precisely from code.
3. Record all current CLI/build/package entrypoints that bypass `build.fol`.
4. Identify which existing `fol code ...` operations will become named default
   build steps versus direct compiler commands.

Exit criteria:

- one stable glossary in docs and code comments
- no ambiguity about current versus planned `build.fol` semantics

## Phase 1: Formalize `build.fol` V1 Compatibility Layer

Purpose:

- preserve the current package model while preparing the richer build system

Work:

1. Split “package control extraction” from “future build graph evaluation” in
   `fol-package`.
2. Introduce an explicit internal representation for:
   - compatibility exports,
   - compatibility dependencies,
   - compatibility native-artifact placeholders,
   - future graph entry node metadata.
3. Keep current `pkg` import behavior green while refactoring internal data
   structures.
4. Add tests that lock current compatibility behavior before expanding features.

Files likely touched:

- `lang/compiler/fol-package/src/build.rs`
- `lang/compiler/fol-package/src/model.rs`
- `lang/compiler/fol-package/src/session.rs`

Exit criteria:

- existing `pkg` import/export behavior remains unchanged
- package preparation owns a richer internal model than just three extracted lists

## Phase 2: Define The Build Graph IR

Purpose:

- create the internal model that the eventual `build.fol` evaluator will build

Work:

1. Design build graph IDs/tables:
   - package
   - step
   - artifact
   - module/root
   - generated file
   - option
   - install action
2. Define graph node categories:
   - executable
   - static library
   - shared library
   - test artifact
   - run action
   - install action
   - documentation action
   - custom/system command action
   - generated-file action
3. Define dependency edges:
   - step -> step
   - artifact -> artifact
   - artifact -> module/root
   - generated file -> artifact
4. Define graph validation rules:
   - no cycles where disallowed
   - missing source root diagnostics
   - invalid install destinations
   - invalid dependency exposure

Exit criteria:

- a standalone `BuildGraph` model exists
- validation can reject malformed graphs before execution

## Phase 3: Standard Build Library API Surface

Purpose:

- define the public FOL-side API that `build.fol` scripts will call

Work:

1. Create a standard build API namespace, likely under `std` / `core`.
2. Design first public graph methods:
   - `standard_target`
   - `standard_optimize`
   - `option`
   - `step`
   - `add_exe`
   - `add_static_lib`
   - `add_shared_lib`
   - `add_test`
   - `add_run`
   - `install`
   - `install_file`
   - `install_dir`
   - `dependency`
3. Decide how FOL syntax expresses structured build arguments:
   - records
   - builder methods
   - named arg records
4. Freeze the naming rules early to avoid churn in examples and docs.

Exit criteria:

- one draft public API exists
- examples can be written even before execution is implemented

## Phase 4: Build Evaluator

Purpose:

- execute `build.fol` as graph-construction code

Work:

1. Choose evaluator boundary:
   - parser/resolver/typecheck input
   - dedicated evaluated subset
   - no arbitrary unrestricted runtime execution
2. Define allowed build-time operations:
   - graph creation
   - option reads
   - path joins/normalization
   - string/container basics
   - controlled file generation
   - controlled process execution
3. Reject non-deterministic or unsupported ordinary language surfaces in
   `build.fol` with explicit diagnostics.
4. Surface build-evaluation diagnostics through `fol-diagnostics`.
5. Guarantee that evaluating the same `build.fol` and inputs yields the same
   graph.

Exit criteria:

- `build.fol` can construct a validated graph in-process
- graph-construction diagnostics are clear and source-located

## Phase 5: Artifact Model

Purpose:

- make artifact creation first-class instead of hardcoded CLI behavior

Work:

1. Define artifact kinds:
   - exe
   - static lib
   - shared lib
   - test bundle
   - generated source bundle
   - docs bundle
2. Define artifact configuration:
   - entry/root source
   - package roots/modules
   - target
   - optimize mode
   - output name
   - linkage mode
   - native artifacts
3. Connect artifacts to existing compiler pipeline:
   - `fol-package`
   - resolver
   - typechecker
   - lowerer
   - backend
4. Preserve emitted artifact reporting through the frontend.

Exit criteria:

- build graph nodes can produce backend artifacts through one shared path

## Phase 6: Step Execution Model

Purpose:

- turn the graph into something the CLI can execute

Work:

1. Define default steps:
   - `build`
   - `run`
   - `test`
   - `install`
   - `check`
2. Define custom named steps.
3. Define step dependency semantics and topological execution.
4. Add incremental step/result caching boundaries.
5. Add stable human/plain/json reporting for:
   - requested step
   - executed substeps
   - skipped cached substeps
   - produced artifacts

Exit criteria:

- the frontend can execute a named build step from `build.fol`

## Phase 7: Standard Target / Optimize / User Options

Purpose:

- match one of Zig’s most important build-system strengths

Work:

1. Define canonical target triple/config model for FOL.
2. Define canonical optimization modes.
3. Define user option declarations:
   - bool
   - int
   - string
   - enum
   - path
4. Wire options into:
   - build evaluation
   - CLI argument parsing
   - artifact configuration
5. Ensure options appear in `fol code ... --help` in a build-aware way.

Exit criteria:

- `build.fol` can configure target/optimize and user options without custom CLI
  one-offs

## Phase 8: Dependency Build Surfaces

Purpose:

- let one package’s build graph expose usable build products to another

Work:

1. Define dependency build handles.
2. Support dependency-provided:
   - modules/source roots
   - artifacts
   - named steps
   - generated outputs
3. Reconcile dependency build surfaces with current package export semantics.
4. Decide how dependency build scripts are evaluated:
   - eagerly,
   - lazily,
   - or per requested surface.
5. Preserve current `loc/std/pkg` source resolution rules while layering build
   surfaces on top.

Exit criteria:

- dependent packages can consume build-exposed artifacts/modules instead of only
  source exports

## Phase 9: Generated Files, Codegen, And External Tools

Purpose:

- reach practical parity with the useful middle of Zig’s build system

Work:

1. Add generated-file nodes.
2. Add write-file / copy-file / install-file helpers.
3. Add controlled system-tool invocation.
4. Add codegen step APIs for:
   - FOL-to-FOL generation
   - foreign schema/code generation
   - asset preprocessing
5. Define dependency tracking for generated outputs.

Exit criteria:

- generated build products can feed later artifact steps safely

## Phase 10: Native Artifacts And C ABI Work

Purpose:

- turn current inert placeholders into real build surfaces

Work:

1. Upgrade current placeholder records:
   - `header`
   - `object`
   - `static_lib`
   - `shared_lib`
2. Define include-path and library-path semantics.
3. Define linking semantics into backend-produced artifacts.
4. Decide whether compile-C / compile-C++ style steps are first-class in this
   milestone or deferred.
5. Lock cross-platform naming/path conventions.

Exit criteria:

- today’s native-artifact placeholders become executable build graph inputs

## Phase 11: CLI Migration

Purpose:

- make `fol code ...` actually honor `build.fol`

Work:

1. Define CLI fallback rules:
   - if `build.fol` has no modern build entry, use compatibility mode
   - if it has a build entry, execute graph steps
2. Map commands:
   - `fol code build`
   - `fol code run`
   - `fol code test`
   - `fol code check`
   onto default build steps
3. Add explicit step selection:
   - `fol code build --step docs`
   - similar targeted execution
4. Keep artifact reporting stable across migration.
5. Preserve human/plain/json output formats.

Exit criteria:

- ordinary user workflows go through `build.fol` by default

## Phase 12: Docs, Scaffolding, And Editor Surfaces

Purpose:

- make the new build model the documented normal path

Work:

1. Rewrite `book` sections on packages/imports/build roots.
2. Update `README.md` build/package descriptions.
3. Update project scaffolding so new packages generate the new `build.fol`
   entrypoint shape.
4. Add editor/LSP affordances for build files:
   - highlighting
   - completion
   - symbol extraction
   - diagnostics
5. Add sample projects:
   - simple exe
   - static lib
   - shared lib
   - generated file
   - dependency-consuming workspace

Exit criteria:

- a new user can learn and use the `build.fol` system from repo docs alone

## Testing Matrix

Each phase must add tests at the right layer.

### Parser / Package Tests

- valid build entry parsing
- compatibility export/dependency parsing
- invalid build declarations
- native artifact parsing

### Build Graph Tests

- graph validation
- cycle detection
- option handling
- step dependency ordering

### Evaluator Tests

- deterministic graph construction
- unsupported-surface diagnostics
- option propagation

### Frontend Tests

- `fol code build/run/test/check` using build graph
- human/plain/json reporting
- step selection

### Integration Tests

- single-package executable
- multi-package dependency build
- generated file feeding compile
- install surface
- native artifact linking when implemented

## Migration Strategy

We should not break current formal packages immediately.

The migration path should be:

1. keep current compatibility extraction
2. introduce modern build entrypoint behind coexistence
3. let CLI detect both modes
4. migrate scaffolding and docs
5. only then consider deprecating compatibility-only `build.fol` files

## Immediate Step Order

These are the exact next actions after approving this plan:

1. Write a short design note that freezes the vocabulary and current-state
   baseline.
2. Introduce a dedicated build-graph model crate/module or a strongly isolated
   internal module in `fol-package`.
3. Refactor current `build.fol` extraction into an explicit compatibility layer.
4. Draft the first public build API shape in docs/examples before coding the
   evaluator.
5. Implement build-graph validation before graph execution.
6. Implement a minimal evaluator that can:
   - read target/optimize options,
   - create one executable artifact,
   - install it,
   - define a `run` step.
7. Route `fol code build` through that minimal graph path.
8. Expand from there to tests, dependencies, generated files, and native
   artifacts.

## Round 1 Slice Tracker

This round focuses on Phase 1 compatibility-layer work in `fol-package`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Replace the old plan with this `build.fol` roadmap and lock the
   first implementation round.
2. `[complete]` Split raw V1 package-control data into an explicit compatibility
   sub-structure in `PackageBuildDefinition`.
3. `[complete]` Add compatibility accessors/helpers so callers stop depending on
   ad-hoc field layout.
4. `[complete]` Introduce modern build-entry metadata types in the package build
   model.
5. `[complete]` Detect canonical `build` entry declarations during `build.fol`
   extraction.
6. `[complete]` Thread build-entry metadata through prepared package state.
7. `[complete]` Add explicit build-mode classification for empty / compatibility /
   hybrid / modern build files.
8. `[complete]` Cover compatibility-plus-entry coexistence in package parser
   tests.
9. `[complete]` Cover prepared-package loading for modern-entry package roots.
10. `[complete]` Expose the richer package-build surface from `fol-package`'s
    public API.

## Round 2 Slice Tracker

This round focuses on Phase 2 build-graph IR work in `fol-package`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the second implementation round around the build-graph IR.
2. `[complete]` Introduce a dedicated `build_graph` module with stable graph ID types.
3. `[complete]` Add core graph node-kind enums for steps, artifacts, modules, generated files, options, and installs.
4. `[complete]` Add `BuildGraph` storage tables plus allocation helpers for each node family.
5. `[complete]` Add step-dependency edges and graph APIs for explicit step-to-step dependencies.
6. `[complete]` Add artifact-input edges for module and generated-file dependencies.
7. `[complete]` Add graph validation error types and empty-graph validation entrypoints.
8. `[complete]` Validate step dependency cycles with source-local regression tests.
9. `[complete]` Validate artifact input references and install-target shape constraints.
10. `[complete]` Re-export the build-graph IR from `fol-package`'s public API.

## Round 3 Slice Tracker

This round focuses on Phase 3 standard build-library API work in `fol-package`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the third implementation round around the draft build API surface.
2. `[complete]` Introduce a dedicated `build_api` module with a graph-backed `BuildApi` entry type.
3. `[complete]` Add draft `standard_target` and `standard_optimize` request/response types.
4. `[complete]` Add generic user-option request types and graph-backed `option` helpers.
5. `[complete]` Add structured artifact argument records and stable build-name validation helpers.
6. `[complete]` Add draft `add_exe` / `add_static_lib` / `add_shared_lib` / `add_test` methods.
7. `[complete]` Add draft `step` and `add_run` API methods with graph-backed step wiring.
8. `[complete]` Add draft `install` / `install_file` / `install_dir` API methods.
9. `[complete]` Add a draft `dependency` request model and graph-backed placeholder surface.
10. `[complete]` Re-export the draft build API surface from `fol-package`.

## Round 4 Slice Tracker

This round focuses on Phase 4 build-evaluator work in `fol-package`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the fourth implementation round around deterministic build-plan evaluation.
2. `[complete]` Introduce a dedicated `build_eval` module with evaluation request/result shell types.
3. `[complete]` Add explicit evaluator-boundary and allowed-operation model types.
4. `[complete]` Add build-evaluation error types with diagnostic integration and source locations.
5. `[complete]` Add deterministic evaluation input modeling and stable determinism-key rendering.
6. `[complete]` Add draft build-evaluation operation types for the public build API surface.
7. `[complete]` Evaluate option operations into a graph-backed `BuildApi`.
8. `[complete]` Evaluate artifact, step, run, install, and dependency operations into a validated graph.
9. `[complete]` Reject unsupported operations and graph-validation failures with explicit evaluation diagnostics.
10. `[complete]` Re-export the draft build-evaluator surface from `fol-package`.

## Round 5 Slice Tracker

This round focuses on Phase 5 artifact-model work in `fol-package`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the fifth implementation round around a first-class artifact model.
2. `[complete]` Introduce a dedicated `build_artifact` module with artifact-definition shell types.
3. `[complete]` Add first-class artifact kind enums covering executable, libraries, tests, generated source, and docs bundles.
4. `[complete]` Add artifact root/source/module configuration records and linkage/output-name fields.
5. `[complete]` Add target/optimize/native-artifact attachment fields to artifact definitions.
6. `[complete]` Add artifact output/reporting model types for emitted crates, binaries, generated bundles, and docs bundles.
7. `[complete]` Add compiler-pipeline plan records that connect package, resolver, typecheck, lower, and backend stages to one artifact definition.
8. `[complete]` Add graph-to-artifact projection helpers for executable/library/test artifact nodes.
9. `[complete]` Add artifact-report summary helpers that preserve frontend-facing output strings and paths.
10. `[complete]` Re-export the draft artifact-model surface from `fol-package`.

## Round 6 Slice Tracker

This round focuses on Phase 6 step-execution-model work in `fol-package`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the sixth implementation round around build-step execution planning.
2. `[complete]` Introduce a dedicated `build_step` module with step-plan request/result shell types.
3. `[complete]` Add explicit default-step kind enums for build, run, test, install, and check.
4. `[complete]` Add custom step-definition and requested-step selector model types.
5. `[complete]` Add topological step-order planning helpers over graph step dependencies.
6. `[complete]` Add step cache-boundary and cache-key model types.
7. `[complete]` Add step execution report/event model types for requested, executed, skipped, and produced outputs.
8. `[complete]` Add graph-to-step projection helpers for default and custom graph steps.
9. `[complete]` Add stable step-report summary helpers aligned with frontend-facing reporting.
10. `[complete]` Re-export the draft step-execution surface from `fol-package`.

## Round 7 Slice Tracker

This round focuses on Phase 7 standard-option work in `fol-package` and `fol-frontend`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the seventh implementation round around canonical build-option modeling.
2. `[complete]` Introduce a dedicated `build_option` module with option-declaration shell types.
3. `[complete]` Add canonical target architecture, operating-system, environment, and triple parsing/rendering types.
4. `[complete]` Add canonical optimization-mode enums and frontend-profile mapping helpers.
5. `[complete]` Extend user option declarations and values to cover `int` and `path` kinds.
6. `[complete]` Add build-option override parsing and resolved-option-set lookup helpers.
7. `[complete]` Replay option declarations and CLI/input overrides through the build evaluator result.
8. `[complete]` Add artifact-target selection helpers that project resolved target/optimize values into artifact config.
9. `[complete]` Add frontend CLI/config build-option override surfaces for target, optimize, and repeated named options.
10. `[complete]` Re-export the draft build-option surface from `fol-package`.

## Round 8 Slice Tracker

This round focuses on Phase 8 dependency build surfaces in `fol-package`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the eighth implementation round around dependency build-surface modeling.
2. `[complete]` Introduce a dedicated `build_dependency` module with dependency-surface shell types.
3. `[complete]` Add dependency-provided module and source-root surface records.
4. `[complete]` Add dependency-provided artifact, step, and generated-output surface records.
5. `[complete]` Add dependency build-handle and dependency-surface collection types.
6. `[complete]` Add export-to-dependency bridge helpers that project prepared export mounts into dependency module surfaces.
7. `[complete]` Add dependency build-evaluation mode enums for eager, lazy, and on-demand surface loading.
8. `[complete]` Extend dependency requests/handles in the draft build API to carry declared build-surface collections.
9. `[complete]` Extend prepared-package modeling to retain optional dependency build-surface exports alongside compatibility exports.
10. `[complete]` Re-export the draft dependency build-surface model from `fol-package`.

## Round 9 Slice Tracker

This round focuses on Phase 9 generated files, codegen, and external tools in `fol-package`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the ninth implementation round around generated-file and codegen modeling.
2. `[complete]` Introduce a dedicated `build_codegen` module with generated-file shell types.
3. `[complete]` Add generated-file action definitions for write, copy, and captured-tool outputs.
4. `[complete]` Add generated-file install/helper projection types for install-file style flows.
5. `[complete]` Add controlled system-tool invocation request/result model types.
6. `[complete]` Add codegen request/result model types for FOL generation, schema generation, and asset preprocessing.
7. `[complete]` Add generated-output dependency collection and lookup helpers.
8. `[complete]` Extend the draft build API with write-file, copy-file, system-tool, and codegen helpers.
9. `[complete]` Extend the build evaluator operation model to replay generated-file and tool/codegen actions.
10. `[complete]` Re-export the draft generated-file and codegen surface from `fol-package`.

## Round 10 Slice Tracker

This round focuses on Phase 10 native artifacts and C ABI work in `fol-package`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the tenth implementation round around native-artifact and C-ABI modeling.
2. `[complete]` Introduce a dedicated `build_native` module with native-artifact shell types.
3. `[complete]` Add first-class native artifact kinds for headers, objects, static libraries, and shared libraries.
4. `[complete]` Add include-path and library-path model types for native artifact search semantics.
5. `[complete]` Add native link mode and link input records for backend-produced artifacts.
6. `[complete]` Add cross-platform native naming/path convention helpers for headers and libraries.
7. `[complete]` Add compatibility projection helpers from parsed placeholder native artifacts into the new native-artifact model.
8. `[complete]` Extend artifact definitions to retain structured native artifact attachments instead of plain strings.
9. `[complete]` Extend prepared-package modeling to retain optional native artifact surfaces alongside compatibility native placeholders.
10. `[complete]` Re-export the draft native-artifact surface from `fol-package`.

## Round 11 Slice Tracker

This round focuses on Phase 11 CLI-migration groundwork in `fol-frontend`.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the eleventh implementation round around CLI migration and build-step routing.
2. `[complete]` Introduce a dedicated frontend build-route module with workflow-mode shell types.
3. `[complete]` Add default build-step mapping for `build` / `run` / `test` / `check` commands.
4. `[complete]` Add explicit `--step` CLI arguments for workspace code commands.
5. `[complete]` Thread selected build-step overrides through frontend configuration.
6. `[complete]` Add workspace build-file route planning that classifies compatibility, hybrid, and modern members.
7. `[complete]` Add compatibility-step execution helpers that map named steps onto existing frontend workspace commands.
8. `[complete]` Route workspace code commands through the new build-route planner and compatibility executor.
9. `[complete]` Preserve stable compatibility command summaries and artifact reporting through routed step execution.
10. `[complete]` Reject modern-entry workspace commands with explicit unsupported-build-entry diagnostics until graph execution lands.

## Round 12 Slice Tracker

This round focuses on Phase 12 docs, scaffolding, editor surfaces, and sample projects.
Each slice must land green with `make build` and `make test` before commit.

1. `[complete]` Lock the twelfth implementation round around docs, scaffolding, editor build-file affordances, and examples.
2. `[complete]` Update frontend package scaffolding so generated `build.fol` files explain the current root/export path and future build entrypoint direction.
3. `[complete]` Add scaffold regression tests that lock the new generated `build.fol` template for bin and lib packages.
4. `[complete]` Rewrite `README.md` build/package descriptions around `build.fol` as the documented entry file and current routed workflow status.
5. `[complete]` Rewrite the `book` package/import/build-root sections so they teach the current `build.fol` model clearly.
6. `[complete]` Update `book` tooling/editor sections so build-file workflows and LSP/editor entrypoints are documented together.
7. `[complete]` Add frontend editor-command regression coverage for `build.fol` parse/highlight/symbol extraction.
8. `[complete]` Add LSP regression coverage for `build.fol` symbol extraction and completion affordances.
9. `[complete]` Add a checked-in `examples/` tree with simple exe, static-lib, shared-lib, generated-file, and dependency-workspace sample projects.
10. `[complete]` Add tests that validate the checked-in examples are discoverable and their formal package `build.fol` files parse cleanly.

## Roadmap Status

The roadmap work in this file is complete as infrastructure and documentation
work, but the product target is only partially complete.

What is true at the current head:

1. `build.fol` is the documented package entry file.
2. `fol-package` has first-class build-graph, option, artifact, step,
   dependency, codegen, and native modeling layers.
3. frontend workspace `build` / `run` / `test` / `check` commands route through
   an explicit build-planning layer.
4. compatibility-only `build.fol` packages still work through that routed path.
5. modern and hybrid build entries are still blocked with explicit unsupported
   diagnostics until graph execution is wired through end-to-end.

## Original Success Criteria

The original product target for this plan was:

1. `build.fol` is the actual build entrypoint for FOL projects.
2. `fol code ...` runs through a real build graph.
3. the graph can express executables, libraries, tests, run/install steps,
   options, and dependency-provided build surfaces.
4. current `pkg` dependency/export semantics are preserved through migration.
5. the result feels comparable to Zig’s build system in power, while still
   reading like FOL.

## Remaining Product Polish

These are the remaining product-level gaps between the completed roadmap work
and the original end-state promise:

1. Make graph-backed modern `build.fol` execution the live path for
   `fol code build/run/test/check`.
2. Replace the current compatibility-only workspace executor with real graph
   step execution and artifact production.
3. Promote checked-in examples from compatibility roots to true modern
   graph-authored builds once that path is live.
4. Tighten user-facing CLI summaries and diagnostics so modern build execution
   feels first-class instead of experimental.
5. Rescan `README`, `book`, and `PROGRESS.md` once modern execution lands so
   the repo status files describe the product without roadmap caveats.
