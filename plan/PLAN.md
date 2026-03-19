# FOL Strict Error Model Plan

Last updated: 2026-03-19

## Goal

Move FOL to a strict split between:

- `err[...]` as a normal first-class value type
- `T / E` as a call-site control-flow surface only

Under this model:

- `err[...]` values can be stored, passed, returned, and later handled
- routines declared as `fun[] name(): T / E` do not produce a storable value
- `var x = name()` is illegal when `name(): T / E`
- `var x: T = name()` is illegal when `name(): T / E`
- `check(name())` remains legal if `check(...)` stays in the language
- `name() || fallback` remains legal
- plain propagation through ordinary expression contexts is removed
- captured recoverable-call locals are removed
- implicit materialization of recoverable results in lowering is removed

This is a deliberate breaking change. Per project policy, the old behavior should be deleted rather than preserved behind compatibility paths.

## Language Decision

### Final semantic split

1. `err[...]` is the only error-shaped thing that is a normal value.
2. `T / E` is not a value type. It is a routine-call outcome that must be handled at the call site.
3. A `/ E` result may only appear in dedicated handling surfaces.

Initial allowed `/ E` handling surfaces:

- `expr || fallback`
- `check(expr)` if we keep it
- possibly future dedicated syntax like `try`, `catch`, `match_err`, etc.

Initial forbidden `/ E` surfaces:

- variable binding
- assignment target initialization
- record field initialization
- container elements
- function arguments to ordinary routines
- return expressions
- arithmetic/logical/comparison operands
- control-flow selectors and conditions
- method receivers
- field/index access
- shell unwrap `!`

### Consequence

This removes the current hybrid model where `/ E` can flow through expression typing and be implicitly propagated by surrounding `... / E` routines.

The new language story becomes:

- use `err[...]` when you want a real value that represents success/error state
- use `T / E` when you want immediate call-site handling with `||` or `check(...)`

## Current Compiler Behavior To Remove

The current compiler has a recoverable-effect model:

- typechecking attaches `recoverable_effect` metadata to typed expressions and typed symbols
- inferred locals may retain a recoverable effect
- plain expression contexts call `plain_value_expr(...)`, which currently permits propagation in `ErrorCallMode::Propagate`
- lowering materializes recoverable values into control flow using `CheckRecoverable`, `UnwrapRecoverable`, and `ExtractRecoverableError`

Current behaviors that must be deleted:

1. Inferred bindings can capture recoverable call results.
2. Plain expression use of `/ E` inside matching `... / E` routines can implicitly propagate.
3. `return load()` is legal because lowering converts recoverable values into report-or-unwrap control flow.
4. Lowered locals may retain `recoverable_error_type`.

## Target Architecture

### Type system

Keep both representations distinct:

- `CheckedType::Error { inner }` for `err[...]`
- `RoutineType { return_type, error_type }` for `T / E`

Do not introduce any coercion or conversion path between them.

### Typechecking

Change the meaning of a recoverable effect:

- a recoverable effect becomes evidence that an expression is only valid in dedicated handling surfaces
- it is no longer something that ordinary value contexts may propagate

Concretely:

- any ordinary value context that sees `recoverable_effect.is_some()` must error
- the old “surrounding routine declares compatible error type” rule is removed for ordinary expressions
- `ErrorCallMode::Propagate` should be removed
- the only remaining “observe” contexts should be the dedicated handlers like `||` and `check(...)`

### Lowering

Lowering should stop supporting generic materialization of recoverable values.

Instead:

- `||` lowers directly from an observed recoverable call result
- `check(...)` lowers directly from an observed recoverable call result if kept
- all ordinary expression lowering assumes no recoverable result reaches it
- generic propagation helpers should be deleted

### Runtime / IR

The recoverable ABI may still exist for routine calls, but only the dedicated handling constructs should touch it.

That means:

- `CheckRecoverable`, `UnwrapRecoverable`, and `ExtractRecoverableError` may remain as IR instructions
- generic lowering paths that synthesize propagation from arbitrary expressions should be removed
- recoverable instructions should appear only under `||`, `check(...)`, or any later explicit handling construct

## Implementation Slices

### Slice 1: Freeze the language contract in tests first `[complete]`

Before changing implementation, rewrite and add tests so the strict model is explicit.

- Add typecheck tests that reject:
  - `var x = load()` where `load(): T / E`
  - `var x: T = load()`
  - `return load()`
  - `return load() + 1`
  - `consume(load())`
  - `when(load()) { ... }`
  - `loop(load()) { ... }` if applicable
  - `load().field`
  - `load()[0]`
  - `load().method()`
- Keep tests that accept:
  - `load() || fallback`
  - `load() || report ...`
  - `load() || panic ...`
  - `check(load())` if `check(...)` survives
- Add tests that continue to accept normal `err[...]` value behavior:
  - `var x: err[int] = ...`
  - passing `err[...]` to routines
  - returning `err[...]`
  - unwrap `value!` on `err[...]`
- Delete tests that currently assert inferred recoverable locals are valid.
- Delete tests that currently assert plain propagation through routine bodies is valid.

