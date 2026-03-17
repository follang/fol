# Editor Tooling

FOL editor support lives in one crate:

- `fol-editor`

That crate owns two related subsystems:

- Tree-sitter assets for syntax-oriented editor work
- the language server for compiler-backed editor services

## Public Entry

The public entrypoints are exposed through `fol tool`:

- `fol tool lsp`
- `fol tool parse <PATH>`
- `fol tool highlight <PATH>`
- `fol tool symbols <PATH>`
- `fol tool tree generate <PATH>`

This keeps editor workflows under the same `fol` binary rather than introducing
a second public tool.

## Split Of Responsibilities

Tree-sitter is the editor syntax layer.

It is responsible for:

- syntax trees while typing
- query-driven highlighting
- locals and symbol-style structure captures
- editor-facing structural parsing

The language server is the semantic editor layer.

It is responsible for:

- JSON-RPC/LSP transport
- open-document state
- compiler-backed diagnostics
- hover
- go-to-definition
- document symbols
- completion, as that feature grows

## Compiler Truth

`fol-editor` does not create a second semantic engine.

Semantic truth still comes from:

- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-diagnostics`

So the model is:

- Tree-sitter answers “what does this text structurally look like?”
- compiler crates answer “what does this code mean?”

## Current Practical Workflow

Use:

```text
fol tool lsp
```

as the language server entrypoint.

Use:

```text
fol tool parse path/to/file.fol
fol tool highlight path/to/file.fol
fol tool symbols path/to/file.fol
```

for parser/query debugging and validation.
