# Zig Gap Round 2 Plan

This plan defines the next build-system round after the completed:

- `.build().meta(...)`
- `.build().add_dep(...)`
- `.build().graph()`
- dependency handles
- unified generated-output handles
- explicit dependency arg forwarding
- install-prefix projection

The goal of this round is to close the next set of gaps that still separate
FOL build from the high-value parts of Zig's build system.

This plan is based on:

- an actual repo scan of the current build surface and tests
- the current FOL build docs under `book/src/055_build`
- the official Zig build-system docs and release-note material on dependency
  access, build options, install prefix separation, and lazy path usage

Grounding inside this repo:

- build API: `lang/execution/fol-build/src/api/build_api.rs`
- build API types: `lang/execution/fol-build/src/api/types.rs`
- dependency surface model: `lang/execution/fol-build/src/dependency.rs`
- semantic registry: `lang/execution/fol-build/src/semantic.rs`
- graph execution: `lang/execution/fol-build/src/executor/graph_methods.rs`
- handle execution: `lang/execution/fol-build/src/executor/handle_methods.rs`
- build evaluation from source: `lang/execution/fol-build/src/eval/source.rs`
- build plan replay: `lang/execution/fol-build/src/eval/plan.rs`
- step/cache/report model: `lang/execution/fol-build/src/step.rs`
- frontend build routing: `lang/tooling/fol-frontend/src/build_route/mod.rs`
- package preparation: `lang/compiler/fol-package/src/session/mod.rs`
- current examples and integration tests:
  - `examples/build_*`
  - `test/integration_tests/integration_editor_and_build.rs`

## Primary Targets

1. make dependency exposure explicit instead of only auto-projected
2. finish the path-handle model so source files and directories are first-class
3. expose public dependency evaluation modes (`eager`, `lazy`, `on-demand`)
4. improve step/help ergonomics with real step descriptions and better default-step UX
5. add a first serious system-integration surface
6. harden docs/examples/editor/tests around the final public build shape

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

### 2. Dependency handles should expose declared exports, not only implicit projections

Today dependency handles query a deterministic projected surface:

- modules
- artifacts
- steps
- generated outputs
- source roots

That is useful, but still too implicit.

The next public model should let the dependency package declare what it exposes
for build consumption.

The chosen direction for this plan:

- keep deterministic fallback projection while building the new model
- but move the public examples/docs/tests toward explicit export declarations
- end state should prefer explicit export names over accidental projection

### 3. Path handles should cover both generated and source-backed paths

Today `OutputHandle` unifies:

- `write_file`
- `copy_file`
- `run.capture_stdout`
- dependency generated outputs

But plain source paths and directories are still mostly strings.

The next model should introduce one higher-level path handle family that can
represent:

- source file under package root
- source directory under package root
- generated file
- copied file
- captured stdout
- dependency generated output
- dependency exported source file/dir, if exposed

This is the FOL equivalent of the capability Zig gets from `LazyPath`,
without copying the name blindly.

### 4. Dependency evaluation mode should become a real public config field

The repo already has:

- `eager`
- `lazy`
- `on-demand`

internally in `DependencyBuildEvaluationMode`, but the public `.build().add_dep`
surface does not yet expose them cleanly.

This round makes those public.

### 5. Step descriptions are worth adding now

Zig's `zig build --help` is useful because steps are not just names; they also
have descriptions and user-facing meaning.

FOL should add:

- optional step description
- clearer default-step reporting
- better CLI/help surfacing

### 6. System integration should start narrow and typed

Do not jump directly to a giant native-toolchain DSL.

Start with typed, explicit public surfaces for:

- system commands/tools
- environment values
- optionally system libraries / pkg-config style requests

If a system-library surface is added, it must be explicit and typed, not a
catch-all stringly escape hatch.

### 7. No legacy shims

If a new export/path/step/help/system surface replaces an older ad hoc shape:

- remove old docs
- remove obsolete helper wording
- do not keep dual public guidance

## Epoch 1: Freeze The New Public Direction

### Slice 1 (complete)

- audit current docs/examples/tests for the next gaps:
  - dependency exports are still described as only projected
  - source paths are still string-based
  - dependency evaluation modes are still mostly internal
  - steps do not have descriptions/help-grade output
  - system integration is still underpowered
- no behavior change

### Slice 2 (complete)

