# FOL Editor Plan

Last updated: 2026-03-17

This plan replaces the closed frontend/package-workflow record.

The next milestone is a single editor-tooling crate:

- `fol-editor`

That crate will own both:

- the Tree-sitter grammar and query assets for editor syntax/structure work
- the language server implementation for diagnostics and navigation

The frontend should expose that functionality under:

- `fol editor ...`

The goal is not to create a second compiler. The goal is to expose the current
compiler and package truth through editor-facing tooling.

## Desired End State

At closeout, the repo should support this workflow:

```text
fol editor lsp
fol editor parse path/to/file.fol
fol editor highlight path/to/file.fol
fol editor symbols path/to/file.fol
```

And editor integrations should have:

- a working Tree-sitter grammar for current `V1`
- highlight queries
- basic locals/symbol extraction queries
- an LSP server that can:
  - initialize
  - track open documents
  - publish diagnostics
  - answer hover
  - answer go-to-definition
  - answer document symbols

## Boundary

`fol-editor` should own:

- Tree-sitter grammar files
- Tree-sitter corpus tests
- editor queries
- LSP transport/runtime
- open-document state
- file-to-package/workspace mapping for editor requests
- conversion between canonical `fol-diagnostics` reports and LSP diagnostics
- navigation/symbol response assembly

It should not own:

- package metadata parsing
- name resolution semantics
- typechecking semantics
- lowering semantics
- backend behavior

Those stay in:

