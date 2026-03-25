# PLAN: Build Entry Ambient Graph Redesign

Last updated: 2026-03-25

## Intent

This plan replaces the current canonical build entry:

```fol
pro[] build(graph: Graph): non
```

with the new canonical build entry:

```fol
pro[] build(): non
```

and adds a build-only ambient accessor:

```fol
.graph(): Graph
```

The goal is to remove the awkward injected-parameter shape from `build.fol`
without turning the build system into a separate language.

The new mental model is:

- `build.fol` is still ordinary FOL
- the build entry no longer receives a magic parameter
- the active build graph is accessed explicitly through `.graph()`
- graph methods stay on `Graph`
- old `build(graph: Graph)` is deleted, not preserved

## Non-Negotiable Rules

1. No compatibility path for `pro[] build(graph: Graph): non`.
2. No dual accepted signatures.
3. No fallback parser or validator behavior.
4. No mixed "sometimes ambient, sometimes injected" execution path.
5. `build.fol` remains ordinary FOL, not a separate DSL.

## New Canonical Surface

### Canonical entry

```fol
pro[] build(): non = {
    var graph = .graph();
    var app = graph.add_exe({
        name = "app",
        root = "src/main.fol",
        fol_model = "std",
    });
    graph.add_run(app);
}
```

### Direct-use form

```fol
pro[] build(): non = {
    .graph().add_exe({
        name = "app",
        root = "src/main.fol",
    });
}
```

### Helper routine form

`Graph` stays as a real build type for helper routines and local bindings:

```fol
fun[] add_app(graph: Graph, name: str, root: str): ArtifactHandle = {
    return graph.add_exe({ name = name, root = root });
}

pro[] build(): non = {
    var graph = .graph();
    var app = add_app(graph, "app", "src/main.fol");
    graph.add_run(app);
}
```

### Ambient helper form

Also valid if the helper wants ambient access:

```fol
fun[] add_app(name: str, root: str): ArtifactHandle = {
    return .graph().add_exe({ name = name, root = root });
}
```

## Scope Decision

This plan does **not** remove `Graph` as a build type.

It removes only the requirement that the build entry receive an injected graph
parameter.

That keeps the change focused and avoids unnecessary churn in:

- helper routine typing
- local graph bindings
- resolver/build stdlib type injection
- graph method semantics

## Current Reality From The Scan

The current `graph` parameter is baked into several layers:

- package/build entry validation in
  `lang/compiler/fol-package/src/build_entry.rs`
- package build parsing diagnostics in
  `lang/compiler/fol-package/src/build.rs`
- build stdlib injection and `Graph` type visibility in
  `lang/compiler/fol-resolver/src/inject.rs`
- restricted build execution in
  `lang/execution/fol-build/src/executor/eval_expr.rs`
- graph-method execution in
  `lang/execution/fol-build/src/executor/graph_methods.rs`
- build docs across `book/src/055_build/*` and module/package chapters
- examples, scaffolding, frontend tests, resolver tests, editor tests

One especially important detail:

- the restricted build executor currently recognizes graph access by comparing
  identifiers against `self.graph_param`
- helper execution has explicit special logic for "graph is the first helper
  parameter"

That means `.graph()` cannot be bolted on as just documentation. It must become
part of the executor/runtime contract.

## Design Direction

The clean design is:

1. canonical build entry is `pro[] build(): non`
2. `.graph()` becomes a build-only dot intrinsic or build-runtime ambient call
3. build executor owns one active graph handle implicitly
4. `Graph` type and graph methods remain unchanged
5. helper routines may still accept `graph: Graph`, but that becomes ordinary
   user choice, not required entry shape

## Architecture Decision

Use a build-only ambient accessor with ordinary call syntax:

- `.graph()`

Do **not** try to model it as:

- a fake implicit local named `graph`
- a source-unit variable injection
- a global function `graph()`
- direct ambient methods like `.add_exe(...)`

Why `.graph()` is the right shape:

- explicit but not awkward
- reads like other dot-root calls already in the language
- keeps graph methods grouped under the `Graph` handle
- avoids polluting normal identifier scope
- keeps build-only ambient capability obvious

## Epoch 1: Freeze The New Contract

Goal:
Write down the new canonical surface before implementation starts.

### Slice Tracker

