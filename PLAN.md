# fol-editor V1 Hardening Plan

This plan is only for the current V1 language/editor contract.

It does not introduce V2 language features.
It does not create a second semantic implementation in the editor.
It keeps compiler-backed analysis as the source of truth.

The goal is to make `fol-editor` materially better for the current shipped
language:

- better quick fixes
- better completion
- better hover / definition / references across real package boundaries
- better diagnostics UX
- stronger document / workspace symbols
- safer rename in the currently-supported scope
- stronger real-example and mixed-workspace coverage

Non-goals for this plan:

- no V2 syntax or semantic work
- no editor-only semantic rules that compete with the compiler
- no compatibility paths for removed syntax or removed build behavior
- no speculative range formatting rollout unless formatter structure safety is
  explicitly solved

## Epoch 1: Freeze V1 Editor Contract

### Slice 1 [complete]
Audit and pin the currently shipped `fol-editor` capability set in docs and
tests:

- hover
- definition
- document symbols
- workspace symbols
- formatting
- code actions
- signature help
- references
- rename
- semantic tokens
- completion

Completion criteria:

- `book/src/050_tooling/500_lsp.md` states the exact current surface
- active lifecycle tests pin the advertised server capabilities

### Slice 2 [complete]
Document the V1 editor non-goals explicitly:

- no V2-aware language support
- no range formatting
- no editor-owned semantic divergence
- no broad rename beyond current safe classes

Completion criteria:

- `docs/editor-sync.md` and `AGENTS.md` reflect the same boundary

### Slice 3 [complete]
Add a top-level regression that fails if active docs/examples describe
unsupported editor features as already shipped.

Completion criteria:

- suite fails on stale claims such as broad rename, broad code actions, or
  range formatting

## Epoch 2: Code Actions Worth Using

### Slice 4 [complete]
Audit the current code-action inventory in `fol-editor` and pin its exact
starting behavior with direct tests.

Completion criteria:

- unresolved-name replacement behavior is locked with exact tests
- parse-only and unsupported typecheck diagnostics remain action-free where
  intended

### Slice 5
Add code actions for missing bundled std dependency diagnostics in the current
V1 contract.

Target behavior:

- if code uses `use std: pkg = {"std"};` but build metadata lacks bundled std,
  the editor offers a quick fix message appropriate to the current build model

Completion criteria:

- editor test proves a real quick fix is returned for the missing-std case

### Slice 6
Add code actions for wrong bundled std alias diagnostics.

Target behavior:

- when a package declared bundled std under a different alias but source uses
  `std`, the editor can suggest the declared alias or the import correction

Completion criteria:

- real package test covers alias mismatch quick fix

### Slice 7
Add code actions for removed import syntax guidance where the compiler already
produces exact replacement structure.

Target behavior:

- unquoted import targets like `use x: pkg = {x};` offer the quoted fix when an
  exact replacement is available

Completion criteria:

- parser/LSP path test proves quick fix appears only when the exact replacement
  is safe

### Slice 8
Add code actions for invalid `fol_model` spellings when the compiler emits exact
guidance.

Target behavior:

- stale `std` mode or stale `mem` mode diagnostics can surface a quick fix to
  `memo` where the replacement is exact

Completion criteria:

- real `build.fol` editor test covers returned quick fix

### Slice 9 [complete]
Tighten code-action ranking and deduplication so editor UX remains stable when
multiple compiler suggestions exist on one line.

Completion criteria:

- code actions sort deterministically
- duplicate or overlapping edits do not multiply in the UI

## Epoch 3: Completion Quality

### Slice 10 [complete]
Audit completion contexts and pin current plain / qualified / dot-trigger /
type-position behavior with direct tests.

Completion criteria:

- completion tests explicitly cover each context class

### Slice 11
Improve bundled std package completion in the canonical V1 form:

- `use std: pkg = {"std"};`
- `std::...`

