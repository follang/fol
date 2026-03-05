# Methods

A method is a routine associated with a receiver type. In FOL, methods can be declared as either `fun` or `pro` and called with dot syntax (`value.method(...)`).

Current parser-supported receiver declaration syntax is:

```fol
fun (parser)parse_msg(code: int): str = {
    return "ok";
}

pro (parser)update(code: int): int = {
    return code;
}
```

The receiver type appears in parentheses right after `fun` or `pro`, followed by the method name.

Receiver types must be user-defined named types. Builtin scalar types (like `int`, `bol`, `str`, `flt`) are rejected for method receiver declarations.

Invalid example:

```fol
fun (int)parse_msg(code: int): str = {
    return "ok";
}
```

This form reports: `Method receiver type must be a user-defined named type`.

Method calls use standard dot syntax:

```fol
var tool: parser = parser.new()
var msg: str = tool.parse_msg(10)
```

Custom error routines also support reporting method call results when receiver-qualified signatures are available:

```fol
fun (parser)parse_err(code: int): str = {
    return "bad-input";
}

fun run(tool: parser, code: int): int : str = {
    report tool.parse_err(code)
    return 0
}
```
