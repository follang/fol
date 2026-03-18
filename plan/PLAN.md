# FOL Build Reset Plan: Zig-Style New Build Only

Last updated: 2026-03-18

## Tracking

Current reset status: in progress

Round 1 slices:

- [x] Slice 1. Replace the remaining bullet-only tracking with a concrete 10-slice execution checklist
- [ ] Slice 2. Delete compatibility parsing and fallback extraction from `fol-package::build`
- [ ] Slice 3. Reject old `build.fol` forms by enforcing only `pro[] build(graph: Graph): non`
- [ ] Slice 4. Load package-store dependencies from `package.yaml`, not compatibility `def pkg = ...`
- [ ] Slice 5. Stop projecting compatibility exports/native artifacts through prepared package loading
- [ ] Slice 6. Remove compatibility/hybrid frontend routing and fallback default build execution
- [x] Slice 7. Rewrite frontend scaffolds and frontend unit tests to generate only the new build form
- [ ] Slice 8. Rewrite resolver and integration fixtures/tests that still depend on `def root`
- [x] Slice 9. Rewrite docs and examples so they teach only the new build model
- [x] Slice 10. Add semantic-only regression coverage for old syntax and invalid build entry shapes

Implemented already:

- [x] Canonical semantic build entry switched to `pro[] build(graph: Graph): non`
- [x] `xtra/logtiny` was converted to the new graph-driven build entry and `fol c b` works there
- [x] Checked-in `test/app/build/*` fixtures were rewritten to use the new build entry
- [x] End-to-end CLI tests now assert those fixtures use `pro[] build(graph: Graph): non`
- [x] Real `fol code build/run` coverage is passing on the rewritten fixtures
- [x] Resolver traversal now skips `build.fol` during ordinary package resolution so the CLI can route build files through the build evaluator path
- [x] Frontend build output no longer reports a deleted `emitted-rust` crate path

Not done yet:

- [ ] Reject `def build(...)` as invalid old syntax
- [ ] Reject `def root: loc = ...` as invalid old syntax
- [ ] Remove compatibility extraction from `fol-package`
- [ ] Remove compatibility/hybrid workflow branching from frontend routing
- [ ] Rehome formal package dependency/export loading off old `build.fol` defs
- [ ] Replace old pkg/import fixtures and tests that still depend on compatibility defs
- [ ] Rewrite docs/scaffolds/examples so they teach only the new build model
- [ ] Delete the remaining legacy build-only tests, fixtures, and code paths

## Core Decision

FOL will use a Zig-style build model:

- `build.fol` is the package build script
- the build entry is an ordinary routine, not a special definition record
- the build script mutates a build graph through a typed build API
- package metadata stays in `package.yaml`
- source roots are declared through the build API, not through legacy top-level `def root: loc = ...`

This repo is new. We do not carry a compatibility-root path.

## Zig Reference Model

Zig uses a real routine entrypoint in `build.zig`:

- `pub fn build(b: *std.Build) void`
- the build script mutates the builder object
- executables, libraries, steps, install actions, and user options are created through API calls
- there is no parallel “root declaration” compatibility surface for package layout

FOL should mimic that architecture, not the syntax literally.

The analogous FOL end state is:

```fol
pro[] build(graph: Graph): non = {
    var target = graph.standard_target();
    var optimize = graph.standard_optimize();

    var lib = graph.add_static_lib({
        name = "logtiny",
        root = "src/lib.fol",
        target = target,
        optimize = optimize,
    });

    graph.install(lib);
}
```

## Non-Negotiables

1. `def root: loc = "src"` is deleted as a supported build mechanism.
2. Top-level compatibility `def` records are not the package build model.
3. `package.yaml` is metadata only.
4. `build.fol` uses only the new semantic build path.
5. The canonical build entry is a routine, not `def build(...)`.
6. The frontend must not contain a second private build interpreter.
7. Every end-to-end build fixture must exercise only the new build entry.
8. No test may pass by silently falling back to compatibility-root behavior.

## Current Problems

These are the blockers that must be removed:

- the parser and package loader still assume `def build(...)` in several places
- top-level `def root: loc = ...` still drives real package loading
- some checked-in build fixtures still use old semantics
- the current tests can pass without exercising the new build path
- the frontend and package layer still contain compatibility-oriented branching
- documentation still describes compatibility defs as active build behavior

## End State

The finished system behaves like this:

1. `fol code build/run/test/check` requires `build.fol`.
2. `build.fol` must declare exactly one canonical build routine.
3. That routine is `pro[] build(graph: Graph): non = { ... }`.
4. The routine is parsed, resolved, typechecked, and evaluated as ordinary FOL.
5. Package source roots are declared through `graph.add_exe`, `graph.add_static_lib`, `graph.add_shared_lib`, `graph.add_test`, `graph.module`, or equivalent typed API calls.
6. Dependency wiring, install wiring, run wiring, generated files, and options are all expressed through the same build API.
7. If no artifact or module roots are declared, `fol code build` fails with a clear error.

## Canonical Build Surface

The canonical package build entry becomes:

```fol
pro[] build(graph: Graph): non = {
    // mutate graph
}
```

Why `pro`:

- Zig’s `build` is a procedure-like entrypoint that mutates builder state
- `fun` implies value-returning function semantics
- `def` is for definitions, but this build entry is operational routine code
- `pro` matches the architectural job better

Accepted build-file surface:

- exactly one top-level `pro[] build(graph: Graph): non = { ... }`
- optional helper `fun`/`pro`/`typ` declarations used by that routine
- ordinary imports

Not accepted:

- `def root: loc = ...`
- compatibility `def` package/export/build records
- `def build(...)`
- multiple canonical build routines

## Work Phases

## Phase 1: Lock The New Syntax

Goal:
- define the one true build entry

Required work:
- update build-entry validation to require `pro[] build(graph: Graph): non`
- reject `def build(...)` as invalid syntax for this project
- reject `def root: loc = ...` as invalid syntax for this project

Exit criteria:
- the canonical build entry is routine-based and enforced centrally

## Phase 2: Remove Compatibility-Root From Loading

Goal:
- make artifact/module roots come only from the build graph

Required work:
- delete package loading logic that derives package roots from `def root: loc = ...`
- delete compatibility extraction of root/export/dependency defs from `build.fol`
- require the build routine to declare roots through graph API calls
- fail clearly when a package has metadata but no build graph roots

Exit criteria:
- package loading no longer consults `def root`

## Phase 3: Align Parser, Resolver, And Typechecker

Goal:
- make the new build entry ordinary FOL

Required work:
- ensure top-level `pro` declarations in `build.fol` parse cleanly
- ensure helper routines/types/imports in `build.fol` are allowed
- define the build stdlib surface so resolver/typechecker can validate calls
- ensure `Graph`, artifact handles, step handles, options, and generated-file handles are real types

Exit criteria:
- `build.fol` passes the normal semantic pipeline with no special parser hacks

## Phase 4: Build Runtime On Real Routine Execution

Goal:
- execute the typed build routine, not a pattern extractor

Required work:
- move canonical build execution to a routine-evaluator path rooted in `pro[] build(...)`
- support ordinary statements, local vars, helper routine calls, record arguments, and method chaining used by the build API
- make graph mutation deterministic and capability-limited
- remove string/pattern extraction of the build body for the canonical path

Exit criteria:
- the build graph is produced by executing the build routine

## Phase 5: Frontend Routing Cleanup

Goal:
- make `fol code *` use only the new path

Required work:
- remove compatibility/hybrid workflow branching from frontend build routing
- route `build`, `run`, `test`, and `check` from the evaluated graph only
- require explicit graph-defined default steps
- keep frontend behavior aligned with build-runtime validation

Exit criteria:
- the frontend has one build flow

## Phase 6: Replace All Fixtures With New Build Packages

Goal:
- ensure tests prove the new system, not the deleted one

Required work:
- replace every checked-in `test/app/build/*` package that still uses `def root`
- add at least these real fixture packages:
  - executable package with `pro[] build(graph: Graph): non`
  - static library package with install step
  - workspace dependency package using `--package-store-root`
  - package with custom named step
  - package using build options
- ensure each fixture looks like a real library or application, even if small
- remove generated `.fol/` outputs from checked-in fixture directories

Exit criteria:
- every build fixture is new-build-only

## Phase 7: Add End-To-End Regression Coverage

Goal:
- make false positives impossible

Required work:
- add CLI tests that run `fol code build`
- add CLI tests that run `fol code run`
- add CLI tests that run `fol code check`
- add CLI tests that run `fol code test` when supported by the graph
- add explicit failure tests for:
  - `def root: loc = ...`
  - `def build(...)`
  - missing build routine
  - multiple build routines
  - build routine with wrong parameter type
  - build routine with wrong return type
- make the tests assert the failure text mentions the migration target

Exit criteria:
- semantic regression tests fail immediately if compatibility behavior sneaks back in

## Phase 8: Docs, Scaffolding, And Examples

Goal:
- make new projects generate only the right thing

Required work:
- rewrite the book sections that still describe compatibility defs as active behavior
- update scaffolded `build.fol` templates to emit `pro[] build(graph: Graph): non`
- update README examples to use new-build-only packages
- point docs at checked-in test fixtures as the source of truth
- document the Zig analogy explicitly:
  - Zig `pub fn build(b: *std.Build) void`
  - FOL `pro[] build(graph: Graph): non`

Exit criteria:
- docs and generated templates teach only the new model

## Phase 9: Delete Legacy Code

Goal:
- finish the migration cleanly

Required work:
- delete compatibility extraction code from `fol-package`
- delete compatibility-root branching from frontend planning and execution
- delete compatibility-only tests and fixtures
- delete docs that describe legacy build defs as supported behavior

Exit criteria:
- there is no supported legacy build path left in the repo

## Required First Execution Round

This is the first implementation sequence to run against the new plan:

1. Change canonical entry validation from `def build(...)` to `pro[] build(graph: Graph): non`.
2. Update parser/package/frontend diagnostics to reject `def build(...)` and `def root: loc = ...`.
3. Convert `xtra/logtiny` to the new routine entry and make `fol c b` work there.
4. Replace the three bogus build fixtures with real new-build-only packages.
5. Add end-to-end CLI tests that prove those packages build through the new path.
6. Remove any temporary fallback added during the conversion.

## Success Definition

We are done only when all of this is true:

- a fresh package can build with only `package.yaml` and `build.fol`
- `build.fol` uses `pro[] build(graph: Graph): non`
- no checked-in example or fixture uses `def root: loc = ...`
- no checked-in example or fixture uses `def build(...)`
- `fol code build/run/test/check` pass on real new-build-only fixtures
- compatibility-root code is deleted
- docs teach only the new model
