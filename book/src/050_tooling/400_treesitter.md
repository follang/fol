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
- point the editor's Tree-sitter parser configuration at that bundle
- let the editor compile/use the parser from there

## File Ownership

### Intentionally Handwritten

These files are human-authored and should remain so:

| File | Purpose | Owner |
|------|---------|-------|
| `tree-sitter/grammar.js` | Grammar rules, precedence, conflicts | Editor/syntax maintainer |
| `queries/fol/highlights.scm` | Highlight capture groups and query patterns | Editor/syntax maintainer |
| `queries/fol/locals.scm` | Scope and definition tracking | Editor/syntax maintainer |
| `queries/fol/symbols.scm` | Symbol navigation captures | Editor/syntax maintainer |
| `test/corpus/*.txt` | Corpus fixtures for grammar validation | Editor/syntax maintainer |

Do not attempt to generate these from the compiler parser. The parsing models
are different and auto-generation creates more fragility than value.

### Validated Against Compiler Constants

These facts appear in handwritten files but are validated by integration tests
to stay in sync with compiler-owned constants:

| Fact | Handwritten Location | Compiler Source |
|------|---------------------|-----------------|
| Builtin type names | `highlights.scm` regex `^(int\|bol\|...)$` | `BuiltinType::ALL_NAMES` in `fol-typecheck` |
| Dot-call intrinsic names | `highlights.scm` regex `^(len\|echo\|...)$` | Implemented `DotRootCall` entries in `fol-intrinsics` |
| Container type names | `highlights.scm` node labels + `grammar.js` choice | `CONTAINER_TYPE_NAMES` in `fol-parser` |
| Shell type names | `highlights.scm` node labels + `grammar.js` choice | `SHELL_TYPE_NAMES` in `fol-parser` |
| Source kind names | `highlights.scm` node labels + `grammar.js` choice | `SOURCE_KIND_NAMES` in `fol-parser` |

The sync tests live in `test/run_tests.rs` under `treesitter_sync`. If you add
a new builtin type, intrinsic, container, shell, or source kind to the compiler,
these tests fail until the tree-sitter files are updated to match.

## When To Update Tree-sitter Files

### grammar.js

Update when:

- A new declaration form is added (e.g. a new `seg` or `lab` declaration)
- A new expression or statement form is added
- A new type syntax is added (e.g. a new container or shell family)
- A new source kind is added
- Operator precedence or conflict rules change

Do not update for:

- New diagnostic codes or error messages
- New resolver/typecheck rules that don't change syntax
- New intrinsics (these use existing `dot_intrinsic` grammar rule)

### highlights.scm

Update when:

- A new keyword needs highlighting
- A new builtin type is added to the compiler
- A new implemented dot-call intrinsic is added
- A new container or shell type family is added
- A new source kind is added
- Highlight group policy changes (e.g. moving a keyword from `@keyword` to `@keyword.function`)

### locals.scm

Update when:

- Scope rules change (e.g. new block forms that introduce scopes)
- Definition capture patterns change

### symbols.scm

Update when:

- New declaration forms should appear in document symbol navigation

### Corpus fixtures

Update when:

- Any grammar rule is added or modified
- A new syntax family needs parse-tree validation

## Corpus Coverage Expectations

Corpus fixtures live in `tree-sitter/test/corpus/` and cover syntax families:

| Corpus File | Covers |
|-------------|--------|
| `declarations.txt` | `use`, `ali`, `typ`, `fun`, `log`, `var` declarations |
| `expressions.txt` | Intrinsic calls, `when`/`loop` control flow, `break`/`return` |
| `recoverable.txt` | Error propagation (`/`), `report`, pipe-or (`\|\|`) |

When a new syntax family is added, it should have corpus coverage. The expected
families that should each have at least one corpus example:

- Import declarations (`use`)
- Type declarations (`typ`, `ali`)
- Routine declarations (`fun`, `log`)
- Variable declarations (`var`)
- Control flow (`when`, `loop`, `case`, `break`, `return`)
- Expressions (binary, call, field access, dot intrinsic)
- Container and shell types
- Record and entry types
- Error handling (`report`, pipe-or, `check`)

## What Tree-sitter Is For

Use the Tree-sitter layer for:

- highlighting
- locals and capture queries
- symbol-style structural views
- editor textobjects and movement later

Do not use it as a substitute for typechecking or resolution.

Those remain compiler tasks.

When a language feature changes syntax, use the
[Feature Update Checklist](./450_feature_checklist.md) to decide whether the
grammar, queries, corpus, or generated language facts also need updates.
