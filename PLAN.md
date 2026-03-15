# FOL V1 Error Handling Plan

Last updated: 2026-03-15

This file defines the next compiler milestone: make recoverable errors a real
`V1` feature through the full compiler chain instead of leaving them half-real
between type checking and backend work.

The active compiler already has:

- routine signatures with `ResultType / ErrorType`
- `report expr`
- typed `opt[...]` / `err[...]` shells
- postfix unwrap `value!` for shell values
- lowered routine signatures that preserve `error_type`
- lowered `Report` terminators

But the current compiler still lacks one coherent story for **calling** an
errorful routine and then deciding whether to:

- propagate the error
- branch on the error
- recover with a default
- panic on failure
- unwrap and continue

That gap must be closed before backend work, because backend calling convention
and runtime behavior depend on it.

## 0. Why This Plan Is Needed

Recoverable errors are already important in the book and in the implemented
front-end surface.

Current repository truth:

- parser already preserves routine `error_type`
- typechecker already enforces `report`
- lowering already emits `Report`
- shell syntax like `err[str]`, `err[]`, `opt[str]`, and postfix `!` already has
  current `V1` typing and lowering support

Current missing piece:

- call sites of routines with declared error types do not yet have a completed
  `V1` semantic model
- builtins like `check(...)` are still book-facing ideas, not a fully owned
  semantic feature
- the compiler does not yet define the backend-facing representation of
  “successful result + possible error” for routine calls

That means `report` is partially real today, but **recoverable error handling as
a user workflow is not complete yet**.

## 1. Local Source Basis For This Plan

This plan is based on the current repository state and current book text:

- [`book/src/650_errors/200_recover.md`](./book/src/650_errors/200_recover.md)
- [`book/src/700_sugar/200_pipes.md`](./book/src/700_sugar/200_pipes.md)
- [`book/src/700_sugar/800_chaining.md`](./book/src/700_sugar/800_chaining.md)
- [`book/src/500_items/200_routines/100_procedures.md`](./book/src/500_items/200_routines/100_procedures.md)
- [`book/src/500_items/200_routines/200_functions.md`](./book/src/500_items/200_routines/200_functions.md)
- [`fol-typecheck/src/exprs.rs`](./fol-typecheck/src/exprs.rs)
- [`fol-lower/src/exprs.rs`](./fol-lower/src/exprs.rs)
- [`VERSIONS.md`](./VERSIONS.md)

## 2. Zig Reference And What To Borrow

Official Zig 0.15.2 error handling is worth studying because it separates the
same core decisions FOL now has to settle:

- error unions carry either a success value or an error
- `try` propagates failure
- `catch` handles failure with a fallback or custom branch
- `if (err_union) |value| { ... } else |err| { ... }` branches on success vs
  error while capturing the chosen payload
- `catch unreachable` or similar forms act like forceful unwraps
- `errdefer` handles cleanup on failure paths

FOL should borrow the **semantic split**, not blindly copy Zig syntax.

For `V1`, the recommended approach is:

- keep FOL’s existing declared routine form `: ResultType / ErrorType`
- keep `report`
- keep shell unwrap `!`
- complete the existing book-facing `check(...)` and `||` handling surfaces
- defer cleanup constructs like Zig `errdefer` to a later milestone

## 3. Recommended V1 Error Contract

### 3.1 Routine Declarations

Keep the existing form:

```fol
fun[] read(path: str): str : io_err = { ... }
```

Meaning:

- success path yields `str`
- failure path yields `io_err`
- `report expr` exits through the failure path

### 3.2 Call-Site Semantics

For `V1`, calls to routines with declared `error_type` must no longer be treated
as ordinary plain values internally.

Instead, they must carry a **recoverable call-result effect** until one of the
supported consumers handles them.

That effect does not need to be a user-spellable type in `V1`, but it does need
to be explicit in:

- typecheck
- lowered IR
- backend contract

### 3.3 Supported V1 Consumers

The plan should make these call-result consumers real:

1. **Propagation**
- keep the book rule that an unhandled errorful call propagates upward in an
  error-aware routine context
- if the surrounding routine cannot carry that error, typecheck must reject it

2. **Explicit check**
- make `check(expr)` a real builtin/intrinsic over errorful routine results
- `check(expr)` returns `bol`
- its contract is “does this routine result currently hold an error?”

3. **Fallback / handler shorthand**
- make `expr || fallback` a real error-handling surface
- if `expr` succeeds, use the success value
- if `expr` fails, evaluate `fallback`
- `fallback` may:
  - return a default success value
  - `panic`
  - `report`
  - possibly branch further

4. **Force unwrap**
- settle a force-unwrap story for errorful routine results
- recommended `V1` decision:
  - keep postfix `!` only for shell values already typed as `opt[...]` or
    `err[...]`
  - do **not** silently extend `call!` to routine results until the result/error
    effect model is explicit and stable
- if a force-unwrap-on-call is wanted later, add it as a deliberate syntax
  decision instead of smuggling it through current shell rules

### 3.4 Branching Form

For `V1`, do **not** invent a new Zig-like capture syntax unless the current book
surfaces prove impossible to lower cleanly.

