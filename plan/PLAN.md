# FOL Editor Plan

Last updated: 2026-03-20

## Goal

Make `fol-editor` good enough to use as the default day-to-day editor path for
FOL:

- completion must work reliably
- hover/definition/symbols must stay correct while editing
- diagnostics must stay fast enough for normal typing
- the public `fol tool` editor surface must match what is actually implemented
- editor features must have locked behavior with focused tests

This plan replaces the deleted `plan/PLAN.md`.

It is based on:

- `plan/future-work/fol-editor.md`
- current `fol-editor` implementation and tests
- current `fol tool` docs
- an observed failing completion test in-tree

## Current State

Already present:

- Tree-sitter grammar, queries, bundle generation, and parser/query debug commands
- LSP transport with `initialize`, `didOpen`, `didChange`, `didClose`, `hover`,
  `definition`, `references`, `rename`, `documentSymbol`, and `completion`
- compiler-backed diagnostics through package/workspace overlay analysis
- completion tests covering plain, qualified, type-position, and dot-trigger
  cases

Observed gaps and risks:

- completion is not stable yet:
  `cargo test -p fol-editor completion -- --nocapture` currently fails in
  `lsp_server_prefers_nearer_symbols_when_completion_names_conflict`
- `textDocument/completion` currently ignores the incoming LSP completion
  context and only calls `plain_completion_items(document, position)`
- every analysis request materializes a full temp overlay copy and rebuilds
  semantic state from scratch
- only full-document sync is supported
- docs still describe completion as a growing feature while the code/test matrix
  is already broader
- future-work still lists features that are partly implemented or already
  exposed internally

## Product Direction

The editor should move in this order:

1. Make existing features correct.
2. Make existing features fast enough.
3. Expose missing but already-supported surfaces cleanly.
4. Add new editor features only after the core loop is trustworthy.

Do not keep placeholder or parallel editor paths once the real path exists.

## Workstreams

### Slice 1: Stabilize Completion Now

Completion is the highest priority because it is directly broken and blocks
basic editor usability.

Work:

- [x] fix symbol shadowing so nearer locals beat outer/top-level symbols
- [x] thread `LspCompletionParams.context` through the server and use it instead of
  silently discarding trigger metadata
- [x] make completion ordering deterministic across:
  local bindings, parameters, imports, top-level items, namespaces, intrinsics,
  and types
- [x] separate plain completion, qualified-path completion, type-position
  completion, and dot-trigger completion at the LSP request boundary instead of
  hiding all branching inside a single plain path
- [x] remove stale or misleading fallback behavior where compiler-backed data should
  be authoritative
- [x] trim the current completion test files so each one proves one thing instead
  of carrying large unused-import noise

Exit condition:

- `cargo test -p fol-editor completion -- --nocapture` passes
- local shadowing is covered by at least one focused regression test
- LSP completion behavior is driven by request context, not ignored

### Slice 2: Lock The Core LSP Contract

The current server already does more than the plan doc says. That contract
needs to be explicit and tested.

Work:

- define the supported v1 LSP feature set:
  diagnostics, hover, definition, references, rename, document symbols,
  completion
- [x] remove any unsupported advertised capability if the implementation is not
  real enough yet
- [x] add capability-level tests for `initialize`
- [x] add lifecycle tests proving open/change/close/update behavior across multiple
  files in one session
- [x] verify `build.fol` works through the same mapping, parse, highlight, symbol,
  and diagnostic paths as source files where intended

Exit condition:

- one test suite locks the exact advertised LSP capabilities
- docs and implementation agree on the shipped v1 feature set

### Slice 3: Stop Rebuilding The World On Every Edit

Correct but slow editor behavior will feel broken in practice.

Work:

- [x] profile the current analysis path around overlay creation, package loading,
  resolver runs, and typecheck runs
- [x] cache workspace/package discovery in-session instead of rediscovering roots
  on each request
- [x] stop copying entire analysis roots into a temp overlay for every request if a
  narrower overlay or in-memory path can be used
- [x] reuse semantic artifacts across requests when the document version has not
  changed
- separate fast-path requests from full diagnostic refresh where possible

Exit condition:

- repeated hover/definition/completion requests on an unchanged document do not
  rebuild the entire workspace every time
