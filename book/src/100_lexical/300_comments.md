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

The hardened front-end still accepts `//` and `/* ... */` comments as compatibility
syntax. They are not the authoritative book syntax, but they remain intentionally
supported by the current lexer and parser.