- [x] Slice 1. Update `book/src/055_build/_index.md` to declare the new
  canonical entry:
  - `pro[] build(): non`
  - ambient `.graph(): Graph`
  - no `build(graph: Graph)` compatibility
- [ ] Slice 2. Update `book/src/055_build/100_build_file.md` to explain:
  - `build.fol` still uses ordinary FOL syntax
  - the graph is ambient through `.graph()`
  - the old injected parameter is deleted
- [ ] Slice 3. Update `book/src/600_modules/100_import.md` and
  `book/src/600_modules/200_blocks.md` so package/build chapters no longer
  reference `build(graph: Graph)`
- [ ] Slice 4. Update version/planning docs if needed so they describe the new
  build contract honestly

### Exit criteria

- The book no longer teaches the old entrypoint.
- The new contract is explicit before parser/runtime work begins.

## Epoch 2: Change Build Entry Validation

Goal:
Make the package/build loader accept only `pro[] build(): non`.

### Slice Tracker

- [ ] Slice 5. Change `BuildEntrySignatureExpectation` in
  `lang/compiler/fol-package/src/build_entry.rs` so the canonical entry has:
  - zero parameters
  - accepted return type `non`
- [ ] Slice 6. Rewrite parameter-count/type validation errors to describe the
  new required shape:
  - no parameters allowed
  - no graph parameter expected
- [ ] Slice 7. Update package build parsing diagnostics in
  `lang/compiler/fol-package/src/build.rs` to say:
  - `build.fol must declare exactly one canonical pro[] build(): non entry`
- [ ] Slice 8. Replace all build-entry validation tests so they reject
  `build(graph: Graph)` and accept `build(): non`

### Exit criteria

- The loader only accepts the new entry signature.
- Old `build(graph: Graph)` fails fast and explicitly.

## Epoch 3: Introduce Ambient `.graph()`

Goal:
Define `.graph()` as the one sanctioned way to access the active build graph.

### Slice Tracker

- [ ] Slice 9. Decide and implement where `.graph()` is modeled semantically:
  - as build-only ambient call metadata in `fol-build`
  - not as a normal source-level declared routine
- [ ] Slice 10. Extend build semantic metadata in
  `lang/execution/fol-build/src/semantic.rs` and `stdlib.rs` to describe
  `.graph(): Graph`
- [ ] Slice 11. Ensure resolver/typecheck for build source units recognize
  `.graph()` without requiring an injected local named `graph`
- [ ] Slice 12. Add parser/resolver/typecheck tests proving:
  - `.graph()` is valid in `build.fol`
  - `.graph()` is invalid in ordinary source units
  - graph methods still work through the returned handle

### Exit criteria

- `.graph()` is a real build-only surface.
- No injected identifier is needed to access the graph.

## Epoch 4: Remove Graph-Parameter Execution Semantics

Goal:
Delete the restricted-executor logic that relies on a graph parameter name.

### Slice Tracker

- [ ] Slice 13. Refactor `BuildBodyExecutor` in
  `lang/execution/fol-build/src/executor/eval_expr.rs` so graph access is not
  tied to `self.graph_param`
- [ ] Slice 14. Remove the special-case identifier logic:
  - `AstNode::Identifier { name } if name == &self.graph_param`
- [ ] Slice 15. Teach expression evaluation to recognize `.graph()` directly and
  return the active graph handle
- [ ] Slice 16. Remove helper-call special handling that treats "first param is
  graph" as an executor-level convention
- [ ] Slice 17. Add executor tests for:
  - `.graph().add_exe(...)`
  - `var g = .graph(); g.add_exe(...)`
  - helper routines that receive `Graph` explicitly
  - helper routines that call `.graph()` ambiently

### Exit criteria

- Restricted execution no longer depends on a magic parameter name.
- `.graph()` is the single ambient graph access path.

## Epoch 5: Rework Build Stdlib And Editor Semantics

Goal:
Align build stdlib/editor behavior with the ambient accessor model.

### Slice Tracker

- [ ] Slice 18. Keep `Graph` injected as a type in build stdlib scope, but stop
  depending on an entry parameter to make it usable
- [ ] Slice 19. Add editor/LSP completion coverage for `.graph()` inside
  `build.fol`
- [ ] Slice 20. Ensure editor diagnostics and symbol/navigation tests for build
  files use the new entry form and ambient graph access