- [x] the implementation has a documented invalidation model

### Slice 4: Incremental Document Model

The server currently assumes full-document sync only. That is acceptable for a
bootstrap but not as the long-term editor model.

Work:

- [x] add incremental text sync support in the document store and LSP types
- [x] keep full sync only if the client requests it; otherwise use incremental
  changes
- [x] make completion/hover/definition operate against the current in-memory
  document version without hidden resync assumptions
- [x] add tests for multiple edits in one session, including incomplete text and
  parse/type errors

Exit condition:

- [x] `textDocument/didChange` supports incremental edits
- [x] document state stays correct through a multi-edit lifecycle test matrix

### Slice 5: Finish The Public Editor Surface

The internal implementation is ahead of the public CLI/docs surface.

Work:

- [x] define whether the public entry remains `fol tool ...` or moves to
  `fol editor ...`; choose one and delete the other path instead of keeping both
- [x] if keeping `fol tool`, expose only real commands
- add public commands once the underlying features are real:
  [x] semantic tokens
  [x] references
  [x] rename
  format
- [x] document editor startup requirements, workspace discovery rules, and failure
  modes clearly
- [x] add an integration test for the chosen public entry surface

Exit condition:

- [x] one public editor command surface exists
- [x] the docs no longer describe internal/editor behavior that users cannot reach

### Slice 6: Semantic Tokens

Semantic tokens are the most natural next feature after completion and
navigation because the semantic snapshot already exists.

Work:

- [x] define a small stable token taxonomy for FOL instead of mirroring every
  internal symbol kind
- [x] implement `textDocument/semanticTokens/full`
- [x] reuse resolver/typecheck facts where useful; do not invent a second semantic
  classifier
- [x] add snapshot tests over representative FOL files and `build.fol`
- [x] expose the matching public command only after the LSP feature is real

Exit condition:

- [x] semantic token output is stable enough for editor integration tests

### Slice 7: References And Rename

These should come before code actions because they build directly on symbol
identity.

Work:

- [x] implement `textDocument/references`
- [x] implement `textDocument/rename`
- [x] make rename refuse unsafe multi-file cases until cross-file correctness is
  solid
- [x] cover locals, parameters, top-level items, imported namespaces, and
  same-package namespaced references
- [x] define the first safe rename boundary explicitly

Exit condition:

- references and rename are correct for the supported symbol classes
- unsupported rename cases fail explicitly instead of doing partial work

### Slice 8: Formatting

Formatting should be a real subsystem, not a placeholder command.

Work:

- decide whether formatting is AST-driven, CST-driven, or hybrid
- implement whole-document formatting first
- add range formatting only if the formatter can preserve stable structure
- wire formatting through LSP and CLI only after the formatter itself is stable
- delete any placeholder formatting exposure until the real formatter exists

Exit condition:

- formatting is deterministic
- formatter output has fixture-based regression coverage

### Slice 9: Code Actions And Signature Help

These are useful, but they are downstream of stable diagnostics, completion,
and formatting.

Work:

- signature help for routine calls
- diagnostic-driven quick fixes only where the fix is structurally obvious
- no speculative or weak code actions
- add focused tests for each supported action

Exit condition:

- each code action has a precise trigger and deterministic rewrite

## Testing Strategy

Add and keep these test layers:

- focused unit tests for completion context, scope lookup, and ranking
- LSP request/response tests for each supported method
- lifecycle tests covering open, edit, diagnose, complete, hover, close
- integration tests through the public `fol` entrypoint
- [x] performance smoke tests for repeated requests on unchanged documents

Avoid giant mixed-purpose test modules. Split tests by behavior and remove
unused imports and setup noise as the suite is touched.

## Documentation Work

Update these docs as slices land:

- [x] `book/src/000_overview/300_editor.md`
- [x] `book/src/050_tooling/300_editor.md`
- [x] `plan/future-work/fol-editor.md`

Specifically:

- [x] move shipped features out of future-work
- [x] keep future-work limited to features that are actually still future work
- [x] document the chosen public command surface and editor capability set

## Immediate Next Slice

Start with Slice 1.

Completion is already failing in-tree and is the clearest usability issue.
Until completion is correct and stable, adding rename, references, formatting,
or semantic tokens is the wrong priority.
