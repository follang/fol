# Zig Gap Round 3 Plan

This plan defines the next build-system round after the completed:

- `.build().meta(...)`
- `.build().add_dep(...)`
- `.build().graph()`
- dependency handles
- explicit dependency exports
- source file and source dir handles
- unified output handles
- dependency arg forwarding
- dependency evaluation modes
- install-prefix projection
- step descriptions
- typed system tools

The goal of this round is to close the next practical gaps that still separate
FOL build from the most useful parts of Zig's build system, without copying Zig
blindly and without introducing public legacy shims.

This plan is based on:

- a fresh repo scan of the current build API, build evaluator, frontend, editor,
  and example coverage
- the current FOL build book under `book/src/055_build`
- the Zig build-system guide and the 0.11, 0.12, and 0.14 release-note material
  around dependency access, lazy path usage, lazy dependencies, step summaries,
  and system integration

Grounding inside this repo:

- build API: `lang/execution/fol-build/src/api/build_api.rs`
- build API types: `lang/execution/fol-build/src/api/types.rs`
- dependency surface model: `lang/execution/fol-build/src/dependency.rs`
- runtime handle model: `lang/execution/fol-build/src/runtime.rs`
- semantic registry: `lang/execution/fol-build/src/semantic.rs`
- graph execution: `lang/execution/fol-build/src/executor/graph_methods.rs`
- handle execution: `lang/execution/fol-build/src/executor/handle_methods.rs`
- build evaluation from source: `lang/execution/fol-build/src/eval/source.rs`
- build plan replay: `lang/execution/fol-build/src/eval/plan.rs`
- step/cache/report model: `lang/execution/fol-build/src/step.rs`
- dependency projection: `lang/compiler/fol-package/src/build_dependency.rs`
- package prep/loading: `lang/compiler/fol-package/src/session/mod.rs`
- frontend routed build planning: `lang/tooling/fol-frontend/src/build_route/mod.rs`
- frontend direct/build execution: `lang/tooling/fol-frontend/src/compile/mod.rs`
- frontend direct compilation path: `lang/tooling/fol-frontend/src/direct.rs`
- editor/LSP build tests: `lang/tooling/fol-editor/src/lsp/tests/*`
- current standalone examples:
  - `examples/build_dep_*`
  - `examples/build_output_handles`
  - `examples/build_install_prefix`
  - `examples/build_source_paths`
  - `examples/build_system_tool`

## Primary Targets

1. add named dependency path exports and path queries
2. move toward one broader path-handle capability instead of split path families
3. make dependency evaluation modes behave more concretely
4. improve build help/reporting/install visibility at the CLI level
5. add a typed public system-library surface
6. support generated-directory style workflows cleanly
7. strengthen option forwarding and dependency configuration ergonomics

## Design Decisions

These decisions are part of the plan unless replaced deliberately.

### 1. Keep `.build()` as the only top-level build entry

Do not reintroduce public `Graph`, `Build`, or manifest files.

Public shape stays:

```fol
pro[] build(): non = {
    var build = .build();
    build.meta({ name = "app", version = "0.1.0" });
    var graph = build.graph();
}
```

### 2. Do not replace explicit dependency exports

The previous round made explicit exports the real build-facing contract.

This round extends that model instead of replacing it:

- keep `export_module`
- keep `export_artifact`
- keep `export_step`
- keep `export_output`
- add path-oriented exports beside them

### 3. Path capability should converge, not multiply

Today the public build surface has multiple path-like families:

- source file handles
- source dir handles
- output handles
- dependency generated-output handles

The next step should unify consumers and metadata around a more general path
family, even if the internal representation still has specialized variants.

Publicly, the direction should feel like:

- one broader path capability
- strongly typed producers
- consumers accept the right path classes without string escape hatches

### 4. Dependency modes should become semantically meaningful

Public dependency modes already exist:

- `eager`
- `lazy`
- `on-demand`

This round should give them clearer semantics in:

- metadata extraction
- package preparation
- fetch behavior
- build evaluation diagnostics
- examples and docs

### 5. System integration must stay typed and narrow

Do not add a giant “shell escape” build language.

This round may add:

- system library requests
- search paths / framework / pkg-config-like typed requests
- generated directory outputs

It must not add:

- arbitrary stringly “do anything” linker DSLs
- compatibility wrappers around old ad hoc helpers

### 6. CLI/help/reporting should describe the real build graph

The build graph now carries more structure than the CLI exposes.

This round should improve:

- step help
- install prefix surfacing
- exported dependency surfaces
- selected dependency modes
- produced outputs and destinations