- add a short build-architecture note in the book describing the next public
  layers:
  - `.build().add_dep(... mode = ...)`
  - explicit dependency exports
  - path handles
  - step descriptions
  - system integration handles
- keep this repo-specific, not a generic Zig essay

### Slice 3 (complete)

- record explicit non-goals for this round:
  - no source-level build manifest file
  - no public `Graph` type name
  - no compatibility API for replaced string-only helpers
  - no fake parallel execution claims

## Epoch 2: Add Explicit Dependency Export Declarations

### Slice 4 (complete)

- inventory how dependency surfaces are projected today from prepared packages
- document exactly which pieces are implicit today:
  - source roots
  - modules
  - artifacts
  - steps
  - generated outputs

### Slice 5 (complete)

- design one explicit export declaration surface in `build.fol`
- recommended direction:
  - export methods on `build` or `graph`, not package metadata
  - examples:
    - `build.export_module({ name = "core", module = lib_mod })`
    - `build.export_artifact({ name = "corelib", artifact = lib })`
    - `build.export_step({ name = "check", step = check })`
    - `build.export_output({ name = "bindings", output = generated })`
- keep names concise and symmetric with the current build API

### Slice 6 (complete)

- add semantic signatures for explicit dependency export methods
- return values should remain chainable or `non` as appropriate
- make receiver placement coherent with the existing `.build()` layering

### Slice 7 (complete)

- add runtime representation for explicitly exported dependency surfaces
- keep export names separate from projected/internal names

### Slice 8 (complete)

- wire export evaluation through the build executor
- exporting a handle should record stable surface metadata

### Slice 9 (complete)

- package preparation should persist explicit exported surfaces on formal packages
- keep deterministic behavior when no explicit export exists yet

### Slice 10 (complete)

- dependency handles should prefer explicit exports when present
- fall back to current projection only where required by this transition round

### Slice 11 (complete)

- add precise diagnostics for:
  - duplicate export names
  - export kind/handle mismatch
  - querying names that are not exported

### Slice 12 (complete)

- add build-eval tests for explicit export declarations and consumption

### Slice 13 (complete)

- add integration tests showing one package exporting:
  - module
  - artifact
  - step
  - generated output
  and another package consuming those through dependency handles

## Epoch 3: Complete The Path Handle Model

### Slice 14 (complete)

- audit current path-like values:
  - `graph.path_from_root(...)`
  - strings passed to `copy_file`
  - strings passed to `install_file`
  - strings passed to `install_dir`
  - generated output handles
  - dependency generated outputs

### Slice 15 (complete)

- define a new public path-handle family above current `OutputHandle`
- recommended family members:
  - source file
  - source dir
  - generated output
  - dependency generated output

### Slice 16 (complete)

- add API/types for source file and source dir handles in
  `lang/execution/fol-build/src/api/types.rs`

### Slice 17 (complete)

- make `graph.path_from_root(...)` return a source-file handle instead of a raw string
- if necessary, split into:
  - `graph.file_from_root(...)`
  - `graph.dir_from_root(...)`
  and delete the ambiguous old path form

### Slice 18 (complete)

- update `graph.copy_file(...)` so `source` can accept a source-file handle
  instead of only a raw string

### Slice 19 (complete)

- update `graph.install_file(...)` so it accepts:
  - source-file handle
  - generated-output handle
  and rejects unrelated values exactly

### Slice 20 (complete)

- update `graph.install_dir(...)` so it accepts a source-dir handle instead of
  only a raw string path

### Slice 21 (complete)

- update `run.add_file_arg(...)` to accept source-file handles where sensible

### Slice 22 (complete)

- keep generated-output composition stable after the path-handle expansion

### Slice 23 (complete)

- add exact diagnostics for:
  - file/dir kind mismatch
  - passing dir handle where file handle is required
  - passing arbitrary scalar where a path handle is required

### Slice 24 (complete)

- add build-eval and integration tests for:
  - source file handles
  - source dir handles
  - mixed source/generated composition

## Epoch 4: Make Dependency Evaluation Mode Public

### Slice 25 (complete)

- extend `.build().add_dep({...})` to accept `mode`
- public accepted values:
  - `eager`
  - `lazy`
  - `on-demand`

### Slice 26 (complete)

- add semantic typing and exact config diagnostics for dependency mode

### Slice 27 (complete)

- thread public mode into dependency request/runtime/preparation structures

### Slice 28 (complete)

