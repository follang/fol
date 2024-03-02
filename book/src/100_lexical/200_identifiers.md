# Identifiers

Identifiers in FOL can be any string of letters, digits and underscores, but beginning with a letter. Two immediate following underscores __ are not allowed.

IDENTIFIER:
[a-z A-Z] [a-z A-Z 0-9 _]* | _ [a-z A-Z 0-9 _]+


An identifier is any nonempty ASCII string of the following form:

Either

- The first character is a letter.
- The remaining characters are alphanumeric or _.

Or

- The first character is _.
- The identifier is more than one character. _ alone is not an identifier.
- The remaining characters are alphanumeric or _.

## Identifier equality

Two identifiers are considered equal if the following algorithm returns true:

```
pro sameIdentifier(a, b: string): bol = {
    result = a.replace("_", "").toLowerAscii == b.replace("_", "").toLowerAscii
}
```

That means all letters are compared case insensitively within the ASCII range and underscores are ignored. This rather unorthodox way to do identifier comparisons is called partial case insensitivity and has some advantages over the conventional case sensitivity: It allows programmers to mostly use their own preferred spelling style, be it humpStyle or snake_style, and libraries written by different programmers cannot use incompatible conventions