without inventing fake parallelism claims.

### 7. No legacy shims

If a broader path model or system-library surface replaces an older ad hoc
shape:

- remove stale docs
- remove stale examples
- remove stale tests
- do not keep dual public guidance

## Epoch 1: Freeze The Round 3 Direction

### Slice 1 (complete)

- audit current docs/examples/tests for the remaining gaps:
  - no path-oriented dependency exports
  - no general dependency path queries
  - dependency modes are still weak semantically
  - no public system-library surface
  - CLI help/reporting still under-exposes step/output/install detail
  - generated-directory workflows are still thin
- no behavior change

### Slice 2 (complete)

- add a short book architecture note describing this round:
  - path exports
  - broader path handles
  - stronger dependency modes
  - system-library surface
  - generated directories
  - improved build help/reporting
- keep it repo-specific, not a generic Zig essay

### Slice 3 (complete)

- record explicit non-goals for this round:
  - no new control file
  - no public `Graph`
  - no compatibility string-path fallback API
  - no fake build parallelism claims
  - no broad shell-script DSL

## Epoch 2: Add Named Dependency Path Exports

### Slice 4 (complete)

- inventory current explicit dependency export model:
  - module
  - artifact
  - step
  - generated output
- document what is missing:
  - source file exports
  - source dir exports
  - generated dir exports
  - general path exports

### Slice 5 (complete)

- design a path-oriented explicit export surface
- recommended direction:
  - `build.export_file({ name = "config", file = source_file })`
  - `build.export_dir({ name = "assets", dir = source_dir })`
  - `build.export_path({ name = "schema", path = output })`
- keep it symmetric with the current explicit-export model

### Slice 6 (complete)

- add semantic signatures and runtime representation for the new path exports
- keep export names separate from internal path identity

### Slice 7 (complete)

- build executor should record explicit path exports in stable dependency-surface
  metadata

### Slice 8 (complete)

- package dependency-surface projection should persist explicit path exports on
  prepared packages

### Slice 9 (complete)

- add build-eval tests for:
  - exporting source file handles
  - exporting source dir handles
  - exporting output handles through the path export surface

### Slice 10 (complete)

- add diagnostics for:
  - duplicate path export names
  - wrong handle kind passed to file/dir/path exports
  - unresolved exported path targets

### Slice 11 (complete)

- add one integration example package exporting:
  - a module
  - an artifact
  - a generated output
  - a source file
  - a source dir

## Epoch 3: Add Dependency Path Queries

### Slice 12 (complete)

- design public dependency-handle queries for the new path exports
- recommended direction:
  - `dep.file("config")`
  - `dep.dir("assets")`
  - `dep.path("schema")`
- keep them coherent with:
  - `dep.module(...)`
  - `dep.artifact(...)`
  - `dep.step(...)`
  - `dep.generated(...)`

### Slice 13 (complete)

- add semantic signatures and typed handle results for dependency file/dir/path
  queries

### Slice 14 (complete)

- dependency handles should resolve explicitly exported path names first
- querying a missing export should produce exact diagnostics

### Slice 15 (complete)

- update path consumers to accept dependency-exported file/dir/path handles where
  appropriate
- expected consumers:
  - install file/dir
  - run file args
  - artifact generated/path attachment where valid

### Slice 16 (complete)

- add evaluator tests for cross-package path consumption through dependency
  handles

### Slice 17 (complete)

- add integration coverage for one package exporting named paths and another
  package installing or passing them to a tool step

## Epoch 4: Broaden The Path Handle Model

### Slice 18 (complete)

- audit current path-like handle families and consumers
- document the current split:
  - `SourceFileHandle`
  - `SourceDirHandle`
  - generated/output handles
  - dependency generated-output handles

### Slice 19 (complete)

- design one broader public path-handle family
- internal variants may remain specialized, but the public consumption model
  should converge

### Slice 20 (complete)

- add a canonical runtime/type representation for generalized path handles

### Slice 21 (complete)

- update consumers so they validate path-handle capabilities by kind rather than
  by unrelated ad hoc branches

### Slice 22 (complete)

- add exact diagnostics for bad path-handle use:
  - file where dir is required
  - dir where file is required
  - path handle of the wrong provenance

### Slice 23 (complete)

- update build docs and examples so path composition uses the broader path model
  consistently

## Epoch 5: Make Dependency Modes Behave More Concretely

### Slice 24 (complete)

- audit current use of dependency modes in:
  - build evaluator
  - metadata extraction
  - package session
  - fetch flows
  - docs/examples

### Slice 25 (complete)

