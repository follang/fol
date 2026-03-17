# FOL Build Plan: `build.fol` Must Be Real FOL

Last updated: 2026-03-17

## Core Decision

`build.fol` must work the way `build.zig` works in Zig at the architectural
level:

- it is a real source file in the language
- it is parsed by the normal parser
- it is resolved and typechecked by the normal semantic pipeline
- its `def build(...)` body is executed as real language code
- that execution is constrained by a build-runtime API and produces a build graph

What we must **not** ship as the final design:

- a forever-special string extractor
- a frontend-only mini interpreter
- a separate fake DSL that only looks like FOL
- duplicated “build semantics” outside the real compiler/runtime path

The current repository has useful groundwork, but it is still short of this
goal. This plan replaces the older roadmap and focuses only on closing that
semantic gap.

## Current Truth At Head

What is already true:

- `build.fol` is lexed and parsed as ordinary FOL source text
- editor tooling can parse/highlight/symbol-complete it
- `fol-package` can load `build.fol` and detect compatibility controls plus the
  canonical `def build(...)` entry
- the repo has a build graph IR, build API surface, option model, artifact
  model, step model, dependency model, generated-file model, and native model
- `fol code build/run/test/check` can route modern and hybrid packages through a
  graph-backed path
- a shared restricted build-source evaluator exists in `fol-package`
- that restricted evaluator now supports:
  - plain method calls like `graph.add_exe("app", "src/app.fol")`
  - object-style artifact creation like `graph.add_exe({ name = "...", root = "..." })`
  - simple handle-style follow-up calls like `graph.install(app)` and
    `graph.add_run(app)`

What is still false:

- `build.fol` is not yet resolved/typechecked/executed as an ordinary FOL
  program
- the build graph is not yet produced by normal language execution
- arbitrary valid FOL code inside `def build(...)` does not work
- method chains like `graph.step(...).depend_on(...)` are not real language
  semantics yet
- the current evaluator still recognizes patterns instead of executing typed FOL

That last bullet is the real problem.

## End State

The correct finished model is:

1. `build.fol` is loaded as a normal package-oriented FOL source unit.
2. The normal parser produces normal AST.
3. The normal resolver resolves imports, names, namespaces, and types inside the
   build file.
4. The normal typechecker validates `def build(...)` against a real build stdlib
   surface.
5. A dedicated build-runtime evaluator executes the typed/lowered build routine
   in a controlled environment.
6. That runtime mutates a `build::Graph` object through ordinary method calls and
   ordinary FOL values.
7. The produced graph is validated and then executed by `fol code
   build/run/test/check`.

This gives FOL real Zig-style parity in the only sense that matters:

- not “same syntax”
- but “same architectural truth”

## Non-Negotiable Constraints

1. `build.fol` stays ordinary FOL syntax.
2. The normal compiler pipeline stays the source of truth.
3. Build execution is deterministic and capability-limited.
4. Package metadata stays in `package.yaml`; build behavior lives in `build.fol`.
5. This project is still very new and does not need backward-compatibility
   preservation as a product requirement.
6. We do not keep legacy implementations just because they existed first. If a
   new semantic path replaces an older extracted/fallback path, the older path
   should be deleted.
7. Existing compatibility defs like `def x: pkg = ...` and `def y: loc = ...`
   may exist during migration, but they must become a subset of the real build
   model, not a permanent parallel system, and duplicate legacy routing should
   be removed instead of maintained.
8. The frontend must not own private build semantics once the shared runtime is
   in place.

## Why Zig Matters Here

Zig does not scrape `build.zig` for patterns. Zig runs real Zig code with a
build API.

FOL must do the analogous thing:

- `build.fol` must be real FOL code
- `def build(...)` must be a real routine
- `graph.add_exe(...)`, `graph.install(...)`, `graph.add_run(...)`,
  `graph.step(...).depend_on(...)`, and similar calls must be validated by the
  real resolver/typechecker and executed by a real evaluator

If we stop short of that, we do not have the right system.

## Legacy Policy

This plan assumes an aggressive cleanup policy.

We are not optimizing for backward compatibility because:

- the project is very new
- there is no meaningful external compatibility burden yet
- carrying parallel old and new implementations will slow down the architecture
  and make the final system harder to reason about

Therefore:

1. Do not preserve old build paths just because they already work.
2. Do not keep fallback implementations once the real semantic path is ready.
3. Do not add compatibility shims unless they are strictly temporary and on the
   shortest path to deletion.
4. Prefer replacement plus deletion over coexistence.
5. Every migration phase should identify which temporary code gets removed at
   the end of that phase.

The default bias must be:

- build the correct path
- switch to it
- delete the obsolete path

## New Execution Architecture

The target architecture should be:

1. Source loading layer
   - load `build.fol` into a dedicated build-package preparation path
   - keep compatibility extraction available only as migration scaffolding

2. Semantic build compilation layer
   - parse `build.fol`
   - resolve against normal imports plus a build stdlib surface
   - typecheck against the real `build` API types

3. Build-runtime lowering/evaluation layer
   - lower the typed build routine into an interpreter-friendly form
   - execute only the allowed build-time subset
   - materialize graph mutations and option reads deterministically

4. Graph execution layer
   - validate graph
   - select requested step
   - execute artifact/step/install/run/test/codegen actions

The key distinction:

- semantic build compilation answers “is this valid FOL build code?”
- build runtime answers “what graph does this valid build code produce?”

## Work Phases

## Phase 0: Freeze The Direction

Goal:
- remove ambiguity about the end state

Required outcomes:
- all docs and progress notes say explicitly that the goal is real FOL semantic
  execution, not a permanently restricted extractor
- future work is measured against “does this move semantics into the real
  pipeline?”

Exit criteria:
- this plan is the active source of truth

## Phase 1: Define The Real Build Stdlib Surface

Goal:
- make the build API type surface concrete enough that resolver/typechecker can
  reason about it

Required work:
- define the canonical `std` build package path and module surface
- define public build types:
  - `build::Graph`
  - artifact handle types
  - step handle types
  - run handle types
  - install handle types
  - option handle/value types
  - dependency handle types
- define canonical method signatures for:
  - `standard_target`
  - `standard_optimize`
  - `option`
  - `add_exe`
  - `add_static_lib`
  - `add_shared_lib`
  - `add_test`
  - `step`
  - `add_run`
  - `install`
  - `install_file`
  - `install_dir`
  - `dependency`
  - generated-file / codegen / tool methods
- define whether object-style argument records are nominal types or structural
  record values
- define the chaining surface:
  - `graph.step(...).depend_on(...)`
  - `graph.add_run(...).step` if needed
  - `graph.install(...).step` if needed

Tests required:
- resolver tests for build stdlib imports
- typechecker tests for build API method calls
- typechecker tests for object-style config records
- typechecker tests for method chaining on handles

Exit criteria:
- `build.fol` signatures can be typechecked against real build API types

Round 1 slice tracker:

- [x] Slice 1. Add a concrete Phase 1 implementation tracker with completion
  rules.
- [x] Slice 2. Add canonical semantic build stdlib module identity types.
- [x] Slice 3. Add semantic build surface type families for graph/handles.
- [x] Slice 4. Add semantic method signature types for the build stdlib.
- [x] Slice 5. Add canonical graph method signatures.
- [x] Slice 6. Add canonical handle method signatures.
- [x] Slice 7. Add object-style artifact config shape types.
- [x] Slice 8. Add option value/config shape types for semantic build calls.
- [x] Slice 9. Add chaining metadata for `.depend_on(...)` and related flows.
- [ ] Slice 10. Re-export and test the full Phase 1 semantic build surface.

## Phase 2: Admit `build.fol` Into The Normal Semantic Pipeline

Goal:
- stop treating `build.fol` as semantically special-cased text

Required work:
- create a dedicated package/source-unit kind for build units if needed, but keep
  them in the normal parser/resolver/typechecker flow
- load `build.fol` into the prepared workspace in a way that normal semantic
  stages can see it
- decide and implement visibility rules:
  - can `build.fol` import package source modules?
  - can package source import from `build.fol`?
  - likely answer: build can see build stdlib and dependency build surfaces, but
    ordinary package code cannot depend on build internals
- ensure build source units are excluded from ordinary runtime artifact lowering
  unless explicitly needed

Tests required:
- package preparation tests for build source units
- resolver tests proving build units are visible where intended and hidden where
  not intended