- `fol-diagnostics`
- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-lower`

## Crate Shape

Use one crate:

- `fol-editor`

Internal module split:

- `tree_sitter`
- `queries`
- `documents`
- `workspace`
- `lsp`
- `convert`

The Tree-sitter side and LSP side live together, but remain internally
separated.

## Frontend Surface

Add an `editor` command family to `fol-frontend`.

Initial public commands:

- `fol editor lsp`
- `fol editor parse <PATH>`
- `fol editor highlight <PATH>`
- `fol editor symbols <PATH>`

The frontend remains orchestration-only:

- parse command
- discover package/workspace roots if needed
- call `fol-editor`
- render output in human/plain/json

## Tree-sitter Scope

The Tree-sitter grammar should cover the current implemented language boundary,
not future-doc syntax.

Target `V1` syntax:

- top-level declarations:
  - `use`
  - `var`
  - `fun`
  - `log`
  - `typ`
  - `ali`
- records and entries
- routines and methods
- block expressions
- `when`
- `loop`
- `return`
- `report`
- `break`
- typed container literals
- shell literals like `nil`
- postfix `!`
- qualified paths
- dot intrinsics
- comments/doc comments

Do not try to encode typechecking or resolution into the grammar.

## Query Scope

Initial query files should support:

- highlights
- injections if needed later
- document symbols
- locals/definitions references for editor structure

The initial required query set is:

- `highlights.scm`
- `locals.scm`
- `symbols.scm`

## LSP Scope

The first LSP milestone should be practical, not maximal.

Required first features:

- `initialize`
- `initialized`
- `shutdown`
- `exit`
- `textDocument/didOpen`
- `textDocument/didChange`
- `textDocument/didClose`
- `textDocument/hover`
- `textDocument/definition`
- `textDocument/documentSymbol`
- `textDocument/publishDiagnostics`

The LSP should begin with full-document reanalysis, not deep incremental
semantic invalidation.

## Analysis Model

The LSP should reuse compiler truth:

1. map file path to package/workspace root
2. load the current package/workspace view
3. replace the in-memory text for the active file
4. run parse/package/resolve/typecheck as needed
5. convert canonical `fol-diagnostics` reports and semantic results into LSP structures

Do not build a separate semantic analyzer in `fol-editor`.

`fol-diagnostics` is the canonical compiler diagnostic model.

So:

- CLI compiler diagnostics
- future LSP diagnostics

should both derive from the same `fol-diagnostics` report shapes instead of
inventing separate compiler-diagnostic models.

## Tree-sitter Vs Compiler Parser

The Tree-sitter grammar is for editors.

The compiler parser remains the source of compiler truth.

So:

- Tree-sitter should be editor-correct and stable
- compiler parser should remain semantic/canonical for compilation

The two should be tested against the same fixture corpus where practical, but
they do not need to share an AST model.

## Test Strategy

The milestone should be proven with:

- Tree-sitter corpus tests
- query snapshot tests
- LSP request/response tests
- real fixture files from `examples/` and `test/apps/fixtures/`
- frontend integration tests for `fol editor ...`

The minimum real fixture sources should include:

- single-file package
- multi-file package
- nested namespaces
- `loc` import package
- records, entries, aliases, methods
- recoverable routines
- containers and intrinsics

## Phases

### Phase 0: Freeze Scope

- `0.1` replace the closed frontend plan with the `fol-editor` plan
- `0.2` freeze the one-crate boundary for Tree-sitter plus LSP
- `0.3` freeze the frontend command surface under `fol editor`
- `0.4` freeze the “compiler truth, editor adapter” architecture

### Phase 1: Crate Foundation

- `1.1` add the `fol-editor` workspace crate
- `1.2` add the public crate API shell
- `1.3` add structured editor-tooling error types
- `1.4` add shared document URI/path helpers
- `1.5` add a document store model
- `1.6` add editor config/session shells

### Phase 2: Tree-sitter Grammar Foundation

- `2.1` add the Tree-sitter grammar scaffold
- `2.2` add base lexical tokens
- `2.3` add declaration parsing rules
- `2.4` add routine/type/body parsing rules
- `2.5` add expression and control-flow parsing rules
- `2.6` add comments and doc-comment handling
- `2.7` add error-recovery rules suitable for editors
- `2.8` add corpus smoke tests

### Phase 3: Tree-sitter V1 Coverage

- `3.1` cover `use` declarations and source kinds
- `3.2` cover variables and typed bindings
- `3.3` cover functions, logicals, and methods
- `3.4` cover records, entries, and aliases
- `3.5` cover `when`, `loop`, `return`, `report`, and `break`
- `3.6` cover qualified paths and dot intrinsics
- `3.7` cover containers, shells, `nil`, and postfix `!`
- `3.8` add corpus fixtures from real `V1` examples

### Phase 4: Query Layer

- `4.1` add `highlights.scm`
- `4.2` highlight declarations, types, keywords, and literals
- `4.3` highlight intrinsics and qualified paths
- `4.4` add `locals.scm`
- `4.5` add `symbols.scm`
- `4.6` snapshot query captures on real fixtures

### Phase 5: Frontend Editor Commands

- `5.1` add `editor` command family to `fol-frontend`
- `5.2` add `fol editor lsp`
- `5.3` add `fol editor parse`
- `5.4` add `fol editor highlight`
- `5.5` add `fol editor symbols`
- `5.6` add human/plain/json output for editor subcommands
- `5.7` add frontend integration tests for editor commands

### Phase 6: LSP Foundation

- `6.1` add LSP transport/session shell
- `6.2` add JSON-RPC message models
- `6.3` add initialize/shutdown/exit handling
- `6.4` add text document open/change/close tracking
- `6.5` add workspace root mapping from open files
- `6.6` add server smoke tests

### Phase 7: Diagnostics

- `7.1` map open documents into compiler package/workspace analysis
- `7.2` publish parser diagnostics from `fol-diagnostics`
- `7.3` publish package/loading diagnostics from `fol-diagnostics`
- `7.4` publish resolver diagnostics from `fol-diagnostics`
- `7.5` publish typecheck diagnostics from `fol-diagnostics`
- `7.6` convert `fol-diagnostics` locations and related labels into LSP ranges
- `7.7` add diagnostics integration tests on real fixtures

### Phase 8: Hover And Definition

- `8.1` add hover request handling
- `8.2` expose symbol/type summary text for hover
- `8.3` add go-to-definition for local declarations
- `8.4` add go-to-definition for imported symbols
- `8.5` add integration tests for hover/definition

### Phase 9: Symbols

- `9.1` add document-symbol extraction via compiler/tree-sitter data
- `9.2` include routines, types, bindings, and namespaces
- `9.3` keep symbol hierarchy stable for nested items
- `9.4` add integration tests for document symbols

### Phase 10: Hardening

- `10.1` add editor-facing guidance for workspace-not-found situations
- `10.2` add editor-facing guidance for unsupported future-version syntax
- `10.3` add stable JSON error shapes for CLI editor subcommands
- `10.4` add deterministic fixture snapshots for parse/highlight/symbol output
- `10.5` lock real examples through tree-sitter and LSP test paths

### Phase 11: Docs Closeout

- `11.1` update the frontend book chapter with `fol editor ...`
- `11.2` add a dedicated book entry for editor tooling
- `11.3` update `README.md`
- `11.4` update `PROGRESS.md`
- `11.5` close the plan only after Tree-sitter, queries, LSP basics, and frontend exposure are real

## Definition Of Done

Do not close this plan until all of this is true:

- `fol-editor` exists as a workspace crate
- Tree-sitter grammar covers the current implemented `V1` syntax
- highlight/local/symbol queries exist and are tested
- `fol editor lsp` runs
- the LSP can publish diagnostics for open files
- hover and go-to-definition work for the initial supported cases
- document symbols work
- `fol editor parse/highlight/symbols` work through `fol-frontend`
- docs describe the implemented editor workflow accurately