- define the concrete intended behavior:
  - `eager`: preload and validate dependency surface immediately
  - `lazy`: delay expensive preparation until dependency handle/build import use
  - `on-demand`: only prepare when the graph or frontend path truly requires it

### Slice 26 (complete)

- preserve dependency mode through all current metadata/execution layers
- no silent dropping during prepare/fetch/build planning

### Slice 27 (complete)

- fetch/package logic should surface the chosen dependency modes clearly in tests
  and summaries where relevant

### Slice 28 (complete)

- add diagnostics for contradictory or unsupported mode usage if any current
  path cannot honor the requested mode yet

### Slice 29 (complete)

- add integration coverage with mixed dependency modes across:
  - `loc`
  - `pkg`
  - `git`

## Epoch 6: Improve Step Help And Build Reporting

### Slice 30 (complete)

- audit current frontend build help/summary/reporting surfaces
- list where step descriptions and output details are currently lost

### Slice 31 (complete)

- improve unknown-step diagnostics further so they list:
  - step name
  - default kind
  - description
  - maybe selected artifact label when relevant

### Slice 32 (complete)

- improve build summaries so they surface:
  - install prefix
  - selected fol models
  - dependency mode summaries where relevant
  - produced output counts

### Slice 33 (complete)

- improve routed build planning summaries so step/output/install context is easier
  to read in tests and user-facing output

### Slice 34 (complete)

- add integration tests that lock the improved help/reporting output

## Epoch 7: Add A Typed System-Library Surface

### Slice 35 (complete)

- audit current system-tool support and backend/native gaps
- define the narrow first public system-library surface

### Slice 36 (complete)

- design typed API methods, likely on `graph`
- recommended direction:
  - `graph.add_system_lib({ name = "ssl" })`
  - optional typed fields for:
    - kind
    - search path handle or path string if necessary
    - framework flag on relevant targets
    - pkg-config style probe mode if added

### Slice 37 (complete)

- add semantic signatures and runtime representation for the system-library
  requests

### Slice 38 (complete)

- backend/build planning should preserve these requests through emitted build
  configuration even if support is initially narrow

### Slice 39 (complete)

- add diagnostics for:
  - invalid system-library config shapes
  - unsupported target/library combinations where known

### Slice 40 (complete)

- add one standalone example package using the typed system-library surface

## Epoch 8: Support Generated Directories

### Slice 41 (complete)

- audit current generated-file flows and identify where generated directories are
  missing

### Slice 42 (complete)

- design one public generated-dir/output-dir surface
- recommended direction:
  - system tool or codegen requests may produce a named directory handle
  - install/run consumers can accept it where valid

### Slice 43 (complete)

- add runtime/type support for generated directory outputs

### Slice 44 (complete)

- extend install and path-consumer logic to accept generated directory handles

### Slice 45 (complete)

- add evaluator/integration tests for:
  - generated dir production
  - generated dir installation
  - generated dir export/query through dependencies if the model supports it

## Epoch 9: Improve Dependency Config Ergonomics

### Slice 46 (complete)

- audit current dependency arg forwarding and compare it to the real use cases in
  examples/tests

### Slice 47

- add stronger support for common forwarded build config values where missing:
  - target
  - optimize
  - user options
  - maybe environment selection if explicitly chosen

### Slice 48

- tighten diagnostics for missing/invalid forwarded dependency config values so
  they fail early and clearly

## Epoch 10: Final Hardening

### Slice 49 (complete)

- add standalone examples for:
  - explicit path exports
  - dependency path queries
  - mixed dependency modes
  - system library use
  - generated directories

### Slice 50 (complete)

- harden editor/LSP/build integration coverage so new build members and handles
  appear in:
  - completion
  - hover where applicable
  - build-fixture integration coverage

### Slice 51 (complete)

- audit and update the build book:
  - remove stale wording about projection-only dependency surfaces
  - describe path exports and dependency path queries
  - describe dependency modes honestly
  - document system-library scope honestly

### Slice 52 (complete)

- final cleanup:
  - remove stale helper wording in tests/docs/examples
  - keep only the chosen public build story
  - ensure all new examples are referenced by docs and tested

## Completion Criteria

This round is complete when all of the following are true:

- dependency packages can explicitly export named file/dir/path surfaces
- dependent packages can query and consume those path exports through dependency
  handles
- the public path model feels broader and less fragmented
- dependency modes are preserved and exercised meaningfully
- CLI/build help and summaries expose step/output/install information better
- a typed public system-library surface exists
- generated-directory workflows are covered
- docs, examples, editor coverage, and integration tests all match the final
  chosen build story
