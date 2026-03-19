# FOL Tooling Integration Plan

Last updated: 2026-03-19

## Goal

Tighten the connection between:

- `fol-diagnostics` and `fol-editor`
- `fol-{lexer,parser,resolver,typecheck,intrinsics}` and `fol-editor`
- compiler-owned language facts and Tree-sitter/LSP/editor-facing assets

The target outcome is:

- compiler diagnostics stay the single canonical error model
- the editor stops re-inventing compiler contracts ad hoc
- LSP behavior is visibly derived from compiler state, not from fragile local heuristics
- Tree-sitter stays editor-oriented, but obvious duplicated language facts are generated instead of copied by hand
- adding a new language feature comes with one explicit update checklist

This plan replaces the previous strict-error-model plan. The language work is done. The next problem is tooling cohesion.

## Audit Summary

### Current diagnostics coupling

Current shape:

- compiler crates produce `fol_diagnostics::Diagnostic`
- `fol-editor` converts those to LSP in `lang/tooling/fol-editor/src/convert.rs`
- `fol-editor` also owns its own editor-side dedup policy in `lang/tooling/fol-editor/src/lsp/analysis.rs`

What is good:

- compiler diagnostics already carry stable codes, labels, notes, helps, and locations
- LSP already uses compiler diagnostics instead of inventing a separate semantic analyzer

What is weak:

- the LSP conversion contract lives in `fol-editor`, not in a compiler-owned integration boundary
- editor dedup policy is separate from `fol-diagnostics` report policy
- editor message formatting (`[CODE] message`, notes, helps, related info) is reassembled locally
- no documented rule says when a compiler diagnostic change requires editor updates

### Current semantic LSP coupling

Current shape:

- `fol-editor` overlays the open document
- then it runs parse/package/resolve/typecheck in-process
- hover/definition/completion/document symbols read from resolved or typed compiler workspaces

What is good:

- the LSP is already compiler-backed
- `fol-intrinsics` is already used by the editor completion path
- typed types are already rendered in hover/completion helpers

What is weak:

- several editor fallback paths still parse text heuristically when semantic data is absent
- symbol/type rendering is handwritten in `fol-editor`
- no shared “semantic projection” API exists for editor consumers
- no feature checklist forces new resolver/typecheck surfaces to update hover/completion/definition behavior

### Current Tree-sitter coupling

Current shape:

- grammar lives in `lang/tooling/fol-editor/tree-sitter/grammar.js`
- query files live in `lang/tooling/fol-editor/queries/fol/*.scm`
- corpus files are stored in the editor crate
- the grammar is handwritten and separate from the compiler parser

What is good:

- the repo clearly separates compiler parsing from editor parsing
- the bundle command gives one editor-consumable output root

What is weak:

- keyword/type/intrinsic/source-kind facts are duplicated manually
- there is no generated manifest for syntax-visible language facts
- there is no documented boundary for what should be generated versus handwritten
- there is no required feature checklist when adding syntax that affects grammar, queries, and corpus coverage

### Current tooling health issues discovered during audit

These are not all caused by the strict error work, but they block confidence:

- `cargo test -p fol-frontend` currently fails due to stale test imports and stale function signatures
- `cargo test -p fol-editor` currently fails due to stale fixture paths, brittle assertions, and tree-sitter test drift
- editor/tooling tests are not currently the reliable guardrail they need to be

That means the first phase must stabilize tooling tests before deeper coupling work.

## Architecture Decision

### Principle 1: Compiler owns truth

These crates own canonical language truth:

- `fol-lexer`
- `fol-parser`
- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-intrinsics`
- `fol-diagnostics`

`fol-editor` should consume that truth, not mirror it.

### Principle 2: Tree-sitter is not the compiler parser

Tree-sitter remains a separate editor grammar.

Do not try to force direct parser generation from the compiler parser. The parsing models are different and that path will create more fragility than value.

Instead:

- keep the grammar handwritten
- generate only the syntax facts that are obviously shared
- keep editor query policy handwritten

### Principle 3: Shared facts should be exported, not recopied

The following should become compiler-owned manifests or helper APIs:

- diagnostic code and presentation policy
- implemented intrinsic names and surfaces
- builtin type names
- shell/container type families
- import source kinds
- keyword families used by syntax tooling

### Principle 4: Adding a language feature must update tooling explicitly

Every new feature should answer:

1. Does it change compiler diagnostics?
2. Does it change semantic editor behavior?
3. Does it change syntax/highlighting/query behavior?
4. Can part of that be generated instead of handwritten?

This must be enforced in docs, tests, and review structure.

## Target End State

### Diagnostics target

- compiler diagnostics remain the only canonical structured error object
- LSP conversion is moved behind a compiler-owned integration boundary
- editor-side diagnostic formatting policy is documented and testable
- dedup rules are explicit and intentionally shared or intentionally layered

Preferred design:

- add a small compiler-adjacent adapter crate or module such as `fol-diagnostics-lsp`
- move `Diagnostic -> LSP diagnostic` conversion there
- keep pure LSP wire types in `fol-editor` if desired, but remove ad hoc message policy from editor code

Important constraint:

- do not make `fol-diagnostics` depend directly on `fol-editor`
- dependency direction should stay compiler/core -> adapter -> tooling, not the reverse

### LSP semantic target

- `fol-editor` keeps using parse/package/resolve/typecheck directly
- hover/definition/document symbols/completion depend on compiler-owned semantic projections where possible
- text-heuristic fallback behavior is reduced and clearly labeled as fallback
- symbol/type rendering is shared instead of duplicated

Preferred design:

- add compiler-owned helper APIs for:
  - symbol display names
  - type rendering for editor/tooling display
  - semantic lookup by position where feasible
- keep transport/session/editor state in `fol-editor`
- keep only UI-facing policy in `fol-editor`

### Tree-sitter target

- grammar stays handwritten
- query files stay handwritten
- generated assets supply shared language facts
- bundle generation becomes reproducible from compiler-owned manifests plus handwritten editor files

Preferred generated inputs:

- keyword lists
- builtin type names
- intrinsic names grouped by surface
- source kinds
- maybe shell/container family names

Likely handwritten forever:

- grammar precedence/conflicts
- query captures and highlight group policy
- editor-specific movement/textobject queries

## Implementation Slices

### Slice 1: Stabilize tooling test baselines

Before changing architecture, make tooling tests trustworthy again.

Work:

- fix `fol-frontend` crate-local tests so they compile again
  - current state: 15 compile errors from stale imports (`FrontendOutputArgs`, `FrontendProfileArgs`) and wrong argument counts in `src/dispatch.rs`, `src/compile/mod.rs`, `src/build_route/tests/`
- fix `fol-editor` crate-local tests so fixture paths and assertions match the current repo
  - current state: ~28 test failures across lifecycle tests, semantic tests, and tree-sitter corpus tests
  - categories: stale fixture paths, brittle assertion text, tree-sitter parser load failures in CI-like environments
- separate environment-dependent tree-sitter tests from deterministic unit tests if needed
- document what test commands are authoritative for tooling

Exit condition:

- `cargo test -p fol-frontend`
- `cargo test -p fol-editor`

both pass in the normal repo environment, or environment-dependent tests are explicitly isolated and labeled

### Slice 2: Define the diagnostics/editor contract

Write down the exact bridge between `fol-diagnostics` and editor consumers.

Work:

- define the stable fields the editor may rely on:
  - severity
  - code
  - primary location
  - secondary labels
  - notes
  - helps
  - suggestions
- define what the LSP adapter is allowed to transform
- define where dedup happens and why
- define the message-prefix policy (`[CODE] ...`) as a real contract
- document the `glitch_to_diagnostic()` downcast chain in `analysis.rs:235-249`
  - current chain: ParseError, PackageError, ResolverError, TypecheckError
  - missing: LowerError, BuildEvalError
  - fallback: `E9999` unknown code — fragile, should become a trait-based dispatch
- document when `parse_single_file_diagnostics()` is used vs `parse_directory_diagnostics()` and why they differ

Exit condition:

- one written contract exists and tests assert it

### Slice 3: Extract a compiler-owned LSP diagnostic adapter

Move `Diagnostic -> editor/LSP-facing diagnostic payload` out of ad hoc editor code.

Work:

- choose one of:
  - extend `fol-diagnostics` with optional adapter helpers
  - add a new crate such as `fol-diagnostics-lsp`
- move location conversion and message assembly there
- keep the adapter dependency one-way from compiler/core toward tooling
- add focused adapter tests
- replace `glitch_to_diagnostic()` downcast chain with trait-based dispatch
  - all error types already implement `ToDiagnostic` — the downcast chain is redundant
  - parser error flow currently returns `Vec<Box<dyn Glitch>>` which forces downcasting
  - preferred: either convert at production site or add a `Glitch::to_diagnostic()` default
- eliminate `parse_single_file_diagnostics()` temp-dir-write pattern
  - current code (`analysis.rs:154-191`) creates temp dirs, writes files to disk, then parses
  - should use in-memory source/stream construction instead

Preferred result:

- `fol-editor` stops hand-formatting diagnostic messages and related info

Exit condition:

- `fol-editor` consumes the shared adapter instead of rebuilding the mapping locally

### Slice 4: Unify diagnostic suppression policy deliberately

Current CLI and LSP suppression logic are separate.

Work:

- decide which layers own dedup:
  - report-level suppression in `fol-diagnostics`
  - view-level suppression in LSP
- if both remain, document why they differ
- if possible, centralize common suppression helpers in compiler-owned code
- test parser cascades through both CLI and LSP outputs

Exit condition:

- duplicate suppression is intentional, documented, and tested across CLI and LSP

### Slice 5: Export compiler-owned semantic display helpers

Reduce duplicated semantic presentation logic in `fol-editor`.

Work:

- identify display logic currently handwritten in `fol-editor`:
  - `render_checked_type()` in `completion_helpers.rs:214-281` — hand-written match over `CheckedType` variants with hardcoded type spellings (`"int"`, `"flt"`, `"bol"`, `"chr"`, `"str"`, `"never"`, `"opt[...]"`, `"err[...]"`, `"vec[...]"`, `"seq[...]"`, `"set[...]"`, `"map[...]"`)
  - `render_symbol_kind()` in `completion_helpers.rs:175-194` — hand-written match over `SymbolKind`
  - `symbol_kind_code()` in `completion_helpers.rs:196-212` — maps `SymbolKind` to LSP kind numbers
  - `completion_symbol_detail()` in `completion_helpers.rs:358-378` — duplicate of `render_symbol_kind` with minor differences
  - hardcoded builtin type list `["int", "flt", "bol", "chr", "str", "never"]` in `semantic.rs:99`
- move the stable parts into compiler-owned helpers or an editor-facing semantic adapter layer
  - `render_checked_type` belongs in `fol-typecheck` (it only uses `TypeTable` and `CheckedType`)
  - `render_symbol_kind` belongs in `fol-resolver` (it only uses `SymbolKind`)
  - builtin type list belongs in `fol-typecheck` or `fol-lexer`
- keep only UI phrasing and protocol shape in `fol-editor`

Exit condition:

- semantic display strings used by hover/completion are derived from compiler-owned helpers

### Slice 6: Tighten LSP semantic lookups around compiler state

Reduce fallback heuristics where semantic data should exist.

Work:

- audit:
  - hover — compiler-backed via `reference_at()` + `hover_for_reference()`, no text fallback (good)
  - definition — compiler-backed via `reference_at()` + `definition_for_reference()`, no text fallback (good)
  - document symbols — compiler-backed via `document_symbols_for_current_path()`, no text fallback (good)
  - completion — mixed: compiler-backed paths exist but extensive text-heuristic fallbacks run alongside them
- classify each fallback path in completion:
  - `fallback_local_scope_items()` — text-scans for `var ` and `fun` parameters, should be covered by resolver
  - `fallback_current_package_top_level_items()` — text-matches `fun[`, `pro[`, `typ[`, `ali[`, `def[` prefixes
  - `fallback_import_alias_items()` — text-matches `use ` prefix
  - `fallback_imported_package_items()` — reads files from disk and text-scans them
  - `fallback_imported_named_type_items()` — combines alias + package fallbacks
  - `fallback_qualified_completion_items()` — combines namespace + imported fallbacks
  - `fallback_local_namespace_items()` — reads filesystem directories
  - `current_routine_name()` — text-matches nearest `fun` keyword before cursor
  - `fallback_decl_name()` — generic text prefix matcher used by multiple fallbacks
- decide for each: keep as emergency fallback, replace with compiler-backed, or remove
- label remaining fallbacks explicitly (e.g. `// FALLBACK: ...` comments)

Exit condition:

- the semantic feature matrix clearly distinguishes canonical compiler-backed behavior from emergency fallback behavior

### Slice 7: Create a language facts manifest for editor tooling

Introduce a compiler-owned export of syntax-visible facts.

Candidate contents:

- builtin types
- keyword families
- source kinds
- shell/container families
- intrinsic names and surfaces

Current duplication audit (exact locations):

- **primitive types** (`int|bol|str|flt|chr|never`): duplicated in `highlights.scm:57-66`, `semantic.rs:99`, `completion_helpers.rs:220-225`; canonical source is `fol-typecheck/src/types.rs` `BuiltinType` enum
- **container types** (`arr|vec|seq|set|map`): duplicated in `grammar.js:63`, `highlights.scm:48-52`, `completion_helpers.rs:238-254`; canonical source is `fol-parser/src/ast/parser_parts/special_type_parsers.rs`
- **shell types** (`opt|err`): duplicated in `grammar.js:64`, `highlights.scm:53-54`, `completion_helpers.rs:229-232`; canonical source is `fol-parser/src/ast/parser_parts/special_type_parsers.rs`
- **source kinds** (`loc|std|pkg`): duplicated in `grammar.js:41`, `highlights.scm:17-19`; canonical source is `fol-parser/src/ast/parser_parts/source_kind_type_parsers.rs`
- **intrinsics** (`len|echo|eq|nq|lt|gt|le|ge|not`): duplicated in `highlights.scm:85-86`; canonical source is `fol-intrinsics/src/catalog.rs`

Implementation options:

- checked-in generated JSON/TOML manifest
- Rust API plus small generator command
- frontend command such as `fol tool dump language-facts`

Exit condition:

- one canonical source exists for syntax-visible facts currently duplicated across compiler/editor assets

### Slice 8: Generate the easy Tree-sitter inputs

Use the manifest to remove manual duplication where generation is low-risk.

Generate:

- keyword lists used by tests/docs
- intrinsic lists consumed by tree-sitter query validation
- maybe type/source-kind query fragments
- maybe generated snapshot files embedded into the tree bundle

Do not generate:

- full grammar structure
- precedence/conflict declarations
- most query capture design

Exit condition:

- obvious duplicated token/name lists are no longer edited in multiple places

### Slice 9: Put Tree-sitter maintenance on explicit rails

Once generation covers the easy facts, define the remaining handwritten contract.

Work:

- document which files are intentionally handwritten:
  - grammar.js
  - highlights.scm
  - locals.scm
  - symbols.scm
- document exactly when a new syntax feature must update them
- add corpus expectations for each syntax family

Exit condition:

- tree-sitter changes stop being guesswork

### Slice 10: Add a compiler-feature update checklist to docs and tests

Every language feature should go through the same maintenance checklist.

Checklist categories:

- lexer/token changes
- parser/AST changes
- resolver/typecheck/lowering changes
- diagnostics changes
- LSP semantic changes
- tree-sitter grammar/query/corpus changes
- docs/examples changes

Exit condition:

- a contributor can follow one document and know exactly what to inspect when adding a feature

### Slice 11: Add generated or shared tooling smoke tests

Add integration tests that fail when compiler/editor contracts drift.

Examples:

- diagnostic adapter output matches expected LSP shape
- intrinsic registry and editor dot-completion remain in sync
- language-facts manifest matches tree-sitter generated fragments
- strict error diagnostics appear identically through CLI JSON and LSP adaptation

Exit condition:

- future drift between compiler and tooling is caught automatically

### Slice 12: Align FrontendError with the diagnostics pipeline

`fol-frontend` has its own `FrontendError`/`FrontendErrorKind` that bypasses `fol-diagnostics`.

Current state:

- `FrontendError` lives in `lang/tooling/fol-frontend/src/errors.rs`
- `Display` formats as `"FrontendWorkspaceNotFound: message"` — not the `[CODE] message` contract
- no `ToDiagnostic` impl exists
- `From<PackageError>` loses the original diagnostic structure
- notes are carried but not formatted through the diagnostic pipeline

Work:

- implement `ToDiagnostic` for `FrontendError`
- assign stable diagnostic codes (e.g. `F1xxx` family)
- route frontend error display through `fol-diagnostics` rendering
- preserve notes through the diagnostic model
- remove the `FrontendErrorKind::as_str()` prefix pattern from Display

Exit condition:

- frontend errors render through the same diagnostic pipeline as compiler errors

### Slice 13: Consolidate frontend compiler pipelines

`fol-frontend` has two parallel compiler pipeline implementations with different error handling.

Current state:

- **direct mode** (`direct.rs:456-535`): creates FileStream, parses, resolves, typechecks, lowers — uses `DiagnosticReport` with structured JSON output via `add_compiler_glitch()` downcast chain
- **workspace mode** (`compile/mod.rs:500-530`): uses `parse_directory_package_syntax()`, resolves, typechecks, lowers — converts errors to `FrontendError` strings via `lower_resolver_errors()`, `lower_typecheck_errors()`, `lower_lowering_errors()`
- binary execution logic is copy-pasted in 3 places (`compile/mod.rs:205-223`, `compile/mod.rs:268-286`, `direct.rs:297-315`)

Work:

- unify error handling: both modes should produce structured diagnostics, not embed error strings
- extract shared binary execution helper
- decide whether direct mode should reuse the workspace compilation path

Exit condition:

- one error handling strategy across both frontend modes

### Slice 14: Revisit command and crate boundaries

After the shared contracts exist, simplify public ownership.

Questions to settle:

- should `fol tool tree generate` become a consumer of generated manifests plus handwritten assets only?
- should LSP-facing diagnostics/types live in a dedicated adapter crate?
- should editor-facing semantic projections live in `fol-editor` or a compiler-owned helper crate?
- should `EditorError`/`EditorErrorKind` in `fol-editor` also route through `fol-diagnostics`?
- should `RuntimeError` and `BackendError` implement `ToDiagnostic`? (currently separate error domains without diagnostic codes)

Exit condition:

- crate boundaries reflect actual ownership instead of historical accidents

## Workstreams By Area

### A. `fol-diagnostics` <-> `fol-editor`

Needed improvements:

- shared adapter for diagnostic conversion
- shared or explicitly layered dedup policy
- stable message-prefix and related-info rules
- tests that lock the compiler-to-editor diagnostic shape

### B. Compiler semantics <-> LSP

Needed improvements:

- compiler-owned display helpers for types/symbols
- less heuristic fallback logic
- stronger compiler-backed hover/definition/completion paths
- explicit rules for behavior under broken/incomplete documents

### C. Compiler facts <-> Tree-sitter

Needed improvements:

- canonical export of syntax-visible facts
- generated fragments for easy duplicated lists
- handwritten grammar/query policy documented
- corpus coverage tied to language-feature surfaces

### D. Frontend/editor error alignment

Needed improvements:

- `FrontendError` should implement `ToDiagnostic`
- `EditorError` should either implement `ToDiagnostic` or be documented as transport-only
- frontend error display should use `[CODE] message` format
- frontend notes should flow through the diagnostic model

## Generation Policy

### Should be generated when practical

- intrinsic names and surfaces
- builtin type names
- source kinds
- keyword manifests used by tests/docs/tooling validation
- maybe small query fragments tied directly to generated name lists

### Should remain handwritten

- full Tree-sitter grammar
- query capture group policy
- LSP transport and session logic
- semantic UX wording beyond compiler-owned display helpers

### Must never be silently duplicated

- diagnostic code meanings
- intrinsic availability/surface classification
- builtin type spellings
- import source-kind spellings

## Risks

### Risk 1: Over-coupling compiler crates to editor protocols

Mitigation:

- keep protocol-specific wire types out of core compiler crates where possible
- use adapters or companion crates rather than reverse dependencies

### Risk 2: Trying to generate too much Tree-sitter state

Mitigation:

- generate facts, not the whole grammar
- keep grammar/query structure human-owned

### Risk 3: Preserving weak fallback semantics forever

Mitigation:

- classify all editor fallbacks as either temporary, required, or removable
- prefer compiler-backed behavior by default

### Risk 4: Docs drifting again

Mitigation:

- tie the checklist to tests
- require feature PRs to touch docs when syntax/diagnostics/editor behavior changes

## Acceptance Criteria

This plan is complete when:

- tooling crate tests are reliable again
- LSP diagnostic conversion is compiler-owned or compiler-adjacent, not ad hoc editor code
- `glitch_to_diagnostic()` downcast chain is replaced with trait-based dispatch
- `parse_single_file_diagnostics()` no longer writes temp files to disk
- semantic display helpers no longer duplicate compiler logic in `fol-editor`
- a language-facts manifest exists for shared syntax-visible facts
- tree-sitter uses generated data for low-risk duplicated facts
- completion fallback paths are explicitly classified and labeled
- frontend errors route through the diagnostics pipeline
- docs clearly explain what must be updated when adding a feature
- compiler/tooling drift is caught by dedicated integration tests

## Immediate Next Slice

Start with Slice 1.

Rationale:

- the current tooling test baseline is already broken
- deeper integration work is hard to verify until tooling tests become trustworthy
