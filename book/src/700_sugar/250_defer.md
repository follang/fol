# Defer

`defer` is scope-exit cleanup sugar.

In current `V1`, it is intentionally narrow:

- only `defer { ... }` block form is supported
- it registers work to run when the current lexical scope exits
- it runs on normal fallthrough
- it runs before `return` leaves the current scope
- multiple deferred blocks run in reverse registration order

This is control-flow sugar.

It is not an ownership system, destructor system, object system, or async
cleanup system.

Model reminder:

- the `.echo(...)` examples below assume `fol_model = "memo"`
- `defer` itself is not std-only; it is valid in `core`, `memo`, and `std`
  within the current V1 surface

## Basic form

```fol
pro[] main(): int = {
    defer {
        .echo("closing");
    };

    .echo("work");
    return 7;
}
```

The deferred body runs before control leaves `main`.

## Reverse order

```fol
pro[] main(): non = {
    defer { .echo(1); };
    defer { .echo(2); };
}
```

This executes as:

- `2`
- then `1`

## Nested scopes

Deferred bodies belong to the scope where they are declared:

```fol
pro[] main(flag: bol): non = {
    defer { .echo("outer"); };

    when(flag) {
        case(true) {
            defer { .echo("inner"); };
            return;
        }
        * { }
    }
}
```

If the inner scope exits first, its deferred bodies run before the outer
scope's deferred bodies.

## Current milestone boundary

The narrow scope-exit `defer` described above belongs in `V1`.

More complicated `defer` behavior belongs later:

- ownership-aware cleanup
- borrowing/pointer/resource cleanup
- async/task/channel cleanup
- native/foreign resource cleanup
- error-only variants such as `errdefer`

Those later forms should be treated as `V3`/`V4` work, not as part of the
current `V1` contract.
