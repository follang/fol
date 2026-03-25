# Build-System Gap Plan

This plan covers the next build-system round after the current one-file
`build.fol` model.

Goal:

- keep the current FOL-first build surface
- do not copy Zig mechanically
- add the highest-value capabilities that Zig already proves out
- avoid parallel legacy paths

Primary targets for this round:

1. public dependency handles from `.build().add_dep(...)`
2. first-class generated output / lazy-path style handles
3. per-dependency option forwarding
4. stronger step execution and cache semantics
5. cleaner install/output-prefix model

This plan is grounded in the current implementation seams:

- build API: `lang/execution/fol-build/src/api/build_api.rs`
- build handle types: `lang/execution/fol-build/src/api/types.rs`
- semantic surface: `lang/execution/fol-build/src/semantic.rs`
- graph execution: `lang/execution/fol-build/src/executor/graph_methods.rs`
- handle execution: `lang/execution/fol-build/src/executor/handle_methods.rs`
- build source evaluation: `lang/execution/fol-build/src/eval/source.rs`
- step planning/cache model: `lang/execution/fol-build/src/step.rs`
- dependency surface model: `lang/execution/fol-build/src/dependency.rs`
- frontend fetch/build routing: `lang/tooling/fol-frontend/src/fetch.rs`
- frontend compile resolver config: `lang/tooling/fol-frontend/src/compile/mod.rs`
- current build docs: `book/src/055_build/*.md`

## Design Decisions

These should be treated as fixed unless a better design is chosen explicitly.

### 1. Keep `.build()` as the top-level public entry

Do not reintroduce public `Graph` or `Build` type names.

Public shape stays:

```fol
pro[] build(): non = {
    var build = .build();
    build.meta({ name = "app", version = "0.1.0" });
    build.add_dep({ alias = "logtiny", source = "git", target = "git+https://..." });
    var graph = build.graph();
}
```

### 2. `.build().add_dep(...)` should return a real dependency handle

Current internal machinery already supports dependency handles and queries.
The public contract should become:

```fol
var logtiny = build.add_dep({
    alias = "logtiny",
    source = "git",
    target = "git+https://github.com/bresilla/logtiny.git",
});
```

and later:

```fol
var logtiny_mod = logtiny.module("logtiny");
var logtiny_lib = logtiny.artifact("logtiny");
var logtiny_gen = logtiny.generated("bindings");
```

### 3. Add a unified path/output handle family

Zig uses `LazyPath`. FOL should not copy the name automatically, but it needs
the same capability class:

- source path from package root
- generated file from `write_file`
- copied file from `copy_file`
- captured stdout from run/system-tool/codegen
- dependency generated output

The handle should compose across graph methods and handle methods.

### 4. Dependency forwarding must stay explicit

Do not auto-forward target/optimize/user options to dependencies.

Public style should be explicit, for example:

```fol
var target = graph.standard_target();
var optimize = graph.standard_optimize();

var dep = build.add_dep({
    alias = "json",
    source = "pkg",
    target = "json",
    args = {
        target = target,
        optimize = optimize,
        use_fast_parser = true,
    },
});
```

### 5. Installation/output must remain user-directed, not hardcoded by package

FOL should move closer to Zig’s separation of cache vs install prefix.

The package author should declare what gets installed.
The user/tool should choose where it lands.

### 6. No compatibility layer for old surface

If a new dependency/output/install shape is chosen:

- remove old docs
- remove fallback parsing
- remove dual surface

## Epoch 1: Freeze The Public Direction

### Slice 1 [complete]

- audit current build docs and tests for stale `.graph()`-first examples
- record the intended public dependency/output story in docs comments
- no behavior change

### Slice 2 [complete]

- add a book note comparing current FOL build goals against:
  - dependency handles
  - generated outputs
  - explicit dependency args
  - install prefixes
- keep this repo-facing, not a generic Zig essay

### Slice 3 [complete]

- add a top-level architecture note for build surface layering:
  - `.build()`
  - `build.meta`
  - `build.add_dep`
  - `build.graph`
  - dependency handles
  - output handles

## Epoch 2: Promote Dependency Handles To The Public Surface

### Slice 4 [complete]

- extend `build.add_dep({...})` executor path so it returns a structured
  dependency handle intentionally, not just metadata side-effects
- tighten tests around returned dependency values

### Slice 5 [complete]

- define public semantic signatures for dependency handle methods:
  - `module(name)`
  - `artifact(name)`
  - `step(name)`
  - `generated(name)`
- wire them into semantic registries

### Slice 6 [complete]

- expose dependency handles in build docs with one simple package example
- document that these reflect exposed surfaces from the dependency package

### Slice 7 [complete]

- add resolver/typecheck coverage for dependency-handle method calls inside
  `build.fol`
- make failures precise for unknown module/artifact/step/generated names

### Slice 8 [complete]

- harden executor and runtime query recording for dependency-handle lookups
- ensure all dependency queries preserve alias + requested name + kind

### Slice 9 [complete]

- add an end-to-end fixture where a dependency is:
  - declared via `build.add_dep`
  - queried via `dep.module(...)` or `dep.generated(...)`
  - fed back into graph operations

## Epoch 3: Build A Unified Output Handle Model

### Slice 10 [complete]

- inventory current output-like values:
  - `write_file`
  - `copy_file`
  - `run.capture_stdout`
  - codegen outputs
  - dependency generated outputs
  - source-root-relative file references
- define one canonical internal handle family

### Slice 11 [complete]

- add explicit API/types for output handles in
  `lang/execution/fol-build/src/api/types.rs`
- keep names repo-appropriate; avoid blindly copying `LazyPath`

