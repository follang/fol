# PLAN: Unify Package Metadata Into `.build()`

Last updated: 2026-03-25

## Intent

Replace the current two-file control model:

- `package.yaml`
- `build.fol`

with one canonical FOL control file:

- `build.fol`

The new public shape is:

```fol
pro[] build(): non = {
    var build = .build();

    build.meta({
        name = "app",
        version = "0.1.0",
        kind = "exe",
    });

    build.add_dep({
        alias = "shared",
        source = "loc",
        target = "../shared",
    });

    var graph = build.graph();
    var app = graph.add_exe({
        name = "app",
        root = "src/main.fol",
        fol_model = "std",
    });
    graph.install(app);
    graph.add_run(app);
};
```

## Core Rules

1. `package.yaml` is deleted completely.
2. Metadata lives in `build.fol` only.
3. Direct dependencies live in `build.fol` only.
4. The only canonical entry is still:
   - `pro[] build(): non`
5. Package metadata is configured through:
   - `.build().meta({...})`
6. Direct dependencies are configured through:
   - `.build().add_dep({...})`
7. Artifact graph access is configured through:
   - `.build().graph()`
8. No compatibility path is kept for `package.yaml`.

## Public Surface

### Canonical control flow

```fol
pro[] build(): non = {
    var build = .build();

    build.meta({
        name = "app",
        version = "0.1.0",
        kind = "exe",
        description = "demo",
        license = "MIT",
    });

    build.add_dep({
        alias = "json",
        source = "pkg",
        target = "json",
    });

    var graph = build.graph();
    var app = graph.add_exe({
        name = "app",
        root = "src/main.fol",
    });

    graph.install(app);
    graph.add_run(app);
};
```

### Semantics

- `.build()` is valid only in `build.fol`
- `build.meta({...})` sets package metadata
- `build.add_dep({...})` registers one direct dependency
- `build.graph()` returns the existing opaque build graph handle
- dependency preloading happens from `build.meta` / `build.add_dep`, not from YAML
- `build()` becomes the single control routine for:
  - package metadata
  - direct dependencies
  - artifact graph

## Non-Rules

These should become invalid:

- any `package.yaml`
- any loader behavior that requires `package.yaml`
- any dependency preload path that reads `package.yaml`
- any docs/examples that teach `package.yaml`

## Architecture Decision

Use one ambient build context with three responsibilities:

- package metadata
- direct dependency declarations
- graph construction

The clean public layering is:

- `.build()`
  - `.meta({...})`
  - `.add_dep({...})`
  - `.graph()`

Do not push metadata onto `.graph()`.
Do not keep a separate `package()` manifest routine.
Do not keep YAML parsing.

## Current Reality

Right now:

- formal packages require `package.yaml`
- metadata is parsed in `lang/compiler/fol-package/src/metadata.rs`
- package loading requires both `package.yaml` and `build.fol`
- dependency preloading comes from parsed metadata
- editor/root detection docs still mention `package.yaml`
- many examples/tests create `package.yaml`

So this is a full package-loading redesign, not just a syntax tweak.

## Epoch 1: Freeze The New Contract

Goal:
Write down the new one-file package model before code changes begin.

### Slice Tracker

- [x] Slice 1. Update book chapters that currently teach `package.yaml`:
  - `book/src/600_modules/100_import.md`
  - `book/src/600_modules/200_blocks.md`
  - `book/src/055_build/100_build_file.md`
- [x] Slice 2. Add docs for the new `.build()` ambient API:
  - `.build().meta({...})`
  - `.build().add_dep({...})`
  - `.build().graph()`
- [x] Slice 3. Update architecture/runtime docs that mention two control files
- [x] Slice 4. Update planning/version docs to state:
  - `package.yaml` is removed
  - `build.fol` is the only package control file

### Exit criteria

- The docs teach only the unified `.build()` model.

## Epoch 2: Define Build-Context Metadata Semantics

Goal:
Add semantic metadata for the new build context API.

### Slice Tracker

- [x] Slice 5. Add an internal opaque build-context handle alongside the existing graph handle
- [x] Slice 6. Define canonical build-context methods:
  - `meta`
  - `add_dep`
  - `graph`
- [x] Slice 7. Define canonical record shapes for:
  - `meta`
  - `add_dep`
- [x] Slice 8. Keep graph methods on the graph handle only, not on the build-context handle
- [x] Slice 9. Add semantic tests for:
  - build-context handle families
  - method lookup
  - config record shapes

### Exit criteria

- The build semantic catalog knows about the new `.build()` surface.

## Epoch 3: Replace Ambient `.graph()` With Ambient `.build()`

Goal:
Make `.build()` the top-level ambient entrypoint.

### Slice Tracker

- [x] Slice 10. Add `.build()` evaluation support in the build executor
- [x] Slice 11. Route `.build().graph()` to the existing internal graph handle
- [x] Slice 12. Keep inferred locals working:
  - `var build = .build();`
  - `var graph = build.graph();`
- [x] Slice 13. Add executor tests for:
  - `.build()`
  - `.build().graph()`
  - inferred locals for build and graph

### Exit criteria

- `.build()` is the new ambient root in `build.fol`.

## Epoch 4: Move Package Metadata Out Of YAML

Goal:
Teach the loader to get metadata from `build.fol` instead of `package.yaml`.

### Slice Tracker

- [x] Slice 14. Define the canonical `meta` record fields:
  - `name`
  - `version`
  - optional `kind`
  - optional `description`
  - optional `license`