Completion criteria:

- completion tests prove bundled std names appear only when the dependency is
  actually declared

### Slice 12
Improve `pkg` import alias completion for declared dependencies.

Target behavior:

- declared aliases appear reliably in import positions
- undeclared aliases do not leak into completion lists

Completion criteria:

- package-backed examples cover positive and negative alias completion

### Slice 13
Improve namespaced member completion for cross-package and bundled std symbols.

Completion criteria:

- completion after `std::`
- completion after dependency alias namespace
- no unrelated package bleed in the offered items

### Slice 14
Improve build-file completion around the current build system:

- `.build()`
- build methods
- graph methods
- dependency-handle methods
- output/path-handle methods

Completion criteria:

- build-file completion tests cover current public build API groups

### Slice 15
Improve mixed-workspace completion filtering so package-local ambiguity does not
bleed wrong-model or wrong-package names into the list.

Completion criteria:

- mixed-model workspace completion stays conservative

## Epoch 4: Hover, Definition, References

### Slice 16
Audit current hover behavior and pin bundled std hover output in real examples.

Completion criteria:

- bundled std routine hover is stable and tested

### Slice 17
Strengthen definition jumps across:

- current package
- declared dependency aliases
- bundled std

Completion criteria:

- definition tests prove cross-package jumps work for the shipped V1 forms

### Slice 18
Strengthen reference results for supported symbol classes across package roots.

Completion criteria:

- same-file local references remain correct
- current-package top-level references remain correct
- supported package-backed references do not duplicate or miss obvious uses

### Slice 19
Add explicit negative navigation coverage for missing bundled std dependency and
alias mismatch situations.

Completion criteria:

- hover / definition / references fail cleanly with current diagnostics instead
  of jumping to nonsense

### Slice 20
Tighten hover detail rendering so current V1 types and symbol kinds display more
consistently in ordinary source and `build.fol`.

Completion criteria:

- stable rendering tests for representative routine/type/build handles

## Epoch 5: Diagnostics UX

### Slice 21
Audit current diagnostic adaptation and pin the intended editor-facing message
shape:

- diagnostic code included
- dedupe behavior
- no extra editor-only semantic wording

Completion criteria:

- diagnostics tests lock the current shape

### Slice 22
Improve related-location / note projection where compiler diagnostics already
carry meaningful extra locations.

Completion criteria:

- editor diagnostics expose more compiler structure without inventing new logic

### Slice 23
Reduce noisy duplicate diagnostics during mid-edit invalid states.

Completion criteria:

- editor retains useful diagnostics while avoiding repeated cascade spam in the
  open file

### Slice 24
Add explicit V1 coverage for current important diagnostic classes:

- missing std dependency
- std alias mismatch
- invalid import target syntax
- invalid `fol_model`
- model-boundary violations in `core` / `memo`

Completion criteria:

- editor tests map these diagnostics to stable LSP output

### Slice 25
Audit build-file diagnostics and ensure build-specific failures remain readable
and current-contract-oriented in the editor.

Completion criteria:

- build-file editor tests cover current public build errors

## Epoch 6: Symbols and Outline

### Slice 26
Audit document symbols for nested namespaces, imports, and current shipped
standard-library examples.

Completion criteria:

- representative document-symbol snapshots are pinned

### Slice 27
Improve workspace symbol relevance and filtering for current open workspace
members.

Completion criteria:

- workspace symbol tests behave better for mixed package sets

### Slice 28
Ensure bundled std examples and dependency-backed packages contribute correct
document/workspace symbols.

Completion criteria:

- symbol tests include bundled std examples and declared dependency examples

### Slice 29
Add explicit negative coverage to ensure unsupported or unresolved entities do
not masquerade as symbols.

Completion criteria:

- unresolved imports and malformed syntax do not produce misleading symbol
  entries

## Epoch 7: Rename Hardening

### Slice 30
Audit current rename support and pin the current safe boundary:

- same-file locals
- current-package top-level symbols

Completion criteria:

- tests and docs state the supported boundary explicitly

### Slice 31
Harden same-file local rename behavior through more syntax shapes:

- routine locals
- parameters
- pattern/destructure bindings where currently supported

Completion criteria:

- rename tests cover the currently intended local symbol classes

### Slice 32
Harden current-package top-level rename so all touched files update correctly in
the supported cases.

Completion criteria:

- multi-file package rename tests are reliable

### Slice 33
Add explicit negative rename coverage for unsupported cases:

- cross-package rename
- ambiguous mixed-workspace symbols
- unresolved symbols

Completion criteria:

- editor rejects these cleanly rather than producing partial edits

## Epoch 8: Workspace Recovery and Real Examples

### Slice 34
Audit workspace mapping and active-model recovery against the current V1
contract:

- `core`
- `memo`
- bundled std declared or not

Completion criteria:

- mapping tests cover single-artifact, uniform-package, and mixed-artifact roots

### Slice 35
Strengthen ambiguous-file behavior in mixed-model workspaces so the editor keeps
model unknown when it should remain conservative.

Completion criteria:

- ambiguous helper-file tests remain conservative

### Slice 36
Add real-example editor coverage for the current canonical examples:

- bundled std example packages
- no-std runnable examples
- raw substrate example
- quoted import syntax examples

Completion criteria:

- example-model tests and integration editor tests cover these roots directly

### Slice 37
Tighten overlay/materialization behavior so editor tests do not rely on shared
temp state or incidental build artifacts.

Completion criteria:

- editor tests are stable under repeated runs and parallel CI-like pressure

## Epoch 9: Tree-sitter and Presentation Sync

### Slice 38
Audit tree-sitter highlight/query coverage for current shipped V1 examples and
current import/build syntax.

Completion criteria:

- real-example highlight tests cover quoted imports and current build syntax

### Slice 39
Add explicit tree-sitter/editor sync coverage for bundled std import forms:

- `use std: pkg = {"std"};`

Completion criteria:

- stale removed import forms fail the editor tree-sitter suite

### Slice 40
Audit semantic-token output for current build-file and package-backed std
examples.

Completion criteria:

- semantic-token tests cover bundled std and build-file identifiers with current
  public naming

### Slice 41
Keep compiler-backed registry sync honest:

- builtin names
- intrinsic names
- source kinds
- model capability filtering

Completion criteria:

- top-level editor-sync regressions fail on drift

## Epoch 10: Docs, Contributor Rules, Closure

### Slice 42 [complete]
Update the book’s LSP chapter so it matches the actual post-hardening V1 editor
surface and limitations.

Completion criteria:

- `book/src/050_tooling/500_lsp.md` reflects real shipped behavior

### Slice 43 [complete]
Update `docs/editor-sync.md` with the final V1 editor-hardening expectations and
the exact responsibilities that remain compiler-backed versus manual.

Completion criteria:

- contributor guidance is current and testable

### Slice 44 [complete]
Update `AGENTS.md` so future feature work is forced to consider:

- code actions
- completion
- navigation
- tree-sitter/editor mirror

Completion criteria:

- contributor rules mention editor checks for current V1 surfaces

### Slice 45 [complete]
Add a top-level stale-claim scan over docs/examples/tests so removed or
unsupported editor claims do not creep back in.

Completion criteria:

- tests fail on stale claims such as unsupported broad rename, unsupported range
  formatting, or ambient std import behavior

## Done Definition

This plan is complete when:

- `fol-editor` remains compiler-backed for semantics
- V1 quick fixes are materially more useful
- completion/navigation are stronger for real package and bundled std workflows
- diagnostics, symbols, and rename are more reliable in the current supported
  scope
- tree-sitter/editor sync stays honest
- docs/tests/examples describe only the real shipped V1 editor surface
