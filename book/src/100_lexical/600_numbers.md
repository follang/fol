# Numbers

A number is either an integer, floating-point or imaginary. The grammar for recognizing the kind of number is mixed.

## Intigers
An integer has one of four forms:

- A decimal literal starts with a decimal digit and continues with any mixture of decimal digits and underscores.
- A hex literal starts with the character sequence `U+0030` `U+0078` (`0x`) and continues as any mixture (with at least one digit) of hex digits and underscores.
- An octal literal starts with the character sequence `U+0030 U+006F` (`0o`) and continues as any mixture (with at least one digit) of octal digits and underscores.
- A binary literal starts with the character sequence `U+0030 U+0062` (`0b`) and continues as any mixture (with at least one digit) of binary digits and underscores.

```
var decimal: int = 45;
var hexadec: int = 0x6HF53BD5;
var octal: int = 0o822371;
var binary: int = 0b010010010;
```
### Underscore

Underscore character `U+005F` (`_`) is a special character, that does not represent anything withing the number laterals. An integer lateral containing this character is the same as the one without. It is used only as a syntastc sugar:

```
var aNumber: int = 540_467;
var bNumber: int = 540467;

assert(aNumber, bNumber)
```

## Floating points

A floating-point has one of two forms:

- A decimal literal followed by a period character `U+002E` (`.`). This is optionally followed by another decimal literal.
- A decimal literal that follows a period character `U+002E` (`.`).

```
var aFloat: flt = 3.4;
var bFloat: flt = .4;
```

## Imaginary numbers