- typechecker tests proving ordinary packages still reject inappropriate build
  symbols

Exit criteria:
- `build.fol` reaches resolver and typechecker as a real source unit

## Phase 3: Locate And Validate The Canonical Build Entry

Goal:
- make `def build(...)` a semantically validated entry routine

Required work:
- identify the canonical build entry after parse/resolution, not by raw-source
  fallback
- validate:
  - exactly one required build entry
  - allowed parameter shape
  - allowed return type
  - disallow ambiguous overload-like shapes if the language permits them later
- emit proper semantic diagnostics with source locations

Tests required:
- zero build entry
- multiple build entries
- wrong parameter type
- wrong return type
- malformed build routine body with normal semantic diagnostics

Exit criteria:
- `build.fol` entry selection is semantic, not textual

## Phase 4: Build-Time Capability Model

Goal:
- define what build code is allowed to do at runtime

Required work:
- specify allowed categories:
  - graph mutation
  - option reads
  - deterministic path operations
  - deterministic string/container operations
  - controlled generated-file emission
  - controlled external tool invocation
- specify forbidden categories:
  - arbitrary filesystem reads/writes
  - arbitrary network
  - wall-clock access
  - ambient environment access outside declared inputs
  - uncontrolled process execution
- define the input envelope:
  - package root
  - working directory
  - declared options
  - target/optimize
  - selected environment variables if any

Tests required:
- diagnostics for forbidden runtime surfaces
- deterministic key tests for identical inputs
- differing determinism keys when declared inputs differ

Exit criteria:
- build runtime permissions are explicit and testable

## Phase 5: Real Build Routine Evaluation

Goal:
- execute typed build code rather than extracting patterns

Required work:
- choose the execution representation:
  - interpret typed AST directly, or
  - lower build routines into a restricted runtime IR and interpret that
- implement evaluation of:
  - local variable bindings
  - record literals used for build configs
  - method calls on `build::Graph` and handles
  - simple expression flow needed by build scripts
  - handle passing through locals
- preserve deterministic state updates into the build graph

Initial supported runtime subset should cover:
- `var target = graph.standard_target()`
- `var optimize = graph.standard_optimize()`
- object-style `add_exe` / library / test calls
- `graph.install(app)`
- `graph.add_run(app)`
- `var step = graph.step(...)`
- `step.depend_on(...)`

Tests required:
- evaluator tests for local handle flow
- evaluator tests for method chaining
- evaluator tests for object config records
- evaluator tests for repeated and aliased handle usage

Exit criteria:
- the current restricted string extractor is no longer needed for supported
  build scripts

## Phase 6: Remove Textual Build-Body Extraction

Goal:
- delete the temporary extractor path once semantic evaluation covers the needed
  surface

Required work:
- switch `fol-package` build evaluation to the semantic runtime path
- remove raw-source build-body extraction from the active execution path
- keep only minimal compatibility scanning for top-level migration-only controls
  if still needed

Tests required:
- prove modern packages execute without textual extraction
- prove editor and CLI behavior stays stable

Exit criteria:
- supported modern `build.fol` execution no longer depends on line-based pattern
  scraping
- the old extractor implementation is deleted, not merely bypassed

## Phase 7: Make Frontend Commands Fully Graph-Driven

Goal:
- ensure `fol code build/run/test/check` execute the graph, not legacy workspace
  assumptions

Required work:
- route command selection through evaluated graph steps only
- make default command mapping explicit:
  - `fol code build` requests step `build`
  - `fol code run` requests step `run`
  - `fol code test` requests step `test`
  - `fol code check` requests step `check`
- support custom named steps through CLI `--step`
- remove remaining implicit `src/main.fol` fallback assumptions except where
  intentionally preserved as default graph synthesis for packages without modern
  build logic

Tests required:
- single-artifact build/run/test/check
- custom named steps
- multiple artifacts with explicit step selection
- modern/hybrid packages with no compatibility fallback

Exit criteria:
- workspace commands are graph-driven by default

## Phase 8: Step Handles And Chaining

Goal:
- support the natural builder style users actually expect

Required work:
- implement chained semantics such as:
  - `graph.step("run", "Run the app").depend_on(run_app)`
  - `graph.install(app).step` if that becomes the API shape
  - run/install/test handles participating in step dependencies