Recommended `V1` branch story:

```fol
var file = open(path)
if (check(file)) {
    report "could not open"
} else {
    return file | stringify(this)
}
```

This is not perfect ergonomically, but it stays within the current language
surface and keeps the milestone bounded.

Direct success/error capture syntax inspired by Zig can be reconsidered for a
later milestone if the current surface becomes too awkward.

## 4. Hard Definition Of Done

This plan is complete only when all of the following are true:

- routines with declared error types behave as real error-aware calls through the
  full chain
- `report` and ordinary returns coexist under one coherent call-result model
- `check(expr)` is typechecked, lowered, and backend-owned
- `expr || fallback` is typechecked, lowered, and backend-owned
- unhandled errorful calls either:
  - propagate in a valid error-aware routine context, or
  - are rejected explicitly elsewhere
- shell `err[...]` and routine error calls have a clear non-conflicting boundary
- backend-facing lowered IR has one stable calling convention for recoverable
  errors
- CLI integration tests prove successful propagation, handled recovery, and
  rejected misuse
- book/docs no longer claim stale parser-only behavior

## 5. Current Known Gaps

### 5.1 Stale Documentation

The recoverable-error chapter still says the parser does not type-check
`report`, which is no longer true.

### 5.2 `check(...)` Is Not Implemented As A Real Semantic Feature

The book treats `check(...)` as a builtin way to inspect recoverable routine
failures, but current typecheck/lower logic only gives special treatment to:

- `panic`
- `report`

### 5.3 Errorful Calls Do Not Yet Have A Stable Effect Model

Current lowered calls are still plain `Call { ... }` instructions. The lowered
IR retains routine `error_type`, but it does not yet define what a caller
receives and how success/error branches are represented after the call.

### 5.4 Propagation Rules Are Not Yet End-To-End Real

The book talks about errors concatenating/propagating upward, but the backend
contract for that does not exist yet.

## 6. Execution Strategy

Do this in dependency order:

1. settle the `V1` semantic contract
2. model it explicitly in typecheck
3. model it explicitly in lowered IR
4. define backend-facing calling convention
5. only then sync docs/book

Do **not** start with syntax expansion.
Do **not** add new sugar before the current book surfaces are either implemented
or deliberately rejected.

## 7. Implementation Slices

### Phase 0. Contract Freeze

- `0.1` `done` Audit every current parser/typecheck/lower surface that mentions:
  - `report`
  - `panic`
  - `check`
  - `err[...]`
  - `opt[...]`
  - `||`
  Audit result:
  - parser already preserves `/` error signatures and `check(...)` syntax
  - typecheck already enforces `report` and `panic`
  - lowering already preserves routine `error_type` and emits `Report`
  - `check(...)` and `||` still need real semantic ownership
- `0.2` `done` Freeze the `V1` contract for errorful routine calls:
  - what counts as propagation
  - what counts as handled recovery
  - where plain use is illegal
  Current state:
  - plain use propagates only in routines with compatible declared error types
  - `check(expr)` and `expr || fallback` are the handled `V1` consumers for
    observing or recovering from routine-call failures
  - plain use outside those consumers is rejected explicitly
- `0.3` `done` Freeze the explicit `V1` boundary:
  - `check(...)` and `||` are in
  - Zig-like `if |value| else |err|` capture syntax is out for now
  - `errdefer`-style cleanup is out for now
  Current state:
  - the implemented `V1` path now commits to `check(...)` and `||`
  - no capture-syntax or cleanup expansion has been added to this milestone

### Phase 1. Typecheck Error-Call Model

- `1.1` `done` Introduce a typecheck-owned representation for errorful routine
  call results so they are no longer treated as plain values too early.
  Current state:
  - typed references and inferred bindings can retain a recoverable error effect
  - expression typing now carries that effect internally instead of flattening it immediately
- `1.2` `done` Make ordinary use of an errorful call illegal unless the
  surrounding context is one of the approved `V1` consumers.
  Current state:
  - plain value contexts now reject errorful expressions unless they are consumed
    by an approved `V1` handler path
  - exact tests lock plain assignment and arithmetic misuse in routines that
    cannot propagate
- `1.3` `done` Implement propagation typing:
  - allow it only in routines with compatible declared error types
  - reject it in routines with no error type
  - reject incompatible error payload propagation
  Current state:
  - matching routine error types can now carry plain-value propagation through
    expression typing
  - routines with no declared error type reject propagation
  - incompatible propagated payloads fail with explicit `IncompatibleType`
    diagnostics
- `1.4` `done` Implement `check(expr)` typing over errorful routine results.
  Current state:
  - `check(expr)` now observes an errorful expression and returns `bol`
  - it rejects plain non-errorful values explicitly
- `1.5` `done` Implement `expr || fallback` typing:
  - success branch type
  - fallback compatibility
  - `panic` / `report` / `return` fallback handling
  Current state:
  - `expr || fallback` now consumes the left-side recoverable call effect
  - fallback values must match the left success type unless they early-exit
  - `report` / `return` / `panic` style never-path fallbacks remain valid
