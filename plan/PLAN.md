# FOL Editor Plan: Highlighting + Completion

Last updated: 2026-03-17

This plan replaces the closed first `fol-editor` milestone.

The next active milestone is narrower and more practical:

- make FOL highlighting feel complete and language-aware
- implement the first useful LSP completion pass

This is not a “general editor improvements” bucket. It is a focused follow-up on
the now-working Tree-sitter + LSP base.

## Goal

At closeout, the editor experience should feel like a real language, not a thin
syntax demo.

That means:

- the Tree-sitter highlight layer should visibly distinguish the important FOL
  surfaces
- `fol tool lsp` should provide real `textDocument/completion`
- completion should be useful in ordinary coding, not just technically present

## Desired End State

After this plan, the repo should support:

```text
fol tool lsp
fol tool parse path/to/file.fol
fol tool highlight path/to/file.fol
fol tool symbols path/to/file.fol
fol tool tree generate /tmp/fol
```

And editors should have:

- richer Tree-sitter highlighting for current `V1`
- stable query coverage for declaration modifiers and language keywords
- first-pass LSP completion for:
  - local bindings
  - routine params
  - top-level declarations in the current package
  - imported declarations
  - type names
  - method/routine names where already semantically visible
  - dot intrinsics for supported `V1` surfaces

## Non-Goals

This plan does not include:

- rename
- references
- semantic tokens
- code actions
- snippet expansion policy
- formatter support
- fuzzy ranking or AI-style completion
- full context-aware completion for every future `V2`/`V3` language surface

Those belong in later editor follow-up work.

## Boundary

This milestone should touch:

- `lang/tooling/fol-editor`
- `lang/tooling/fol-frontend` only where command dispatch/output changes are
  needed
- Neovim-facing Tree-sitter bundle generation only if required for query
  packaging
- book/docs only after the feature work is stable

It should not change:

- compiler semantics unless a completion/highlight pass exposes a real bug
- package model
- backend behavior

## Principles

### 1. Highlighting should reflect FOL, not generic C-ish defaults

FOL has visible language markers like:

- `fun[exp]`
- `fun[par]`
- `fun[rec]`
- `log[...]`
- `typ[...]`
- `ali[...]`
- shells like `nil`
- effect-ish surfaces like `report`, `check`, `panic`
- package/import kinds like `loc`, `pkg`, `std`

Those should be represented intentionally in the query layer.

### 2. Completion should come from semantic truth

Completion should not be built from regexes or Tree-sitter-only guesses where
compiler truth is available.

Preferred sources:

- typed/resolved workspace state
- current document overlay state
- explicit intrinsic tables

### 3. Ship useful completion before clever completion

First pass should prefer:

- predictable
- stable
- easy to test

over:

- fuzzy ranking
- edit-distance matching
- complicated insert text transformations

## Current Baseline Gaps

Before this milestone, the real editor gaps are:

- Tree-sitter highlighting still treats many FOL surfaces too generically
- declaration modifiers like `[exp]` and similar bracketed markers are not yet
  represented clearly enough
- import source kinds and effect-ish keywords are not distinct enough in the
  current query layer
- dotted intrinsic surfaces are present, but not rich enough to feel language
  specific
- `fol tool highlight` is still more of a smoke command than an inspection tool
- the LSP is attached and serving diagnostics/navigation, but completion is not
  implemented yet
- completion provider capabilities are not advertised yet
- there is no first-pass completion contract yet for locals, params, imports,
  types, or dot intrinsics

## Highlighting Scope

The highlight pass should cover the current `V1` syntax more deeply than the
first milestone.

Target improvements:

- declaration heads and head modifiers:
  - `fun`
  - `log`
  - `typ`
  - `ali`
  - `var`
  - `use`
- bracketed declaration modes:
  - `[exp]`
  - `[par]`
  - other currently surfaced markers that exist in the real parser/grammar
- import source kinds:
  - `loc`
  - `pkg`
  - `std`
- control/effect keywords:
  - `when`
  - `loop`
  - `return`
  - `break`
  - `report`
  - `check`
  - `panic`
  - `unreachable`
- declaration names by role:
  - routines
  - logs
  - aliases
  - records
  - entries
  - variants
  - bindings
