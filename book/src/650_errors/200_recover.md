# Recoverable errors

Recoverable errors are part of the current `V1` compiler contract.

FOL routines may declare both:

- a success type
- an error type

The current signature form is:

```fol
fun load(path: str): int / str = { ... }
```

This means:

- successful execution yields `int`
- failure yields `str`
- `report expr` exits through the routine's error path

## Declaring error-aware routines

Use `/ ErrorType` after the success type:

```fol
fun load(path: str): int / str = {
    when(path == "") {
        case(true) { report "empty path" }
        * { return 1 }
    }
}
```

`report expr` must produce a value compatible with the declared `ErrorType`.

This is checked by the current compiler. The following is invalid:

```fol
fun load(path: str): int / str = {
    report 1
}
```

because `1` is `int`, while the routine declares `str` as its error type.

The same rule applies to methods:

```fol
typ Parser: rec = {
    name: str
}

fun (Parser)parse_err(code: int): str = {
    return "bad-input"
}

fun run(tool: Parser, code: int): int / str = {
    report tool.parse_err(code)
}
```

## Propagation

Calling an error-aware routine in a plain value position propagates the error
upward, but only inside another error-aware routine with a compatible error
type.

```fol
fun read_code(path: str): int / str = {
    when(path == "") {
        case(true) { report "missing path" }
        * { return 7 }
    }
}

fun main(path: str): int / str = {
    return read_code(path)
}
```

This works because `main` also declares `/ str`.

If the surrounding routine does not declare a compatible error type, the call is
rejected during type checking.

## Inspecting a recoverable call

`check(expr)` is the `V1` surface for asking whether an error-aware routine call
failed.

It returns `bol`.

```fol
fun read_code(path: str): int / str = {
    when(path == "") {
        case(true) { report "missing path" }
        * { return 7 }
    }
}

fun main(path: str): int / str = {
    var value: int = read_code(path) || 0

    when(check(read_code(path))) {
        case(true) { report "read failed" }
        * { return value }
    }
}
```

## Recovering with `||`

`expr || fallback` is the `V1` shorthand for recovering from an error-aware
routine call.

Rules:

- if `expr` succeeds, use its success value
- if `expr` fails, evaluate `fallback`
- `fallback` may:
  - provide a default value
  - `report`
  - `panic`

Examples:

```fol
fun read_code(path: str): int / str = {
    when(path == "") {
        case(true) { report "missing path" }
        * { return 7 }
    }
}

fun with_default(path: str): int = {
    return read_code(path) || 0
}

fun propagate_with_context(path: str): int / str = {
    return read_code(path) || report "read failed"
}

fun must_succeed(path: str): int = {
    return read_code(path) || panic "read failed"
}
```

## Routine errors are not `err[...]` shells

Routine call results with `/ ErrorType` are not the same thing as `err[...]`
shell values.

That distinction is important in the current compiler:

- `check(expr)` and `expr || fallback` work on error-aware routine calls
- postfix `!` works on shell values such as `opt[...]` and `err[...]`
- postfix `!` does not unwrap routine call results declared with `/ ErrorType`

Examples:

```fol
var failure: err[str] = "broken"
var value: str = failure!
```

This is a shell unwrap.

The following is a different surface:

```fol
fun read_code(path: str): int / str = { ... }
```

Here the call result is a recoverable routine result, not an `err[...]` shell.
Use propagation, `check(...)`, or `||` with it.

## Current V1 boundary

The current compiler supports:

- declared routine error types with `/`
- `report expr`
- propagation through compatible routine contexts
- `check(expr)`
- `expr || fallback`
- lowering of these surfaces into explicit recoverable-error IR

The current compiler does not yet claim:

- advanced Zig-style success/error capture syntax
- cleanup constructs like `errdefer`
- full backend/native runtime ABI documentation in the book

Those belong to later milestones.