- define stable handle identity rules
- define whether methods mutate in place or return updated handles

Tests required:
- chained step creation and dependency wiring
- dependencies declared through handle methods instead of raw names
- duplicate dependency handling

Exit criteria:
- documented chained build style works as real FOL code

## Phase 9: Options As Real Values

Goal:
- make build options real semantic values, not placeholders

Required work:
- typecheck `target`, `optimize`, bool/int/string/path/enum options as real
  build values
- permit passing options through object config records
- define how option defaults, overrides, and reads behave during evaluation

Tests required:
- CLI overrides reaching build runtime
- option values flowing through local variables
- option values inside `add_exe({ ... })`

Exit criteria:
- options participate in real semantic evaluation

## Phase 10: Dependency Build Surfaces

Goal:
- allow one package’s build graph to consume another package’s exported build
  surface semantically

Required work:
- define dependency build imports and handle visibility
- expose modules/artifacts/steps/generated outputs from dependency builds
- determine eager vs lazy dependency build evaluation

Tests required:
- dependency artifact consumption
- dependency step wiring
- dependency generated-file consumption

Exit criteria:
- dependency build surfaces are usable from real build code

## Phase 11: Generated Files, Tools, And Native Inputs

Goal:
- cover the rest of the build graph surface through the real evaluator

Required work:
- generated file actions
- codegen requests
- controlled system-tool actions
- native include/lib/link surfaces
- test/docs/install style expansion as needed

Tests required:
- generated file feeding later artifact creation
- codegen outputs consumed by artifacts
- controlled external tool outputs
- native attachment propagation

Exit criteria:
- major build graph node families are usable from semantic build execution

## Phase 12: Compatibility Absorption

Goal:
- fold old package-control behavior into the real build model and delete parallel
  legacy structure

Required work:
- decide how top-level `pkg` / `loc` compatibility defs map into real build
  semantics
- migrate package preparation to derive exports/dependencies from the real build
  model where possible
- delete duplicate compatibility code paths that no longer add value

Tests required:
- compatibility packages still load during migration
- hybrid packages prefer semantic build execution
- export/dependency behavior remains correct

Exit criteria:
- compatibility behavior is a subset of the real model, not a separate system
- superseded legacy code paths are removed from the repository

## Phase 13: Product Completion Criteria

We are done only when all of these are true:

1. `build.fol` is parsed, resolved, typechecked, and evaluated through the real
   compiler/runtime path.
2. The active execution path does not depend on textual build-body extraction.
3. `fol code build/run/test/check` operate on the evaluated graph by default.
4. Object-style artifact configs, option values, handle variables, and chained
   step wiring work in real build code.
5. Dependency surfaces, generated files, and install/run/test steps work through
   that same semantic path.
6. The old compatibility surface is either absorbed or intentionally tiny and
   non-duplicative.

## Immediate Implementation Order

This is the recommended order to execute from current head:

1. Phase 1: lock the real build stdlib surface and chaining API
2. Phase 2: admit `build.fol` into resolver/typechecker as a real semantic unit
3. Phase 3: semantic entry validation for `def build(...)`
4. Phase 4: finalize build-time capability boundaries
5. Phase 5: implement real build routine evaluation over typed/lowered code
6. Phase 6: delete textual extraction from the active execution path
7. Phase 7: make frontend commands fully graph-driven
8. Phase 8 onward: expand chains, options, dependencies, generated files,
   native/tooling, and compatibility absorption

## Progress Tracking Template

When work starts on this new plan, progress should be reported against phases,
not vague percentages.

Recommended tracking format:

- Phase 1: not started / in progress / complete
- Phase 2: not started / in progress / complete
- Phase 3: not started / in progress / complete
- Phase 4: not started / in progress / complete
- Phase 5: not started / in progress / complete
- Phase 6: not started / in progress / complete
- Phase 7: not started / in progress / complete
- Phase 8: not started / in progress / complete
- Phase 9: not started / in progress / complete
- Phase 10: not started / in progress / complete
- Phase 11: not started / in progress / complete
- Phase 12: not started / in progress / complete

## Final Standard

If a future implementation still needs to ask “can the restricted extractor
understand this build pattern?”, the plan is not complete.

The correct question is:

- “is this valid FOL build code, and if so, what graph does its execution
  produce?”