- define frontend/package semantics for each mode:
  - when it is fetched
  - when it is prepared
  - when it is fully evaluated
- document this honestly if the runtime still behaves partly eagerly underneath

### Slice 29 (complete)

- add tests for public dependency mode acceptance and exact diagnostics

### Slice 30 (complete)

- add at least one example package that uses mixed dependency modes

## Epoch 5: Improve Step Help And User-Facing Step Ergonomics

### Slice 31 (complete)

- extend `graph.step(...)` to accept an optional description
- keep old no-description shape only if it is the same canonical call form;
  otherwise replace it directly

### Slice 32 (complete)

- persist step descriptions in graph/runtime/step-report data

### Slice 33 (complete)

- teach frontend summaries and `build --help`-style output to show:
  - step name
  - default-step kind if any
  - description if present

### Slice 34 (complete)

- improve diagnostics when the user asks for an unknown named step:
  - show known steps
  - prefer descriptions where available

### Slice 35 (complete)

- add tests for described steps in:
  - build eval
  - build route planning
  - CLI output/help reporting

## Epoch 6: Add A Narrow System Integration Surface

### Slice 36 (complete)

- audit current system tool / codegen support and define the smallest good
  public expansion

### Slice 37 (complete)

- choose one or both of these initial public surfaces:
  - typed system-command environment/file arguments
  - typed system-library request surface

### Slice 38 (complete)

- if system-library support is added, make it explicit and typed:
  - library name
  - mode/static-shared preference if any
  - provider strategy if any
- do not add a vague stringly escape hatch

This round chose the narrower typed system-command surface only, so no
system-library API was added here.

### Slice 39 (complete)

- wire the chosen system integration requests into graph/runtime/step reporting

### Slice 40 (complete)

- add tests and one standalone example for the new system integration surface

## Epoch 7: Tighten Dependency Import/Build Surface Separation

### Slice 41 (complete)

- now that explicit exports exist, re-audit the boundary between:
  - source imports (`use alias: pkg = {...}`)
  - build-surface dependency handle queries
- document the exact separation again

### Slice 42 (complete)

- remove obsolete dependency projection code or assumptions that are superseded
  by explicit exports

### Slice 43 (complete)

- add negative tests proving:
  - exported build surfaces do not silently change source import rules
  - source imports do not silently expose non-exported build surfaces

## Epoch 8: Examples And Docs

### Slice 44 (complete)

- add one standalone example focused on explicit dependency exports

### Slice 45 (complete)

- add one standalone example focused on source-path handles

### Slice 46 (complete)

- add one standalone example focused on dependency mode selection

### Slice 47 (complete)

- add one standalone example focused on described custom steps

### Slice 48 (complete)

- update the build book sections:
  - `100_build_file.md`
  - `200_graph_api.md`
  - `300_handle_api.md`
  - `400_options.md`
  - `900_direction.md`
- remove wording that still describes these as only future direction if they
  land in this round

## Epoch 9: Editor And Tooling Audit

### Slice 49 (complete)

- audit editor/LSP completion and tree-sitter sync for:
  - new build export methods
  - new path-handle methods
  - dependency mode field names
  - step description config fields
  - system integration surface names
- add regression tests for the new build-only completion items

## Epoch 10: Final Cleanup

### Slice 50 (complete)

- final repo-wide scan for:
  - stale build docs
  - dead helpers from the old projection-only model
  - duplicate path-handle representations
  - stale example text that still teaches weaker surfaces

## Suggested Execution Order

Recommended order:

1. Epoch 2
2. Epoch 3
3. Epoch 4
4. Epoch 5
5. Epoch 6
6. Epoch 7
7. Epoch 8
8. Epoch 9
9. Epoch 10

Reason:

- explicit dependency exports and full path handles are the biggest remaining
  structural gaps
- public dependency mode is already half-present internally
- step/help improvements depend on the stabilized public graph surface
- system integration should come after the handle model is clearer
- docs/examples/editor work should lock the final surface after behavior lands

## Exit Criteria

This round is complete when:

- dependency-facing build surfaces can be exported explicitly by a dependency
  package and queried predictably by consumers
- source files and directories are first-class path handles instead of mostly
  raw strings
- dependency modes are public on `.build().add_dep({...})`
- step descriptions show up in frontend/help/reporting
- one narrow but real system integration surface exists and is tested
- the book/examples/editor/tests all reflect the final public build API
