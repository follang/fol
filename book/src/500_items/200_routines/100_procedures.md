# Procedures

Procedures are most common type of routines in Fol. When a procedure is "called" the program "leaves" the current section of code and begins to execute the first line inside the procedure. Thus the procedure "flow of control" is:

- The program comes to a line of code containing a "procedure call".
- The program enters the procedure (starts at the first line in the procedure code).
- All instructions inside of the procedure are executed from top to bottom.
- The program leaves the procedure and goes back to where it started from.
- Any data computed and RETURNED by the procedure is used in place of the procedure in the original line of code.

Procedures have side-effects, it can modifies some state variable value(s) outside its local environment, that is to say has an observable effect besides returning a value (the main effect) to the invoker of the operation. State data updated "outside" of the operation may be maintained "inside" a stateful object or a wider stateful system within which the operation is performed.

Current milestone note:

- ordinary procedure declarations are part of `V1`
- recoverable procedure errors (`Result / Error`) are part of `V1`
- ownership-, borrowing-, and heap-move-specific calling conventions are later
  systems-language work

So this chapter describes the routine surface that exists now, while any
pointer/borrowing examples should be read as future design rather than current
compiler behavior.

Procedures can also declare a custom recoverable error type with `/` after the result type:

```
pro[] write(path: str): int / io_err = {
    report "permission denied";
}
```

The first `:` declares the result type, and `/` declares the routine error type.

Current `V1` note:

- a procedure declared as `pro[] write(...): T / E` does not produce an
  `err[E]` shell value that can be unwrapped with `!`
- it produces a recoverable routine result with a success path and an error path
- use `check(...)` or `expr || fallback` at the call site
- keep postfix `!` for `opt[...]` and `err[...]` shell values only

### Passing values

In the current `V1` compiler, procedure parameters are ordinary typed inputs.
You pass values to them exactly as you would pass values to any other routine.

The more ambitious ownership- and borrowing-specific parameter rules described
in older drafts are not part of the current `V1` procedure contract yet.

Simple example:
```
pro[] write_line(text: str): non = {
    .echo(text)
}

pro[] main(): int = {
    var message: str = "hello"
    write_line(message)
    return 0
}
```

### Ownership and borrowing

Ownership-, borrowing-, and pointer-specific procedure semantics are part of the
later `V3` systems milestone, not the current `V1` compiler surface.

Older drafts used several experimental spellings for those ideas, including:

- all-caps borrowable parameter names
- `.give_back(...)`
- double-parenthesis routine parameters

Those forms should be read as future design notes, not as current language
guarantees.

The current authoritative split is:

- ordinary procedures and functions are part of `V1`
- recoverable routine errors are part of `V1`
- ownership/borrowing calling conventions are part of `V3`

See the memory chapters and [VERSIONS.md](../../../VERSIONS.md) for the version
boundary.