- [ ] Slice 21. Update build-file completion helpers so they no longer assume a
  local identifier named `graph`

### Exit criteria

- `fol-editor` understands `.graph()` in build files.
- Editor tests no longer encode the old entrypoint.

## Epoch 6: Rewrite Examples, Scaffolding, And Fixtures

Goal:
Move all user-facing examples to the new build style.

### Slice Tracker

- [ ] Slice 22. Rewrite scaffolded `build.fol` templates in
  `lang/tooling/fol-frontend/src/scaffold.rs` to emit:
  - `pro[] build(): non`
  - `var graph = .graph();`
- [ ] Slice 23. Rewrite all checked-in examples under `examples/` to the new
  form
- [ ] Slice 24. Rewrite package fixtures under `test/apps`, `test/large_examples`,
  and `xtra/` to the new form
- [ ] Slice 25. Rewrite resolver/frontend/editor helpers that currently write
  synthetic build files using `build(graph: Graph)`
- [ ] Slice 26. Add focused positive examples for:
  - direct `.graph().add_exe(...)`
  - local binding `var graph = .graph()`
  - helper routine with explicit `Graph`
  - helper routine with ambient `.graph()`

### Exit criteria

- New projects and all examples teach only the new build style.
- No checked-in example still uses the deleted entry form.

## Epoch 7: Update Frontend And Build-Route Assumptions

Goal:
Make CLI/build routing depend on the new entry contract everywhere.

### Slice Tracker

- [ ] Slice 27. Update build-route error text in
  `lang/tooling/fol-frontend/src/build_route/mod.rs` to reference
  `pro[] build(): non`
- [ ] Slice 28. Update compile/fetch helpers and synthetic build fixtures in
  `fol-frontend` tests
- [ ] Slice 29. Add integration tests proving:
  - new-style build files plan and execute
  - old-style build files fail with the new canonical-entry error
- [ ] Slice 30. Verify routed `build/run/test/check` still work unchanged on top
  of the new graph-access mechanism

### Exit criteria

- CLI/build routing fully speaks the new entry shape.
- Error messages are consistent.

## Epoch 8: Hard Delete Old Surface

Goal:
Remove the last code/comments/tests that encode the injected-parameter model.

### Slice Tracker

- [ ] Slice 31. Remove remaining docs/comments that describe:
  - "Graph is the sole parameter to `pro[] build`"
  - helper conventions based on the old entry parameter
- [ ] Slice 32. Audit resolver/typecheck/build tests for lingering
  `return graph` / `build(graph: Graph)` fixtures and replace them
- [ ] Slice 33. Add regression tests that specifically fail if the old entry
  form starts parsing/validating/executing again
- [ ] Slice 34. Run a full repo grep audit and remove the last stale references
  to the old canonical entry from tracked source

### Exit criteria

- The old entrypoint is gone from implementation and tracked examples.
- Regression tests keep it gone.

## Open Design Questions To Resolve During Implementation

These should be answered early and then kept stable:

1. Should `.graph()` be callable only in `build.fol`, or in helper routines
   defined inside `build.fol` too?
   - recommendation: yes, anywhere inside build source units
2. Should ordinary helper routines still be allowed to accept `graph: Graph`?
   - recommendation: yes, keep this allowed
3. Should `.graph()` return the same logical handle identity every call?
   - recommendation: yes
4. Should there be any implicit local named `graph`?
   - recommendation: no
5. Should the book prefer:
   - direct `.graph().add_exe(...)`
   - or `var graph = .graph();`
   - recommendation: prefer local binding in most docs for readability

## Recommended Implementation Order

Do the work in this order:

1. docs and contract freeze
2. build-entry validator switch
3. `.graph()` semantic surface
4. restricted executor rewrite
5. frontend/build-route tests
6. editor/tests/examples/scaffolding rewrite
7. repo-wide deletion of old entry references

That order avoids a half-migrated state where docs/examples teach one shape but
the loader still requires another.

## Expected End State

When this plan is complete:

- every canonical build file uses `pro[] build(): non`
- ambient graph access is `.graph()`
- graph methods still live on `Graph`
- `Graph` remains a build-only type for helpers and locals
- the executor no longer depends on a magic parameter name
- scaffolding, examples, CLI tests, LSP tests, and the book all agree
- `pro[] build(graph: Graph): non` is deleted everywhere and rejected explicitly
