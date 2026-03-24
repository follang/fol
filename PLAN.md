# PLAN: Compiler-Backed Editor Sync

Last updated: 2026-03-24

## Intent

This plan replaces the current "remember to update `fol-editor` manually"
workflow with a stricter model:

- LSP semantic behavior should come from the real compiler pipeline whenever
  possible.
- `fol_model` awareness must be visible in editor diagnostics, hover,
  completion, and semantic analysis.
- tree-sitter should not be fully generated from the compiler, but all
  repetitive editor registries and query fragments that can be derived from
  compiler-owned data should become generated or compiler-backed.
- drift between compiler features and editor syntax assets should become a test
  failure, not tribal knowledge.

The end state is not:

- fully hand-maintained editor metadata forever
- a second language implementation inside `fol-editor`
- full tree-sitter grammar generation from the parser
- "maybe update the editor later"

The end state is:

- compiler-owned semantic truth
- model-aware LSP behavior
- generated or compiler-derived editor registries where practical
- manual grammar/query work only where structure really must stay hand-authored
- aggressive sync tests that fail when compiler and editor drift

## Core Principles

1. `fol-editor` should consume compiler-owned facts before inventing editor-only
   ones.
2. `fol_model = core | alloc | std` must affect editor diagnostics and semantic
   tooling exactly the same way it affects `fol code build`.
3. Manual grammar authoring is acceptable.
4. Manual keyword/type/intrinsic lists are not acceptable if they can be
   derived from compiler registries.
5. No compatibility editor paths for stale syntax once a new syntax form is
   chosen.
6. Every behavior-changing slice includes tests in the same change.

## Current Baseline

Already true today:

- `fol-editor` LSP semantic analysis reuses parser, package loading, resolver,
  and typechecker from the real compiler.
- tree-sitter sync tests already compare some editor assets against compiler
  constants:
  - builtin types
  - implemented dot intrinsics
  - container type names
  - shell type names
  - source kinds
- tree-sitter assets live in one place:
  - `lang/tooling/fol-editor/tree-sitter/grammar.js`
  - `lang/tooling/fol-editor/queries/fol/*.scm`

Not good enough yet:

- editor behavior is not explicitly `fol_model` aware in all user-facing areas
- completions and some query families still rely on hand-maintained surfaces
- tree-sitter query files still embed lists that can drift from compiler data
- feature additions can still succeed in the compiler while the editor layer
  silently lags
- there is no canonical generated editor-metadata pipeline

## Boundaries

Things this plan should automate:

- keyword/type/intrinsic registries used by editor features
- model-aware completion and diagnostics policy
- query fragment generation for regex-like name sets
- test enforcement that editor assets match compiler data

Things this plan should not try to automate fully:

- complete tree-sitter grammar generation from the parser
- all highlight capture structure
- all LSP UX behavior
- formatter policy

## Required Workflow

For every implementation slice:

- keep the slice commit-sized
- if behavior changes, add tests in the same slice
- after each slice:
  - run `make build`
  - run `make test`
  - if both pass:
    - mark the slice complete here
    - commit it

## Epoch 1: Freeze The Editor Contract

Goal:
Write down exactly what is compiler-owned, what remains manual, and where
`fol_model` must surface in editor behavior.

### Slice Tracker

- [x] Slice 1. Replace or expand editor docs so they explicitly state:
  - LSP semantic truth comes from compiler analysis
  - tree-sitter grammar stays manual
  - query fragments and registries should be compiler-derived where possible
  - `fol_model` must affect editor diagnostics and completion
- [x] Slice 2. Add a canonical doc, likely `docs/editor-sync.md`, covering:
  - compiler-owned editor data
  - generated editor data
  - manual editor data
  - test gates that prevent drift
- [ ] Slice 3. Add a matrix to the doc for `core`, `alloc`, `std`:
  - which type surfaces should appear in completion
  - which intrinsics should appear in completion
  - which diagnostics should be shown by analysis
  - which example packages should exercise each mode

### Exit criteria

- The intended editor architecture is written down.
- `fol_model` expectations are explicit before code changes start.

## Epoch 2: Introduce Compiler-Owned Editor Metadata

Goal:
Create one compiler-backed metadata surface that `fol-editor` can consume.

### Slice Tracker

