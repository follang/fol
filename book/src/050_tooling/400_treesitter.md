# Tree-sitter Integration

The Tree-sitter side of FOL is the editor-facing syntax layer.

It is not the compiler parser.

## What Is In The Repo

The editor crate carries:

- the grammar source
- corpus fixtures
- query files on disk

Canonical query assets live as real files, not just embedded Rust strings:

- `queries/fol/highlights.scm`
- `queries/fol/locals.scm`
- `queries/fol/symbols.scm`

This is intentional because editors such as Neovim expect query files on disk in
the standard Tree-sitter layout.

## Generated Bundle

To generate a Neovim-consumable bundle, run:

```text
fol tool tree generate /tmp/fol
```

That writes a bundle containing the grammar and query assets under the target
directory.

The intended consumer path is:

- generate bundle
- point the editor’s Tree-sitter parser configuration at that bundle
- let the editor compile/use the parser from there

## Why Use `fol tool tree generate`

This command exists so editor integration can consume a generated FOL parser
bundle without forcing you to manually copy query files around.

It also gives one stable place to inspect what the editor will actually use.

## What Tree-sitter Is For

Use the Tree-sitter layer for:

- highlighting
- locals and capture queries
- symbol-style structural views
- editor textobjects and movement later

Do not use it as a substitute for typechecking or resolution.

Those remain compiler tasks.
