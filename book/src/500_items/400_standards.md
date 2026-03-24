# Standards

This chapter describes later `V2` contract and conformance design, not current
`V1` compiler behavior.

Current milestone note:

- standards are not part of the implemented `V1` typechecker
- blueprints and extensions are not part of the implemented `V1` typechecker
- examples here are future design sketches only

The intent of standards is procedural and data-oriented. They are not class
hierarchies, inheritance trees, or object systems.

## Standard

In later milestones, a standard is intended to be a named collection of
required receiver-qualified routine signatures and/or required data, created
with `std`.

```fol
std geometry: pro = {
    fun area(): flt[64];
    fun perim(): flt[64];
};
```

The planned forms are:

- protocol `pro[]` for required routines
- blueprint `blu[]` for required data
- extended `ext[]` for routines plus data

```fol
std geometry: pro = {
    fun area(): flt[64];
    fun perim(): flt[64];
};

std geometry: blu = {
    var color: rgb;
    var size: int;
};

std geometry: ext = {
    fun area(): flt[64];
    fun perim(): flt[64];
    var color: rgb;
    var size: int;
};
```

## Contract

Later milestones may allow a type to declare that it satisfies a standard. The
compiler would then check that the required data and receiver-qualified
routines exist.

```fol
std geo: pro = {
    fun area(): flt[64];
    fun perim(): flt[64];
};

std rect(geo): rec[] = {
    width: int[64];
    heigh: int[64];
}
```

Under that design, `rect` would need matching receiver-qualified routines such
as:

```fol
fun (rect)area(): flt[64] = { result = self.width + self.heigh }
fun (rect)perim(): flt[64] = { result = 2 * self.width + 2 * self.heigh }
```

The goal is still procedural. A call like `shape.area()` would remain sugar
for a receiver-qualified routine call, not an object-owned virtual method.