- [ ] Slice 4. Add a compiler-owned editor metadata API, probably in an existing
  compiler crate or a new small compiler-side module, exposing:
  - declaration keywords
  - builtin type names
  - container type names
  - shell type names
  - source kind names
  - implemented intrinsic names grouped by surface
- [ ] Slice 5. Extend that metadata API to carry `fol_model` capability facts:
  - whether a type family is legal in `core`
  - whether an intrinsic is legal in `core`
  - whether an intrinsic is `std`-only
  - whether a completion item should be suppressed by model
- [ ] Slice 6. Add unit tests proving the metadata API is canonical and matches
  existing compiler registries exactly.

### Exit criteria

- Editor-relevant language facts have one compiler-owned source.
- `fol_model` capability facts exist in machine-readable form.

## Epoch 3: Make LSP Model-Aware

Goal:
Ensure editor semantic analysis knows which model the current artifact/package is
using and reports boundaries consistently.

### Slice Tracker

- [ ] Slice 7. Extend editor workspace analysis so it can recover the active
  `fol_model` for the opened package or routed artifact.
- [ ] Slice 8. Thread `fol_model` into semantic snapshots and editor workspace
  caches.
- [ ] Slice 9. Add tests proving LSP diagnostics for:
  - `.echo(...)` in `alloc`
  - `str` in `core`
  - dynamic `.len(...)` in `core`
  match build-mode diagnostics for the same package.
- [ ] Slice 10. Add hover or symbol-surface tests proving model context is not
  lost when analyzing package files under `core`, `alloc`, and `std`.

### Exit criteria

- LSP analysis is explicitly aware of `fol_model`.
- Editor diagnostics agree with compiler/build diagnostics for model boundaries.

## Epoch 4: Make Completion Model-Aware

Goal:
Stop offering completion items that are invalid for the current model.

### Slice Tracker

- [ ] Slice 11. Move intrinsic completion sources to use compiler-owned editor
  metadata instead of hand-maintained item lists.
- [ ] Slice 12. Filter completion items by active `fol_model`:
  - hide `std`-only intrinsics in `core` and `alloc`
  - hide heap-backed type families in `core`
  - keep `alloc` items visible in `std`
- [ ] Slice 13. Add completion tests for:
  - `core` does not suggest `.echo`
  - `alloc` does not suggest `.echo`
  - `std` does suggest `.echo`
  - `core` does not suggest `str`, `seq`, `vec`, `set`, `map` as normal type
    surfaces
- [ ] Slice 14. Add completion tests for mixed-model workspaces so package-local
  artifact context does not bleed across unrelated members.

### Exit criteria

- Completion is generated from compiler-backed facts where practical.
- Completion respects `fol_model`.

## Epoch 5: Generate Query Fragments From Compiler Metadata

Goal:
Reduce manual drift in tree-sitter queries without pretending grammar structure
 can be generated.

### Slice Tracker

- [ ] Slice 15. Split query content into:
  - hand-written structural query bodies
  - generated/compiler-derived regex fragments for names and symbol families
- [ ] Slice 16. Add a small generation step or checked-in generator that writes
  canonical fragments for:
  - builtin type names
  - implemented dot intrinsics
  - source kinds
  - container type names
  - shell type names
- [ ] Slice 17. Change `highlights.scm` consumption so those compiler-derived
  fragments are included from generated files or composed deterministically at
  bundle-generation time.
- [ ] Slice 18. Add tests proving generated fragments are stable and do not
  require hand edits when compiler registries change.

### Exit criteria

- Repetitive query name lists are compiler-derived.
- Manual query maintenance is reduced to structural capture logic.

## Epoch 6: Tighten Tree-Sitter Sync Gates

Goal:
Turn more classes of editor drift into obvious test failures.

### Slice Tracker

- [ ] Slice 19. Expand sync tests beyond `highlights.scm` to cover:
  - locals query expectations where compiler data can define them
  - symbols query expectations where declaration families are compiler-known
- [ ] Slice 20. Add tests that fail if new implemented dot intrinsics exist in
  the compiler but are absent from tree-sitter highlight coverage.
- [ ] Slice 21. Add tests that fail if new declaration heads/keywords exist in
  compiler constants but are absent from grammar/query coverage.
- [ ] Slice 22. Add tests for `defer`, named args, variadics, and `fol_model`
  example packages so real syntax fixtures exercise the tree-sitter bundle.

### Exit criteria