- typed-binding punctuation and shells:
  - type annotations
  - `/`
  - `!`
  - `nil`
- intrinsic-like dotted names:
  - `.len`
  - `.echo`
  - comparison/bool/query intrinsics already supported in `V1`

The goal is not “more colors” by itself. The goal is that important syntactic
roles are visually distinguishable.

## Completion Scope

The first completion pass should support these contexts:

### Plain identifier completion

When completing in an identifier position, offer:

- local bindings
- routine params
- top-level values/routines/types in the current package
- imported symbols that are visible in the current scope

### Qualified completion

When completing after a namespace/package qualifier, offer:

- visible declarations exported by that namespace/package

### Type-position completion

When completing in a declared type position, offer:

- builtin types
- visible named record/entry/alias types
- imported type declarations

### Dot completion

When completing after `.`, offer:

- supported intrinsics for the receiver family, when type information is known
- otherwise, a conservative fallback of current `V1` intrinsic names if the
  receiver family cannot yet be derived safely

### Trigger behavior

First pass triggers should be minimal and explicit:

- ordinary completion request
- `.` trigger for intrinsic/member-like completion
- `:` or `/` should not gain completion unless that falls out naturally and
  stays correct

## Completion Output Contract

The LSP should advertise a real completion provider.

First completion items should include:

- `label`
- `kind`
- `detail`
- simple `insertText` or plain replacement text

Do not overcomplicate the first pass with:

- snippets everywhere
- documentation resolution requests
- command-based follow-up actions

Useful `kind` mapping matters:

- variable/local
- function/routine
- method
- module/namespace
- type/class/struct-ish mapping for record/entry/alias surfaces
- keyword

## Testing Rules

Every feature/fix commit in this milestone should include its relevant test.

Required test layers:

- query snapshot tests
- `fol tool highlight` output tests
- `fol tool symbols` stability where query changes affect symbol capture
- LSP request/response tests for completion
- root integration tests where frontend command behavior changes

Completion tests should cover:

- local bindings
- params
- imported names
- type-position candidates
- dot completion
- no bogus suggestions from unrelated files

## Fixture Policy

Use real checked-in `.fol` fixtures where possible.

Prefer:

- `test/apps/fixtures/...`
- `test/apps/showcases/...`
- `xtra/logtiny`

over tiny synthetic one-liners, unless a one-liner is the clearest regression.

## Phases

### Phase 0: Freeze boundary and baseline

- `0.1` complete: replace the old editor-closeout plan with this focused
  highlight + completion plan
- `0.2` complete: document the exact current highlight and completion gaps
  before changes
- `0.3` complete: add a small acceptance checklist for Neovim-facing
  verification

### Phase 1: Highlight query audit

- `1.1` complete: audit current `highlights.scm` against current `V1` grammar
  node shapes
- `1.2` complete: remove any remaining impossible or overly generic patterns
- `1.3` complete: lock query validity against the generated parser bundle path
- `1.4` complete: add tests that fail when highlight queries reference
  non-existent nodes

### Phase 2: Declaration and modifier highlighting

- `2.1` complete: highlight declaration heads distinctly: `fun`, `log`, `typ`,
  `ali`, `var`, `use`
- `2.2` complete: highlight declaration modifiers in bracket forms like
  `[exp]`, `[par]`, and other real surfaced markers
- `2.3` complete: highlight declaration names by role:
  function/log/type/alias/binding
- `2.4` complete: add highlight snapshots for declaration-heavy real fixtures

### Phase 3: Keyword and import-kind highlighting

- `3.1` complete: highlight control/effect keywords consistently
- `3.2` complete: highlight import source-kind markers: `loc`, `pkg`, `std`
- `3.3` complete: highlight shell-related keywords/literals including `nil`
- `3.4` complete: lock real-fixture snapshots for keyword/import-heavy files

### Phase 4: Type and intrinsic highlighting

- `4.1` complete: highlight builtin types and named type references more clearly
- `4.2` complete: highlight typed-binding/type-annotation surfaces distinctly
- `4.3` complete: highlight dotted intrinsic names like `.len`, `.echo`, comparisons, and
  boolean/query intrinsics
- `4.4` complete: add snapshots for container/shell/intrinsic-heavy fixtures

