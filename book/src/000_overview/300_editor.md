# Editor Tooling

FOL now has a dedicated editor-tooling crate:

- `fol-editor`

It owns:

- Tree-sitter assets for syntax/highlighting/query work
- the language server for compiler-backed editor services
- build-file affordances for `build.fol` through the same parse/highlight/symbol
  and LSP surfaces used for ordinary source files

The detailed operational reference now lives in the Tooling section:

- [Editor Tooling](../050_tooling/300_editor.md)
- [Tree-sitter Integration](../050_tooling/400_treesitter.md)
- [Language Server](../050_tooling/500_lsp.md)
- [Neovim Integration](../050_tooling/600_neovim.md)

Use this page as the overview pointer, not the full reference.
