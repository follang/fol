# Recoverable errors

`report` can be used to handle recoverable errors. As [discussed here](/docs/spec/functions/#return), FOL uses two variables `result` nd `error` in return of each routine. As name implies, `result` represents the type of the value that will be returned in a success case, and `error` represents the type of the error `err[]` that will be returned in a failure case.

To declare routine-local custom error type, use function/procedure signature form:

```
fun/pro name(args): ResultType / ErrorType = { ... }
```

`report expr` should produce a value compatible with `ErrorType`.

The current hardening-phase front end does not type-check `report` expressions.
The parser keeps `report` as a syntax form and leaves compatibility checks,
forward declaration checks, and error propagation rules to later semantic work.

```fol
fun load(path: str): int / str = {
    report make_err(path)
    return 0
}

fun make_err(path: str): str = {
    return "missing: " + path
}
```

The same rule applies to receiver methods:

```fol
fun run(tool: parser, code: int): int / str = {
    report tool.parse_err(code)
    return 0
}

fun (parser)parse_err(code: int): str = {
    return "bad-input"
}
```

If the forward-declared callee return type is incompatible with the routine error
type, that is a later semantic error rather than a parser error in the current
front end.

```fol
fun load_bad(path: str): int / str = {
    report make_code(path)
    return 0
}

fun make_code(path: str): int = {
    return 1
}
```

This is invalid because `report make_code(path)` produces `int` while the routine
error type is `str`, but the current parser does not diagnose that mismatch on its
own.

When we use the keyword `report`, the error is returned to the routine's error variable and the routine qutis executing (the routine, not the program).
```
use file: std = {"fs/file"}

pro main(): int = {
    pro[] fileReader(path: str): str = {
        var aFile = file.readfile(path)
        if ( check(aFile) ) {
            report "File could not be opened" + file                        // report will not break the program, but will return the error here, and the routine will stop
        } else {
            return file.to_string()                                         // this will be executed only if file was oopened without error
        }
    }
}
```

Form this point on, the error is concatinated up to the main function. This is known as propagating the error and gives more control to the calling code, where there might be more information or logic that dictates how the error should be handled than what you have available in the context of your code.

```
use file: std = {"fs/file"}

pro main(): int = {
    var f = file.open("main.jpg");                                           // main.jpg doesn't exist
    if (check(f)) {
        report "File could not be opened" + file                             // report will not break the program
    } else {
        .echo("File was open sucessfulley")                                  // this will be executed only if file was oopened without error
    }
}
```
