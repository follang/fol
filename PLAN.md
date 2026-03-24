# V1 Defer Plan

This plan is only for a narrow `V1` `defer`.

It does not include ownership-aware cleanup, borrowing-aware destruction,
pointer/resource semantics, async cleanup, or coroutine/task cleanup.

Those more complicated `defer` semantics belong to later milestones once the
language has real ownership/runtime semantics.

## Goal

Add a small, explicit `defer` statement to ordinary routine bodies.

The `V1` contract should be:

- `defer { ... }` is a statement
- it registers one body to run when the current lexical scope exits
- it runs on normal fallthrough
- it runs on `return`
- multiple defers run in reverse registration order
- it is purely scope/control-flow sugar in `V1`

The `V1` contract should explicitly exclude:

- ownership-driven destruction semantics
- pointer/resource finalization semantics
- async/task/channel cleanup semantics
- error-only variants such as `errdefer`
- expression-form `defer`

## Slice Tracker

- [x] Slice 1: rewrite milestone docs so narrow `V1` `defer` is allowed and ownership-heavy `defer` is deferred to `V3`/`V4`
- [x] Slice 2: add book chapter text for narrow `V1` `defer`
- [x] Slice 3: add lexer/parser/AST support for `defer { ... };`
- [x] Slice 4: add body/typecheck validation for `defer`
- [x] Slice 5: add lowering support so defers execute on scope exit and early return
- [x] Slice 6: harden nested-scope lowering behavior and reverse-order execution
- [x] Slice 7: add parser tests
- [x] Slice 8: add typecheck and lowering tests
- [x] Slice 9: add runnable app/example tests
- [x] Slice 10: run `make build` and `make test`, then update this file if any semantics need tightening

## Design

### Syntax

`V1` syntax:

```fol
defer {
    .echo("cleanup");
};
```

Only block-body form belongs in `V1`.

### Semantics

Inside one scope:

```fol
defer { .echo(1); }
defer { .echo(2); }
```

must execute as:

- `2`
- then `1`

If a scope returns early:

```fol
defer { .echo("closing"); }
return 7;
```

the deferred body must run before control leaves that scope.

Nested scopes should only run their own deferred bodies when that specific scope
exits.

### Out Of Scope

Do not add in this plan:

- `defer expr`
- `errdefer`
- `defer` tied to ownership moves or borrow invalidation
- `defer` attached to build/runtime/native resource models
- concurrency-aware cleanup rules

## Likely Files

- `lang/compiler/fol-lexer/src/token/buildin/mod.rs`
- `lang/compiler/fol-lexer/src/lexer/stage1/element.rs`
- `lang/compiler/fol-parser/src/ast/node.rs`
- `lang/compiler/fol-parser/src/ast/parser_parts/type_references_and_blocks.rs`
- `lang/compiler/fol-parser/src/ast/parser_parts/flow_body_parsers.rs`
- `lang/compiler/fol-parser/src/ast/parser_parts/program_parsing.rs`
- `lang/compiler/fol-typecheck`
- `lang/compiler/fol-lower/src/exprs/body.rs`
- `book/src/...`
- `plan/VERSIONS.md`

## Exit Criteria

- the docs clearly say narrow `V1` `defer` is supported
- the docs clearly say ownership-heavy/resource-heavy `defer` belongs later
- parser accepts `defer { ... }`
- lowering executes deferred blocks in reverse order
- deferred blocks run before `return` leaves the current scope
- tests cover nested scopes and multiple defers
