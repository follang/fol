# Comments

Backtick-delimited comments are the authoritative comment syntax in FOL.

## Normal comments

Single-line and multiline comments use the same backtick-delimited form.

SINGLE_LINE_COMMENT:
```
`this is a single line comment`
```

MULTI_LINE_COMMENT:
```
`this is a
multi
line
comment`
```

## Doc comments

Documentation comments use the `[doc]` prefix inside the same backtick-delimited
comment family.

DOC_COMMENT:
```
`[doc] this is a documentation comment`
```

## Current front-end compatibility

The hardened front-end still accepts `//` and `/* ... */` comments as frozen
compatibility syntax. They are not the authoritative book spelling, but they remain
intentionally supported by the current lexer and parser.

The current front-end also preserves comment kind and raw spelling past lexing. In the
parser today, standalone root comments and standalone routine-body comments lower to
explicit AST comment nodes, and many inline expression-owned comments now survive
through `AstNode::Commented` wrappers around the parsed node they belong to. That
gives later doc-comment tooling retained comment content to build on instead of
re-scanning raw source text.
