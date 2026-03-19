# Feature Update Checklist

Use this checklist whenever a new language feature, syntax form, intrinsic, or
error surface is added.

This chapter is about maintenance discipline, not just implementation order.

## Quick Reference

When adding a feature, touch these layers in order:

1. **Lexer** — new keywords, operators, tokens
2. **Parser** — new AST nodes, syntax rules
3. **Semantics** — resolver, typecheck, lowering, intrinsics
4. **Diagnostics** — new error/warning cases
5. **LSP** — hover, completion, definition, symbols
6. **Tree-sitter** — grammar, queries, corpus
7. **Generated facts** — compiler-owned constants
8. **Docs** — language chapters, tooling pages
9. **Tests** — unit, integration, editor, corpus

Automated guards exist for some of these. The `treesitter_sync` integration
tests verify that `highlights.scm` matches compiler-owned constants for builtin
types, intrinsic names, container/shell types, and source kinds. Adding a new
constant to the compiler without updating Tree-sitter will fail those tests.

## 1. Lexical Surface

Check:

- new keywords
- new operators or punctuation
- new literal/token families
- comment or whitespace effects

Update:

- `fol-lexer`
- lexical docs under [`100_lexical`](../100_lexical/_index.md)
- any generated keyword/facts manifest if one exists

## 2. Parser Surface

Check:

- new declarations
- new expressions
- new statement forms
- new type forms
- new precedence or ambiguity rules

Update:

- `fol-parser`
- parser tests
- Tree-sitter grammar if the syntax is editor-visible
- Tree-sitter corpus fixtures for the new syntax family

## 3. Semantic Surface

Check:

- name resolution
- type checking
- lowering
- intrinsic availability
- runtime/backend impact

Update:

- `fol-resolver`
- `fol-typecheck`
- `fol-lower`
- `fol-intrinsics`
- runtime/backend crates if needed

## 4. Diagnostics Surface

Check:

- new error cases
- new warning/info cases
- changed wording for existing rules
- changed labels, notes, helps, or suggestions

Update:

- compiler producer diagnostics
- `fol-diagnostics` contract tests
- editor/LSP diagnostic adapter tests if the visible shape changes
- docs under [`650_errors`](../650_errors/_index.md) if behavior changed materially

## 5. LSP Surface

Check:

- hover content
- go-to-definition behavior
- document symbols
- completion
- open-document analysis behavior under broken code

Update:

- `fol-editor` semantic analysis
- `fol-editor` semantic display helpers or compiler-owned helpers they consume
- LSP tests

Important rule:

- prefer compiler-backed meaning
- use fallback heuristics only when the compiler cannot supply semantic data yet

## 6. Tree-sitter Surface

Check:

- syntax shape visible while typing
- highlight groups
- locals captures
- symbols captures
- corpus examples

Update:

- `tree-sitter/grammar.js`
- `queries/fol/highlights.scm`
- `queries/fol/locals.scm`
- `queries/fol/symbols.scm`
- `tree-sitter/test/corpus/*.txt`

Important rule:

- Tree-sitter is for syntax-facing editor behavior
- it does not replace compiler semantics

## 7. Generated Facts

Check whether the feature adds a new fact that should be exported once instead
of copied by hand.

Examples:

- intrinsic names
- builtin type names
- source kinds
- keyword groups
- shell/container family names

If yes:

- update the compiler-owned source
- regenerate editor-facing artifacts from that source
- do not patch multiple copies manually

## 8. Documentation

Update:

- the language chapter for the feature
- tooling docs if editor behavior changes
- diagnostics docs if compiler reporting changes
- examples/fixtures that demonstrate the preferred form

## 9. Tests

Add or update:

- compiler unit tests
- integration tests
- editor/LSP tests if semantic editor behavior changes
- Tree-sitter tests/corpus if syntax-facing behavior changes

Keep the feature test in the same change as the feature.

## 10. Final Review Questions

Before considering the feature complete, answer:

1. Is compiler meaning implemented?
2. Are diagnostics correct and structured?
3. Does the LSP reflect the new meaning where needed?
4. Does Tree-sitter reflect the new syntax where needed?
5. Did we generate shared facts instead of duplicating them?
6. Did docs explain the intended user-facing behavior?