- [x] Slice 15. Add extraction logic for `build.meta({...})`
- [x] Slice 16. Validate metadata constraints formerly enforced by YAML parsing:
  - required `name`
  - required `version`
  - valid package name
  - no duplicate metadata keys
- [ ] Slice 17. Replace `parse_package_metadata` usage in package loading with build metadata extraction
- [ ] Slice 18. Keep equivalent diagnostics for malformed metadata
- [ ] Slice 19. Add tests for:
  - valid metadata extraction
  - missing required metadata
  - invalid names
  - duplicate metadata declarations

### Exit criteria

- Formal package identity comes from `build.fol`, not YAML.

## Epoch 5: Move Direct Dependencies Out Of YAML

Goal:
Teach the loader to get direct dependencies from `build.fol`.

### Slice Tracker

- [ ] Slice 20. Define canonical `add_dep` record fields:
  - `alias`
  - `source`
  - `target`
- [ ] Slice 21. Support only current direct dependency source kinds:
  - `loc`
  - `pkg`
  - `git`
- [ ] Slice 22. Add extraction logic for `build.add_dep({...})`
- [ ] Slice 23. Replace metadata dependency preload paths with extracted build dependencies
- [ ] Slice 24. Keep validation for:
  - valid alias names
  - supported source kinds
  - non-empty targets
  - duplicate dependency aliases
- [ ] Slice 25. Add tests for:
  - local deps
  - package-store deps
  - git deps
  - duplicate alias rejection
  - malformed dependency rejection

### Exit criteria

- Direct dependency loading comes from `build.fol`, not YAML.

## Epoch 6: Delete `package.yaml` From Package Loading

Goal:
Remove YAML as a required or supported package control input.

### Slice Tracker

- [ ] Slice 26. Delete `lang/compiler/fol-package/src/metadata.rs` public usage from active loader paths
- [ ] Slice 27. Remove `package.yaml` file existence checks from formal package loading
- [ ] Slice 28. Require `build.fol` only for formal package roots
- [ ] Slice 29. Update package-session identity/display-name derivation to use build metadata
- [ ] Slice 30. Remove loader diagnostics that mention missing `package.yaml`
- [ ] Slice 31. Add tests proving:
  - formal packages load with `build.fol` only
  - roots with only YAML fail or are unsupported
  - dependency preloading still works without YAML

### Exit criteria

- `package.yaml` is no longer part of active loading.

## Epoch 7: Frontend, Fetch, Lockfile, And Store Integration

Goal:
Make all package-management flows work with `build.fol` metadata.

### Slice Tracker

- [ ] Slice 32. Update fetch/materialization flows that currently read/write assumptions about `package.yaml`
- [ ] Slice 33. Ensure lockfile/fetch diagnostics point at `build.fol` metadata when relevant
- [ ] Slice 34. Update package-store root validation for build-only metadata
- [ ] Slice 35. Update frontend diagnostics/help text away from `package.yaml`
- [ ] Slice 36. Add integration tests for:
  - git deps
  - package-store deps
  - lockfile flows
  - fetch flows
  under the new one-file package model

### Exit criteria

- package-management UX works without YAML.

## Epoch 8: Editor And Root Detection

Goal:
Make tooling stop treating `package.yaml` as a root marker or required package indicator.

### Slice Tracker

- [ ] Slice 37. Update editor/root discovery docs from:
  - `package.yaml`
  - `fol.work.yaml`
  toward the new build-only package marker
- [ ] Slice 38. Update editor integration tests and fixtures to use `build.fol` only
- [ ] Slice 39. Ensure LSP workspace/package detection still succeeds for formal packages
- [ ] Slice 40. Add regression tests proving editor features work for build-only package roots

### Exit criteria

- editor tooling no longer depends on `package.yaml`.

## Epoch 9: Examples, Fixtures, And Book Migration

Goal:
Replace checked-in YAML examples/fixtures with the new `build.fol` metadata model.

### Slice Tracker

- [ ] Slice 41. Rewrite examples under `examples/` to remove `package.yaml`
- [ ] Slice 42. Rewrite formal-package fixtures under `test/app`, `test/apps`, and `test/large_examples`
- [ ] Slice 43. Rewrite resolver/package/session fixtures that currently author YAML metadata
- [ ] Slice 44. Update book examples to show:
  - `build.meta({...})`
  - `build.add_dep({...})`
  - `build.graph()`
- [ ] Slice 45. Add positive example coverage for:
  - single package
  - loc dep
  - pkg dep
  - git dep
  - mixed models

### Exit criteria

- checked-in examples teach only the new one-file control model.

## Epoch 10: Hard Cleanup

Goal:
Delete the old YAML path entirely.

### Slice Tracker

- [ ] Slice 46. Remove public exports that only exist for YAML metadata parsing
- [ ] Slice 47. Remove stale comments and docs that mention `package.yaml` as required control metadata
- [ ] Slice 48. Add regression tests that fail if `package.yaml` becomes required again
- [ ] Slice 49. Run a full repo audit for:
  - `package.yaml`
  - metadata parser references
  - YAML-root assumptions
- [ ] Slice 50. Final consistency pass on diagnostics and scaffolding

### Exit criteria

- `package.yaml` is gone from the active package system.

## Expected End State

When this plan is complete:

- `build.fol` is the only package control file
- the only canonical entry is:
  - `pro[] build(): non`
- package metadata is declared through:
  - `.build().meta({...})`
- direct dependencies are declared through:
  - `.build().add_dep({...})`
- artifact graph access is declared through:
  - `.build().graph()`
- no package loader path reads `package.yaml`
- no docs/examples require `package.yaml`
- no compatibility path remains
