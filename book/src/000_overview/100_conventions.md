# Notation And Conventions

This book is the language specification for FOL.

It is written as a spec first:
- examples describe intended language behavior
- implementation status may lag behind the book
- when code and prose disagree, the disagreement should be resolved explicitly

## Reading Syntax Examples

The examples in this book follow a few conventions.

- Keywords are written literally:
  - `fun`
  - `pro`
  - `typ`
  - `use`
- Punctuation is written literally:
  - `(`
  - `)`
  - `[`
  - `]`
  - `{`
  - `}`
  - `:`
  - `=`
- Placeholder names are descriptive:
  - `name`
  - `type`
  - `expr`
  - `body`
  - `source`

For example:

```fol
fun[options] name(params): return_type = { body }
```

means:
- `fun` is a literal keyword
- `options` is a placeholder for zero or more routine options
- `name` is the declared routine name
- `params` is a parameter list
- `return_type` is a type reference
- `body` is a routine body

## Spec Vocabulary

The book uses the following terms consistently:

- declaration:
  a top-level or block-level form that introduces a named entity
- statement:
  an executable form that appears in a block
- expression:
  a form that produces a value
- type reference:
  a syntactic reference to a type
- type definition:
  a declaration form that creates a new named type
- source kind:
  an import/source locator family such as `loc`, `url`, or `std`

## Examples vs Grammar

Examples are illustrative, not exhaustive.

When a chapter gives one or two examples, that does not imply the syntax is limited to only those exact spellings. The normative rule is the chapter text plus the grammar intent described there.

## Terminology Preferences

This book prefers the following terms:

- routine:
  umbrella term for `fun`, `pro`, and `log`
- record:
  named field-based type
- entry:
  named variant-based type
- standard:
  protocol/blueprint/extension-style contract surface
- module:
  namespace/package surface addressed through `use`, `def`, or `seg`

## Status Notes

Some older chapters still use inconsistent wording inherited from earlier drafts.

During cleanup, the following principles apply:
- keep examples unless they are contradictory
- prefer clarifying rewrites over removal
- keep the chapter tree stable where possible
- make chapter indexes explain scope before detail
