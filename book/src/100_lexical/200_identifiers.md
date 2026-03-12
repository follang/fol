# Identifiers

Identifiers in the current front-end are ASCII names built from letters, digits, and
underscores, but they may not start with a digit. Repeated underscore runs `__` are
not allowed.

IDENTIFIER:
```
[a-z A-Z _] [a-z A-Z 0-9 _]*
```

The hardened front-end currently accepts:

- leading underscores
- internal underscores
- non-leading digits

The hardened front-end currently rejects:

- leading digits
- repeated underscore runs
- non-ASCII identifier spellings

`_` by itself is still accepted by the current lexer/parser boundary as a dedicated
placeholder or binder surface. It should not be treated as an ordinary named identifier
for later-phase semantic work.

## Identifier equality

Parser-owned duplicate checks currently treat two identifiers as equal if the following
algorithm returns true:

```
pro sameIdentifier(a, b: string): bol = {
    result = a.replace("_", "").toLowerAscii == b.replace("_", "").toLowerAscii
}
```

That means ASCII letters are compared case-insensitively and underscores are ignored
for those parser-owned duplicate checks. The lexer and stream still preserve original
identifier spelling; they do not canonicalize token or namespace text up front.
