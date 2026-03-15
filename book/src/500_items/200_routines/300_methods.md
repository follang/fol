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

Current parser-supported receiver syntax is intentionally broader than a named-only rule.
At parse time, receiver positions accept named, qualified, builtin-scalar, and
bracketed/composite type references. This keeps extension-style examples such as
`typ[ext] int: int; pro (int)print(): non = { ... }` and dispatch examples on extended
builtin aliases in scope for the front-end.

The dedicated parser-level rejection in this hardening phase is still for special
builtin forms such as `any`, `none`, and `non`.

Invalid example:

```fol
fun (any)parse_msg(code: int): str = {
    return "ok";
}
```

This form reports: `Method receiver type cannot be any, non, or none`.

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

fun run(tool: parser, code: int): int / str = {
    report tool.parse_err(code)
    return 0
}
```
