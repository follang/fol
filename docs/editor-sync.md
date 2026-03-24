# Editor Sync

This document is the canonical contract for keeping the compiler, LSP, and
tree-sitter assets aligned.

## Intent

The editor layer should not become a second language implementation, and it
should not depend on copied compiler name lists.

The intended split is:

- compiler crates own semantic truth
- `fol-editor` reuses compiler analysis whenever possible
- tree-sitter grammar remains hand-authored
- repetitive editor registries should be compiler-derived
- drift should fail tests

## Ownership

### Compiler-owned editor data

These facts should come from compiler crates or compiler-owned registries:

- declaration keyword names
- builtin type names
- container type names
- shell type names
- source kind names
- implemented intrinsic names
- capability facts tied to `fol_model`

### Generated editor data

These should be generated or assembled from compiler-owned data where possible:

- highlight regex fragments for builtin names
- highlight regex fragments for implemented intrinsic names
- completion source lists for builtin names
- completion source lists for implemented intrinsics

### Manual editor data

These are the intentional manual surfaces that remain after sync automation:

- tree-sitter grammar structure
- highlight capture structure
- locals query structure
- symbols query structure
- LSP UX details such as ranking and presentation

## Registry Audit

The current editor registry surface is split like this.

### Must stay manual

These encode editor UX or structural syntax intent, not language-name facts:

| Area | Location | Why it stays manual |
|------|----------|---------------------|
| tree-sitter grammar structure | `lang/tooling/fol-editor/tree-sitter/grammar.js` | grammar shape is structural and cannot be usefully derived from compiler registries |
| structural highlight captures | `lang/tooling/fol-editor/queries/fol/highlights.base.scm` | capture layout is editor-facing presentation logic |
| locals/symbols query structure | `lang/tooling/fol-editor/queries/fol/locals.scm`, `lang/tooling/fol-editor/queries/fol/symbols.scm` | scope/symbol capture shapes are structural tree-sitter authoring |
| completion ranking and tie-breaking | `lang/tooling/fol-editor/src/lsp/completion_helpers.rs` | ordering and UX priority are editor policy |
| semantic token kind mapping | `lang/tooling/fol-editor/src/lsp/semantic.rs` | token categories are an editor-facing legend, not a compiler registry |

### Should be compiler-backed

These are language-name families and should not stay duplicated:

| Area | Location | Canonical source |
|------|----------|------------------|
| builtin type suggestions | `lang/tooling/fol-editor/src/lsp/semantic.rs` | `fol_typecheck::editor_builtin_type_names()` |
| container/shell type suggestions | `lang/tooling/fol-editor/src/lsp/semantic.rs` | `fol_typecheck::editor_container_type_names()`, `fol_typecheck::editor_shell_type_names()` |
| dot intrinsic completion names | `lang/tooling/fol-editor/src/lsp/semantic.rs` | `fol_typecheck::editor_implemented_intrinsics()` |
| `fol_model` availability filtering | `lang/tooling/fol-editor/src/lsp/semantic.rs` | `fol_typecheck::editor_intrinsic_available_in_model()` and `editor_type_family_available_in_model()` |
| command summaries for source kinds and intrinsic families | `lang/tooling/fol-editor/src/commands.rs` | compiler-owned editor metadata |

### Can become generated or centrally assembled

These should be rendered from one canonical helper instead of repeated string
lists:

| Area | Location | End state |
|------|----------|-----------|
| checked-in `highlights.scm` name families | `lang/tooling/fol-editor/src/tree_sitter.rs`, `queries/fol/highlights.scm` | generated from compiler metadata |
| command summary detail strings | `lang/tooling/fol-editor/src/commands.rs` | assembled from shared metadata helpers |
| editor sync regression snapshots | `lang/tooling/fol-editor/src/tree_sitter.rs`, `test/run_tests.rs` | compare against canonical rendered metadata instead of copied strings |

### Intentional leftovers after this plan

If a registry is still manual after the plan completes, it should be manual for
one of these reasons:

- it defines tree-sitter structure, not a compiler name family
- it defines editor ranking or rendering policy
- it describes a token/UX vocabulary that is intentionally editor-owned

## `fol_model` contract

The editor must treat `fol_model` as a real semantic boundary.

That means:

- diagnostics shown by LSP should match compiler/build diagnostics
- completion should hide surfaces that are invalid for the active model
- mixed-model workspaces should not silently bleed one model into another

## Model matrix

| Model | Type completion | Intrinsic completion | Diagnostics focus | Example packages |
|-------|-----------------|----------------------|-------------------|------------------|
| `core` | scalar, array, record, entry, shell surfaces only | no `std`-only intrinsics, no heap-only guidance | reject `str`, dynamic containers, dynamic `.len(...)`, `.echo(...)` | `examples/core_blink_shape`, `examples/core_defer`, `examples/core_records` |
| `alloc` | `core` types plus `str`, `vec`, `seq`, `set`, `map` | no `std`-only intrinsics | reject `.echo(...)`; allow heap-backed strings and containers | `examples/alloc_defaults`, `examples/alloc_containers`, `examples/alloc_collections` |
| `std` | all currently implemented type surfaces | all currently implemented V1 intrinsics valid for host artifacts | ordinary semantic/type diagnostics plus hosted-runtime behavior | `examples/std_cli`, `examples/std_echo_min`, `examples/std_named_calls` |

For mixed-model workspaces, editor tests should also cover
`examples/mixed_models_workspace`.

## Routed artifact fallback

When the editor can map an opened file to one routed artifact root from
`build.fol`, it should use that artifact's `fol_model`.

When the file does not map to one specific routed artifact:

- if every routed artifact in the package uses the same `fol_model`, the editor
  should reuse that uniform package model
- if routed artifacts disagree, the editor should keep the model unknown rather
  than guessing

That keeps mixed-model packages deterministic and avoids silently bleeding one
artifact model into unrelated helper files.

## Test gates

The minimum test gates for editor sync are:

- compiler constants match tree-sitter query name families
- compiler intrinsics match editor highlight/completion name families
- model-boundary diagnostics match between LSP and build-mode compilation
- real example packages for `core`, `alloc`, and `std` stay editor-readable

## Contributor rule

If a language feature changes semantic behavior only:

- the editor should usually pick it up through compiler-backed analysis

If a language feature adds new names:

- update the compiler-owned registry
- generated editor surfaces should follow from that

If a language feature changes syntax shape:

- update tree-sitter grammar and structural queries
- keep the manual surface small and test-guarded