- `1.6` `done` Lock exact diagnostics for:
  - errorful call used as plain value
  - propagation in routines with no error type
  - incompatible propagated error payloads
  - invalid `check(...)`
  - invalid `||` fallback types
  Current state:
  - exact tests now cover each of those misuse classes

### Phase 2. Lowered IR Error Model

- `2.1` `done` Extend lowered IR so errorful routine calls are represented
  explicitly instead of looking identical to plain calls.
  Current state:
  - lowered `Call` instructions now retain an optional `error_type`
  - ordinary and errorful routine calls no longer collapse to one indistinguishable
    backend-facing shape
- `2.2` `done` Add lowered instructions or terminators for:
  - checked call
  - success/error branch after call
  - handled fallback path
  - upward propagation path
  Current state:
  - lowered recoverable values now use explicit `CheckRecoverable`,
    `UnwrapRecoverable`, and `ExtractRecoverableError` instructions
  - propagation and `||` handling now lower through real control-flow branches
    instead of pretending errorful calls are plain values
- `2.3` `done` Keep routine signatures carrying both return and error types in
  a way the backend can consume directly.
  Current state:
  - lowered routine signatures still retain `return_type` plus `error_type`
  - lowered call instructions now mirror that error metadata at call sites so
    later backend work can consume both sides of the contract directly
- `2.4` `done` Extend the lowering verifier so impossible error-flow shapes are
  rejected explicitly.
  Current state:
  - verifier now checks recoverable-error metadata on locals/globals and validates
    recoverable-only instructions plus `panic` terminators
- `2.5` `done` Add exact lowered snapshot tests for:
  - success call
  - propagated call
  - `check(...)` branch
  - `||` default
  - `|| report ...`
  - `|| panic ...`
  Current state:
  - lowering tests now lock propagation, `check(...)`, default fallback,
    `report` fallback, and `panic` fallback shapes directly

### Phase 3. Backend Calling Convention Contract

- `3.1` `pending` Freeze the first backend-facing ABI for error-aware routines:
  - one return slot plus one error slot
  - tagged success/error result object
  - or another concrete representation
- `3.2` `pending` Ensure the chosen representation works for:
  - same-package calls
  - imported `loc/std/pkg` calls
  - nested routine calls
  - calls inside `when` / loops / returns
- `3.3` `pending` Document the runtime meaning of:
  - success
  - failure
  - propagated failure
  - forced failure via `panic`

### Phase 4. V1 User-Facing Handling Paths

- `4.1` `done` Add end-to-end success tests for ordinary propagation through
  multiple routines with matching error types.
  Current state:
  - CLI tests now compile multi-routine propagation fixtures and assert lowered
    propagation control flow is present
- `4.2` `done` Add end-to-end success tests for `check(expr)` plus `if/else`
  handling.
  Current state:
  - CLI tests now compile handled `check(...)` fixtures and assert explicit
    recoverable checks survive into lowered output
- `4.3` `done` Add end-to-end success tests for `expr || default_value`.
  Current state:
  - CLI tests now compile default-value fallback fixtures and assert the lowered
    default branch stays explicit
- `4.4` `done` Add end-to-end success tests for `expr || report ...`.
  Current state:
  - CLI tests now compile report-fallback fixtures and assert the error branch
    lowers to `Report`
- `4.5` `done` Add end-to-end success tests for `expr || panic ...`.
  Current state:
  - CLI tests now compile panic-fallback fixtures and assert the error branch
    lowers to `Panic`
- `4.6` `done` Add negative tests for:
  - trying to assign an errorful call directly to a plain value in a routine that
    cannot propagate
  - using `check(...)` on a plain non-errorful value
  - incompatible fallback value types
  - incompatible propagated error types
  Current state:
  - typecheck unit tests and CLI integration tests now cover all four failure
    classes with exact diagnostic assertions

### Phase 5. Shell Alignment

- `5.1` `pending` Reconcile routine error calls with existing `err[...]` shell
  values so users and compiler do not confuse them.
- `5.2` `pending` Decide whether explicit conversion between:
  - routine error call results
  - `err[...]` shell values
  belongs in `V1` or later.
- `5.3` `pending` Lock `V1` postfix unwrap behavior so it stays limited to shell
  values unless a deliberate new call-unwrap surface is introduced.

### Phase 6. Docs And Book Sync

- `6.1` `pending` Update [`README.md`](./README.md) and [`PROGRESS.md`](./PROGRESS.md)
  only after the error-call model is truly implemented.
- `6.2` `pending` Rewrite [`book/src/650_errors/200_recover.md`](./book/src/650_errors/200_recover.md)
  so it no longer describes parser-only behavior and instead explains the real
  current `V1` handling story.
- `6.3` `pending` Sync [`book/src/700_sugar/200_pipes.md`](./book/src/700_sugar/200_pipes.md)
  with the actual implemented meaning of `check(...)` and `||`.
- `6.4` `pending` Rewrite this file into a completion record only after the full
  `V1` error path is real through the compiler chain.

## 8. What Should Happen After This Plan

Only after this plan is complete should the project treat recoverable errors as
fully real `V1` behavior and proceed into backend implementation without a
semantic hole in one of the most important language features.
