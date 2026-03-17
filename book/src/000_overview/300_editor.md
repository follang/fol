# Editor Tooling

FOL now has a dedicated editor-tooling crate:

- `fol-editor`

That crate owns two related but separate editor surfaces:

- Tree-sitter assets for syntax-oriented editor work
- a language server for compiler-backed editor services

## Public Entry

The public entrypoint stays under the frontend:

- `fol editor lsp`
- `fol editor parse <PATH>`
- `fol editor highlight <PATH>`
- `fol editor symbols <PATH>`

So editor workflows remain part of the main `fol` tool instead of introducing a
second standalone user command.

## Tree-sitter Role

The Tree-sitter side is the editor syntax layer.

It owns:

- the grammar source
- corpus fixtures
- `highlights.scm`
- `locals.scm`
- `symbols.scm`

Its job is:

- syntax trees while typing
- highlighting
- locals and symbol-style structure queries
- editor-oriented recovery on incomplete text

It is not the compiler parser.

## LSP Role

The language server is the semantic editor layer.

It owns:

- JSON-RPC transport
- open-document tracking
- file-to-package/workspace mapping
- conversion into LSP responses

Its current first-milestone features are:

- initialize / shutdown / exit
- open / change / close tracking
- diagnostics
- hover
- go-to-definition
- document symbols

## Compiler Truth

`fol-editor` does not create a second semantic engine.

Semantic truth still comes from compiler crates:

- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-diagnostics`

That means:

- Tree-sitter gives editor structure
- compiler crates give symbol/type/diagnostic truth

## Diagnostics

LSP diagnostics are adapted from canonical compiler diagnostics.

So the same producer-owned diagnostic shape now serves:

- CLI rendering
- JSON output
- editor/LSP diagnostics

This keeps the editor path aligned with the compiler instead of duplicating
error logic inside the language server.

## Current Boundary

The current editor milestone is the first practical layer, not a full IDE
platform.

It already covers:

- Tree-sitter grammar for current `V1`
- on-disk query files in editor-consumable layout
- frontend-owned `fol editor ...` commands
- LSP diagnostics for parser/package/resolver/typecheck failures
- hover and definition for initial supported cases
- document symbols

Later editor work such as references, rename, completion, semantic tokens, and
formatting belongs to follow-on milestones.