- Common "forgot to update tree-sitter" failures are caught automatically.
- Real checked-in examples participate in syntax coverage.

## Epoch 7: Add Editor Model Fixtures

Goal:
Use real example packages to lock editor behavior by model.

### Slice Tracker

- [ ] Slice 23. Add `fol-editor` integration fixtures for:
  - `core` example package
  - `alloc` example package
  - `std` example package
- [ ] Slice 24. Add LSP diagnostics tests over those packages:
  - legal surfaces stay quiet
  - forbidden surfaces produce exact model-aware errors
- [ ] Slice 25. Add semantic token tests over those same packages so model
  examples also harden highlighting and tokenization paths.
- [ ] Slice 26. Add completion tests opened against those package roots, not
  just synthetic one-file snippets.

### Exit criteria

- Real model examples drive editor integration coverage.
- LSP and tree-sitter are tested against the same example family users read.

## Epoch 8: Build-System Awareness In The Editor

Goal:
Make the editor understand routed build artifacts instead of only package-wide
 assumptions.

### Slice Tracker

- [ ] Slice 27. Teach editor workspace mapping to inspect `build.fol` routed
  artifacts and recover the artifact model for the current file where possible.
- [ ] Slice 28. Add tests for mixed-model workspaces where:
  - one member contains `core`
  - one member contains `alloc`
  - one member contains `std`
  and the editor chooses the right model per opened file/package.
- [ ] Slice 29. Define and document fallback behavior when the editor cannot map
  a file to a specific routed artifact:
  - whether to use package default
  - whether to use conservative model intersection
  - whether to show "unknown model context" notes
- [ ] Slice 30. Add tests for that fallback policy so ambiguous editor context
  behaves deterministically.

### Exit criteria

- Editor model awareness is based on build reality, not guesswork.
- Mixed-model workspaces behave deterministically in LSP.

## Epoch 9: Reduce Remaining Hand-Written LSP Registries

Goal:
Audit and remove manual editor registries that can be replaced with
 compiler-owned data.

### Slice Tracker

- [ ] Slice 31. Audit all completion/hover/navigation helper tables in
  `fol-editor` and classify each one:
  - must stay manual
  - can become compiler-backed
  - can become generated
- [ ] Slice 32. Replace at least the low-risk manual registries first:
  - intrinsic labels
  - builtin type suggestions
  - keyword family lists
- [ ] Slice 33. Add regression tests proving compiler-backed LSP registries stay
  synchronized without hand editing.

### Exit criteria

- Obvious duplicated registries are gone.
- Remaining manual registries are intentional and documented.

## Epoch 10: Final Hardening And Removal Of Weak Paths

Goal:
Finish with an editor pipeline that is hard to drift accidentally.

### Slice Tracker

- [ ] Slice 34. Add a top-level "editor sync" test suite that exercises:
  - compiler metadata export
  - generated query fragments
  - tree-sitter bundle validation
  - LSP diagnostics by model
  - LSP completion by model
- [ ] Slice 35. Remove stale manual comments/docs that imply editor updates are
  mostly hand-maintained.
- [ ] Slice 36. Add a contributor-facing doc section:
  - "if you add a language feature, these editor sync tests must pass"
  - "if the feature changes syntax shape, update grammar/query structure"
  - "if the feature only adds names or registries, generation should cover it"
- [ ] Slice 37. Run a full repo audit and close any remaining drift holes found
  during implementation.

### Exit criteria

- Editor sync expectations are explicit for contributors.
- The remaining manual surfaces are the ones that should stay manual.

## Expected End State

When this plan is complete:

- adding a new implemented intrinsic should usually require:
  - compiler change
  - maybe structural query update if syntax shape changed
  - no hand-edited duplicate intrinsic lists
- adding a new builtin/type family should usually require:
  - compiler change
  - generated metadata/query fragments update automatically or through one
    canonical generator
- opening a package in the editor should surface `core` / `alloc` / `std`
  boundaries through the real compiler pipeline
- mixed-model workspaces should behave predictably in LSP
- tree-sitter grammar will still be hand-authored, but compiler-driven drift in
  query-name families will be minimized and test-guarded

## Non-Goals

- Generate the entire tree-sitter grammar from the parser.
- Replace all `fol-editor` logic with compiler internals.
- Hide model ambiguity by guessing silently.
- Keep old manual registries if a compiler-backed source is available.