### Phase 5: Highlight command and bundle hardening

- `5.1` complete: make `fol tool highlight` output more inspection-friendly for query work
- `5.2` complete: ensure `fol tool tree generate` always exports the latest query set
- `5.3` complete: add regression coverage so generated bundles contain the current query
  files exactly
- `5.4` complete: verify the generated bundle remains Neovim-consumable

### Phase 6: Completion protocol foundation

- `6.1` complete: add `textDocument/completion` request handling to the LSP server
- `6.2` complete: advertise a real completion provider from `initialize`
- `6.3` complete: define the internal completion item model in `fol-editor`
- `6.4` complete: add focused stdio/request tests for completion request/response framing

### Phase 7: Scope and symbol completion

- `7.1` complete: return local binding completions
- `7.2` complete: return routine parameter completions
- `7.3` complete: return current-package top-level declaration completions
- `7.4` complete: return imported visible declaration completions
- `7.5` complete: filter duplicate/irrelevant candidates deterministically
- `7.6` complete: lock tests for local/imported symbol completion

### Phase 8: Type-position completion

- `8.1` complete: detect ordinary declared-type completion contexts
- `8.2` complete: offer builtin type completions in type positions
- `8.3` complete: offer visible named type completions in type positions
- `8.4` complete: add tests for record/entry/alias/builtin type completion

### Phase 9: Qualified and namespace completion

- `9.1` complete: detect qualified path completion contexts
- `9.2` complete: offer namespace/package members after qualification
- `9.3` complete: keep package-local and imported namespace completion separated clearly
- `9.4` complete: add tests for `loc` and same-package namespace completion

### Phase 10: Dot completion

- `10.1` complete: detect `.` completion trigger contexts
- `10.2` complete: map typed receiver families to supported `V1` intrinsics
- `10.3` complete: return conservative fallback intrinsic suggestions when typing
  context is incomplete but still safe
- `10.4` complete: add tests for `.len`, `.echo`, comparison, and boolean/query
  completion

### Phase 11: Ranking, filtering, and response shaping

- `11.1` complete: choose stable completion item kinds/details for FOL symbols
- `11.2` complete: return deterministic ordering for repeated requests
- `11.3` complete: avoid noisy suggestions from unrelated packages/files
- `11.4` complete: add plain tests locking item labels/kinds/order

### Phase 12: Frontend and tool command coverage

- `12.1` keep `fol tool lsp` compatible with the new completion capability
- `12.2` extend frontend/editor tests if command summaries or help output shift
- `12.3` ensure `fol tool parse/highlight/symbols` remain stable while query
  work lands

### Phase 13: Real-editor hardening

- `13.1` test the full Neovim path against the generated Tree-sitter bundle
- `13.2` test real LSP diagnostics + completion on checked-in package fixtures
- `13.3` fix any remaining overlay/root/filtering bugs exposed by editor use
- `13.4` keep each discovered bug as a permanent regression test

### Phase 14: Docs closeout

- `14.1` update `book` docs for richer highlighting and first completion support
- `14.2` update repo status docs if the public editor surface changed
- `14.3` turn this file into a completion record once the milestone is finished

## Acceptance Checklist

This plan is only done when all of these are true:

- `fol tool highlight <PATH>` visibly reports the richer highlight captures
- generated Tree-sitter bundles contain the latest `.scm` queries
- Neovim Tree-sitter highlighting covers declaration modifiers and intrinsic
  surfaces better than the previous milestone
- `fol tool lsp` advertises completion support
- completion returns useful candidates for locals, imports, types, and dot
  intrinsics
- diagnostics still stay file-correct while completion is enabled
- `make build` passes
- `make test` passes

### Neovim Verification

The acceptance pass should also verify this real editor flow:

1. `fol tool tree generate /tmp/fol`
2. Neovim Tree-sitter loads the generated parser/query bundle
3. a real `.fol` file highlights declaration modifiers and intrinsic surfaces
4. `fol tool lsp` attaches to the buffer
5. source-file diagnostics appear on real source errors
6. hover and go-to-definition still work after the completion changes land
7. completion appears in the same buffer for locals/imports/types/dot contexts

## Progress

Current milestone state:

- `43 / 49` slices complete
- `88%`