Exit condition:

- test names and expectations describe the new model before implementation lands

### Slice 2: Replace propagation with hard rejection in typechecking `[complete]`

Centralize the strict rule in the typechecker helpers.

- Remove `ErrorCallMode::Propagate`
- Keep a single “observed recoverable expression” mode only for explicit handlers
- Replace `plain_value_expr(...)` behavior:
  - if `recoverable_effect.is_some()`, ordinary use is always an error
  - emit a dedicated message like:
    - `recoverable routine results with '/ ErrorType' cannot be used as plain values; handle them with '||' or check(...)`
- Audit every call site of `plain_value_expr(...)`
- Rename helpers if needed so the semantics are obvious:
  - for example `require_plain_value_expr(...)`
  - and `observe_recoverable_expr(...)`

Exit condition:

- no ordinary expression context can silently propagate a `/ E` result

### Slice 3: Ban binding capture completely `[complete]`

Binding capture needs an explicit direct check even after Slice 2, because bindings currently infer and store the recoverable effect.

- In binding initializer typing:
  - reject any initializer whose typed expression has `recoverable_effect.is_some()`
  - do this for both explicit and inferred bindings
- Remove code that stores `symbol.recoverable_effect` for inferred bindings
- Audit grouped/destructured bindings and record fields for the same issue
- Ensure diagnostics say the problem is the `/ E` call result itself, not a downstream type mismatch

Exit condition:

- there is no valid path where a local symbol retains a recoverable effect from a routine call

### Slice 4: Ban implicit return propagation `[complete]`

Current `return load()` works because the typechecker and lowerer treat `/ E` as materializable.

- In return typing:
  - reject `recoverable_effect.is_some()` before ordinary assignability
  - replace the old compatibility-based propagation rule with an explicit handling requirement
- Update diagnostics accordingly

Exit condition:

- `return load()` is illegal unless wrapped in a future explicit handler form

### Slice 5: Ban `/ E` in all ordinary expression surfaces `[complete]`

Do a full audit of typechecking sites that currently accept plain values.

Surfaces to cover:

- binary operators
- unary operators other than dedicated handlers
- function and method arguments
- field access
- index access
- record initializers
- container literals
- `when` selectors and branch conditions
- loop conditions and iterables
- intrinsic operands except dedicated handlers
- dot-root intrinsics

Expected result:

- every one of these surfaces rejects `/ E` values directly and consistently
- the old “requires a surrounding routine with a declared error type” diagnostics disappear from strict-mode paths

Exit condition:

- the only surviving legal `/ E` consumer surfaces are explicit handlers

### Slice 6: Simplify typed metadata `[complete]`

Once bindings and plain expressions can no longer carry recoverable results, simplify the typed model.

- Re-evaluate whether `TypedSymbol.recoverable_effect` is still needed
- Re-evaluate whether `TypedReference.recoverable_effect` is still needed for ordinary references
- Likely keep `TypedExpr.recoverable_effect` only as transient expression metadata for explicit handlers
- Remove any typed metadata that exists only to support captured recoverable locals

Preferred end state:

- recoverable metadata exists only on expression nodes/references required by `||` and `check(...)`
- local symbols do not carry recoverable-effect state

### Slice 7: Remove generic recoverable materialization from lowering `[complete]`

This is the core strictness change in lowering.

- Delete or rewrite `materialize_recoverable_value(...)`
- Delete the idea that ordinary `lower_expression_expected(...)` may convert a recoverable value into report-or-unwrap control flow
- Ensure ordinary lowering errors if a recoverable result leaks into it
- Keep direct lowering for:
  - `||`
  - `check(...)` if retained

This will likely simplify:

- `lower_expression_expected(...)`
- local binding lowering
- return lowering
- any call sites that currently choose between observed vs expected lowering based on recoverable local state

Exit condition:

- lowering no longer implements implicit propagation

### Slice 8: Remove recoverable local storage from lowered IR

The current lowered local model can store `recoverable_error_type` on locals.

Under the strict model, that should disappear for ordinary locals.

- Audit `RoutineCursor`, local allocation helpers, and verifier logic
- Remove `recoverable_error_type` from locals if no longer necessary
- If some recoverable operand metadata is still needed for direct `||`/`check(...)` lowering, keep it only as temporary expression lowering state, not as persistent general-purpose local storage
- Update verifier assumptions and tests

Preferred end state:

- routine call recoverable metadata is attached only to the specific call-result lowering path, not stored as a general local capability

### Slice 9: Keep `err[...]` as the real value-based error surface

After removing `/ E` capture and propagation, strengthen `err[...]` as the first-class error container.

Short-term work:

- keep existing `err[...]` type behavior intact
- keep postfix `!` behavior for `err[...]`
- ensure docs clearly state `err[...]` is the storable form

Follow-up design work:

- design `err[...]` methods or operators such as:
  - `.unbox()`
  - `.is_err()`
  - `.if_err(...)`
  - `.map(...)`
  - `.map_err(...)`
- decide whether those methods belong to dot-method syntax, intrinsic lowering, or library methods

This method design is separate from the strict `/ E` cleanup and should not block it.

