# PLAN: Opaque Ambient Build Graph

Last updated: 2026-03-25

## Intent

This plan replaces the current public build entry model:

```fol
pro[] build(graph: Graph): non
```

with:

```fol
pro[] build(): non
```

and a build-only ambient accessor:

```fol
.graph()
```

The key rule is:

- `Graph` is not a public language type
- `Graph` is not nameable by users
- `Graph` is valid only as an internal compiler/runtime concept
- user-visible build graph access happens only through `.graph()`
- `.graph()` is valid only in `build.fol`

This keeps the build system ordinary FOL while removing the awkward injected
parameter and avoiding collisions with user-defined types named `Graph`.

## Required Language Contract

### Canonical build entry

```fol
pro[] build(): non = {
    var graph = .graph();
    var app = graph.add_exe({
        name = "app",
        root = "src/main.fol",
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

### Non-rules

These should be invalid:

```fol
pro[] build(graph: Graph): non = { ... }
fun[] helper(graph: Graph): non = { ... }
var graph: Graph = .graph();
```

So:

- inferred locals are allowed
- explicit `Graph` type syntax is not allowed
- helper routines cannot name `Graph`

## Core Principles

1. `.graph()` is the only public access path to the build graph.
2. `.graph()` is valid only in `build.fol`.
3. `Graph` must not exist as a public injected build type.
4. The compiler may keep any internal runtime/semantic graph type it wants.
5. Old `build(graph: Graph)` is deleted with no compatibility path.

## Current Reality

The current repo still assumes a public `Graph` surface in many places:

- build entry validation
- package build parsing errors
- resolver build-stdlib injection
- restricted build executor graph-parameter logic
- editor tests and build-file completion
- scaffolded `build.fol`
- examples and fixtures
- book chapters

That means this is a real semantic migration, not a docs-only rename.

## Architecture Decision

Use this exact model:

- user-facing surface:
  - `.graph()`
  - methods on the returned handle
- internal implementation:
  - any opaque build-graph handle representation
- forbidden user surface:
  - public `Graph` type
  - explicit `Graph` annotations
  - build entry parameters

## Epoch 1: Freeze The New Contract

Goal:
Write down the real public contract before code changes begin.

### Slice Tracker

- [x] Slice 1. Update `book/src/055_build/_index.md` to say:
  - canonical entry is `pro[] build(): non`
  - graph access is `.graph()`
  - `Graph` is not a public type
- [x] Slice 2. Update `book/src/055_build/100_build_file.md` to explain:
  - `build.fol` is still ordinary FOL
  - `.graph()` is build-only
  - explicit `Graph` annotations are not part of the language surface
- [x] Slice 3. Update package/module chapters in:
  - `book/src/600_modules/100_import.md`
  - `book/src/600_modules/200_blocks.md`
  so they stop teaching `build(graph: Graph)`
- [x] Slice 4. Update version/planning docs if they mention public `Graph`

### Exit criteria

- The book teaches only `.graph()` and `build(): non`.

## Epoch 2: Remove Public Graph From Build Validation

Goal:
Make the build loader accept only `pro[] build(): non`.

### Slice Tracker

- [x] Slice 5. Change build-entry validation in
  `lang/compiler/fol-package/src/build_entry.rs` to require:
  - zero parameters
  - return type `non`
- [x] Slice 6. Rewrite validation errors to describe:
  - no parameters allowed
  - old `build(graph: Graph)` is invalid
- [x] Slice 7. Update package build parse errors in
  `lang/compiler/fol-package/src/build.rs` to reference
  `pro[] build(): non`
- [x] Slice 8. Update all build-entry validation tests to:
  - accept `build(): non`
  - reject `build(graph: Graph)`

### Exit criteria

- The loader no longer encodes public `Graph`.

## Epoch 3: Add `.graph()` As A Build-Only Surface

Goal:
Make `.graph()` the only public graph access path.

### Slice Tracker

- [x] Slice 9. Define `.graph()` in `fol-build` semantic metadata as a
  build-only ambient call
- [x] Slice 10. Ensure resolver/typecheck recognize `.graph()` only for build
  source units
- [x] Slice 11. Ensure ordinary source units reject `.graph()` explicitly
- [x] Slice 12. Add tests for:
  - `.graph()` valid in `build.fol`
  - `.graph()` invalid outside `build.fol`
  - `.graph().add_exe(...)` valid in build files

### Exit criteria

- `.graph()` is real and build-only.

## Epoch 4: Make The Returned Handle Opaque

Goal:
Keep graph access usable without making `Graph` a public type name.

### Slice Tracker

- [x] Slice 13. Remove public build-stdlib injection of a user-visible `Graph`
  type from resolver semantics
- [x] Slice 14. Keep the internal semantic/runtime graph receiver type opaque to
  source-level type syntax
- [x] Slice 15. Allow inferred locals from `.graph()`:
  - `var graph = .graph();`
- [x] Slice 16. Reject explicit type spellings like:
  - `var graph: Graph = .graph();`
  - helper params/returns using `Graph`
- [x] Slice 17. Add tests proving:
  - inferred local binding works
  - explicit `Graph` annotations fail
  - a user-defined ordinary `Graph` type in non-build code is unaffected

### Exit criteria

- The graph handle is usable but not nameable by users.

## Epoch 5: Remove Graph-Parameter Execution Logic

Goal:
Delete executor behavior centered on the injected graph parameter.

### Slice Tracker

- [x] Slice 18. Refactor `BuildBodyExecutor` so graph access is no longer tied
  to `self.graph_param`
- [x] Slice 19. Remove special identifier handling for the graph parameter
- [x] Slice 20. Evaluate `.graph()` directly to the active internal graph handle
- [x] Slice 21. Remove helper-call conventions that depended on graph being the
  first helper parameter
- [x] Slice 22. Add executor tests for:
  - direct `.graph().method(...)`
  - local inferred graph handle
  - no public `Graph` parameter path

### Exit criteria

- Execution is ambient and opaque, not parameter-based.

## Epoch 6: Rewrite User-Facing Surfaces

Goal:
Move scaffolding, examples, and tests to the new design.

### Slice Tracker

- [x] Slice 23. Rewrite frontend scaffolding to emit:
  - `pro[] build(): non`
  - `.graph()` access
- [x] Slice 24. Rewrite examples under `examples/`
- [x] Slice 25. Rewrite fixtures under `test/apps`, `test/large_examples`, and
  `xtra/`
- [x] Slice 26. Rewrite synthetic build-file helpers across frontend/editor
  tests
- [x] Slice 27. Add example coverage for:
  - direct `.graph()` use
  - inferred local graph binding
  - user-defined non-build `Graph` type without collisions

### Exit criteria

- No checked-in example teaches public `Graph`.

## Epoch 7: Update Editor And Frontend Expectations

Goal:
Make tooling reflect the new public surface.

### Slice Tracker

- [x] Slice 28. Update `fol-editor` build-file completion and diagnostics for
  `.graph()`
- [x] Slice 29. Remove editor assumptions that `Graph` is a public build type
- [x] Slice 30. Update frontend/build-route errors and fixtures to reference
  `build(): non`
- [x] Slice 31. Add integration tests for:
  - new-style build files
  - old-style build files rejected
  - `.graph()` not usable outside `build.fol`

### Exit criteria

- CLI and editor agree on the new build contract.

## Epoch 8: Full Repo Cleanup

Goal:
Delete the last stale references to public `Graph`.

### Slice Tracker

- [x] Slice 32. Remove docs/comments that still describe `Graph` as public
- [x] Slice 33. Replace remaining tests that reference `build(graph: Graph)`
- [x] Slice 34. Add regression tests that fail if public `Graph` comes back
- [x] Slice 35. Run a full repo audit and remove remaining stale public-Graph
  references

### Exit criteria

- `Graph` is no longer part of the public build language.

## Expected End State

When this plan is complete:

- build entry is `pro[] build(): non`
- `.graph()` is the only public graph access surface
- `.graph()` is valid only in `build.fol`
- the returned graph handle supports build methods
- the graph handle can be inferred in locals
- `Graph` is not a public type name
- user code can define its own `Graph` without collision
- old `build(graph: Graph)` is rejected everywhere
