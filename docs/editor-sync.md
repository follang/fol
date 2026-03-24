# Editor Sync

This document is the canonical contract for keeping the compiler, LSP, and
tree-sitter assets aligned.

## Intent

The editor layer should not become a second language implementation.

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

These still need hand maintenance:

- tree-sitter grammar structure
- highlight capture structure
- locals query structure
- symbols query structure
- LSP UX details such as ranking and presentation

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