### Slice 10: Rewrite diagnostics

Current diagnostics are phrased around the propagation model. They need to be rewritten to teach the strict model.

Messages to remove or replace:

- `requires a surrounding routine with a declared error type in V1`
- any wording that implies plain value contexts may propagate

Messages to add:

- `/ ErrorType` results are not plain values
- `/ ErrorType` results cannot be assigned to variables
- `/ ErrorType` results cannot be returned directly
- `/ ErrorType` results must be handled with `||` or `check(...)`
- `err[...]` is the value form if you need storage and later handling

Exit condition:

- diagnostics explain the language rule directly instead of exposing internal propagation machinery

### Slice 11: Rewrite docs to match the strict model

The docs currently describe propagation as part of the V1 contract. That must be removed.

Files already known to need updates:

- `book/src/650_errors/200_recover.md`
- `book/src/400_type/400_special.md`
- `book/src/500_items/200_routines/_index.md`
- `book/src/500_items/200_routines/200_functions.md`
- any examples that show `return load()` or captured recoverable locals

Required doc changes:

- remove propagation as supported behavior
- state that `/ E` is immediate handling only
- show `||` and `check(...)` examples
- state clearly that `err[...]` is the value-based alternative

Exit condition:

- docs no longer teach or imply implicit propagation

### Slice 12: Remove outdated integration fixtures and examples

Audit tests and examples for old behavior.

- Replace propagation fixtures with strict-handling fixtures
- delete or rewrite fixtures that rely on:
  - `return load()`
  - `var attempt = load()`
  - `return load() + 1`
- keep fixtures that demonstrate:
  - `||` defaulting
  - `|| report`
  - `|| panic`
  - `check(load())`
  - `err[...]` shell/value use

Exit condition:

- no fixture in the repo demonstrates the removed propagation model

## Specific Code Areas To Audit

### Typecheck

- `lang/compiler/fol-typecheck/src/exprs/helpers.rs`
- `lang/compiler/fol-typecheck/src/exprs/bindings.rs`
- `lang/compiler/fol-typecheck/src/exprs/controlflow.rs`
- `lang/compiler/fol-typecheck/src/exprs/operators.rs`
- `lang/compiler/fol-typecheck/src/exprs/calls.rs`
- `lang/compiler/fol-typecheck/src/exprs/access.rs`
- `lang/compiler/fol-typecheck/src/exprs/literals.rs`
- `lang/compiler/fol-typecheck/src/exprs/mod.rs`
- `lang/compiler/fol-typecheck/src/model.rs`

### Lowering

- `lang/compiler/fol-lower/src/exprs/expressions.rs`
- `lang/compiler/fol-lower/src/exprs/calls.rs`
- `lang/compiler/fol-lower/src/exprs/bindings.rs`
- `lang/compiler/fol-lower/src/exprs/body.rs`
- `lang/compiler/fol-lower/src/exprs/cursor.rs`
- `lang/compiler/fol-lower/src/verify/instruction.rs`
- `lang/compiler/fol-lower/src/verify/mod.rs`

### Docs and tests

- `test/typecheck/test_typecheck_foundation.rs`
- `test/typecheck/test_typecheck_error_typing.rs`
- `test/integration_tests/integration_cli_lowering.rs`
- `test/apps/fixtures/recoverable_propagation/main.fol`
- `test/apps/fixtures/recoverable_fallback/main.fol`
- `book/src/650_errors/200_recover.md`
- `book/src/400_type/400_special.md`
- `book/src/500_items/200_routines/_index.md`
- `book/src/500_items/200_routines/200_functions.md`

## Ordered Execution Plan

1. Rewrite tests to define the strict contract.
2. Change typechecker helpers so `/ E` is rejected in all ordinary value contexts.
3. Ban binding capture explicitly and remove symbol-level recoverable locals.
4. Ban direct return propagation.
5. Audit all expression surfaces for consistent rejection.
6. Simplify typed metadata.
7. Remove generic recoverable materialization from lowering.
8. Remove lowered local recoverable storage if no longer needed.
9. Rewrite diagnostics.
10. Rewrite docs and examples.
11. Run full test suite and remove any remaining propagation-era behavior.

## Acceptance Criteria

The strict model is complete when all of the following are true:

- `err[...]` remains a normal value type.
- `/ E` results cannot be assigned to locals.
- `/ E` results cannot be returned directly.
- `/ E` results cannot appear in ordinary expressions.
- only explicit handler surfaces can consume `/ E`.
- no typed local symbol stores recoverable-call state.
- lowering no longer performs implicit propagation.
- docs and tests consistently teach the strict split.

## Non-Goals

- preserving backward compatibility with propagation-based `/ E` code
- introducing migration warnings
- adding fallback behavior
- merging `/ E` and `err[...]` into one representation

## Follow-Up After This Plan

After the strict split lands, do a separate design pass for `err[...]` ergonomics:

- value methods
- library helpers
- pattern/match support
- naming of `unbox`, `unwrap`, `if_err`, `map_err`, etc.

That work should start only after the `/ E` propagation model has been fully removed.