### Slice 12

- make `graph.write_file(...)` and `graph.copy_file(...)` return the unified
  output handle instead of ad hoc generated-file values

### Slice 13

- make `run.capture_stdout()` return the same unified output handle
- ensure stdout capture is compatible with downstream graph consumers

### Slice 14

- make dependency `generated(...)` return the same unified output handle class

### Slice 15

- add graph methods that accept output handles where path-like inputs make
  sense today:
  - `install_file`
  - run file args
  - artifact generated attachments

### Slice 16

- add source-eval tests proving output handles can flow through local helpers
  without degrading to strings

### Slice 17

- add book chapter section for output-handle composition with examples

## Epoch 4: Explicit Dependency Argument Forwarding

### Slice 18

- extend dependency request schema to accept `args = { ... }`
- support scalar values first:
  - `bool`
  - `int`
  - `str`
  - option handles for target/optimize/path-like values

### Slice 19

- add semantic typing for dependency arg records
- reject unknown field/value shapes cleanly

### Slice 20

- thread dependency args through:
  - build evaluation operations
  - dependency runtime representation
  - fetch/preparation structures if needed

### Slice 21

- define how forwarded args influence dependency build evaluation:
  - target
  - optimize
  - dependency-specific user options

### Slice 22

- add package-loading tests for:
  - forwarded target/optimize
  - forwarded user bool/string/path/int options
  - missing required dependency options

### Slice 23

- add book examples showing explicit forwarding
- state clearly that nothing is implicitly inherited

## Epoch 5: Tighten Dependency Surface Exposure

### Slice 24

- define what a dependency exposes by default:
  - package source roots
  - generated outputs
  - installed artifacts
  - named graph modules
  - named steps

### Slice 25

- make dependency surface projection deterministic and testable
- remove ad hoc alias assumptions

### Slice 26

- decide whether imports should continue to resolve by alias projection under
  `.fol/pkg/<alias>` or instead through a richer exposed-surface registry
- document the chosen model and remove the unused one

### Slice 27

- add regression tests for mixed local/pkg/git dependency exposure
- include at least one imported generated-output case

## Epoch 6: Strengthen Step Execution And Cache Semantics

### Slice 28

- audit what step cache keys currently model vs what frontend/backend actually
  reuse
- document the gap

### Slice 29

- make produced outputs participate in step cache-key semantics where relevant
- generated file changes should invalidate dependent steps predictably

### Slice 30

- add explicit step execution reporting that distinguishes:
  - requested
  - executed
  - skipped-from-cache
  - skipped-by-foreign-run-policy

### Slice 31

- make run/system-tool/codegen step summaries expose enough detail for frontend
  reporting and tests

### Slice 32

- add tests covering deterministic step ordering and cache invalidation
  boundaries for:
  - options
  - artifact roots
  - generated outputs
  - dependency args

### Slice 33

- if execution is still fully serial, document that honestly and add a follow-up
  note for future parallel execution
- do not fake parallelism in docs

## Epoch 7: Install Prefix And Output Layout

### Slice 34

- audit current build root, cache root, git cache root, and install semantics
- define a clean public model:
  - cache/build internals
  - user-visible install prefix

### Slice 35

- add frontend config for install prefix selection, separate from build/cache
- make default behavior deterministic for local development

### Slice 36

- teach `graph.install`, `graph.install_file`, and `graph.install_dir` to
  project into the chosen install prefix model instead of only implicit paths

### Slice 37

- add integration coverage verifying install roots can move without changing
  build graph source

### Slice 38

- update docs to match the install model and explicitly distinguish:
  - build cache
  - fetched package store
  - install outputs

## Epoch 8: Reduce Ad Hoc Stringly Config Shapes

### Slice 39

- review artifact/dependency config records for fields still parsed mostly as
  loose strings
- identify where typed handles should replace strings

### Slice 40

- improve dependency config validation:
  - `source`
  - `target`
  - `args`
  - alias rules

### Slice 41

- improve artifact config validation for:
  - root source
  - fol model
  - target/optimize handles

### Slice 42

- add exact diagnostics for unsupported combinations instead of generic
  “unsupported build method” failures

## Epoch 9: Docs, Examples, And Hardening

### Slice 43

- add one standalone example focused on dependency handles

### Slice 44

- add one standalone example focused on generated-output handle composition

### Slice 45

- add one standalone example focused on dependency arg forwarding

### Slice 46

- add one standalone example focused on install prefix behavior

### Slice 47

- add negative examples for:
  - unknown dependency surface queries
  - invalid dependency args
  - invalid output-handle usage
  - invalid install projections

## Epoch 10: Cleanup And Final Audit

### Slice 48

- remove stale internal `.graph()` wording from build diagnostics/tests/docs
  that should now speak in terms of `.build()` and public handles

### Slice 49

- audit editor/LSP completion for the new build-surface methods and handle
  member names

### Slice 50

- final repo-wide scan for:
  - stale build docs
  - dead test helpers
  - obsolete dependency projection code
  - duplicate output-handle representations

## Suggested Execution Order

Recommended implementation order:

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

- dependency handles are already half-present internally
- output handles are the next biggest foundation
- dependency args depend on both
- install/output and cache reporting should come after the handle model is real

## Exit Criteria

This plan is complete when:

- `.build().add_dep(...)` returns a first-class public dependency handle
- dependency handles can query modules/artifacts/steps/generated outputs
- generated outputs flow through one unified public handle model
- dependency args can be forwarded explicitly and tested end to end
- step/cache reporting is materially stronger and documented honestly
- install/output layout is explicit and configurable
- docs/examples reflect the final surface without fallback language
