# V1 Niceties Plan

This plan is for `V1` only.

It explicitly excludes `V2`, `V3`, and `V4` work.

It also treats FOL as a procedural language with data types and routine sugar,
not as an object-oriented language. Any book text that implies classes,
inheritance, object-owned methods, or OOP runtime dispatch should be corrected
or deleted as part of Phase 1.

## Scope

The target V1 niceties to complete are:

- record methods as procedural receiver sugar
- named call arguments
- default parameters
- variadic routine calls
- call-site unpack for variadic calls
- related call-binding diagnostics, lowering, backend, docs, and tests

Secondary follow-up niceties, only after the above are stable:

- method calls using named/default/variadic arguments
- anonymous routine capture support if still desired for V1
- inquiry execution if we decide inquiries are part of real V1 behavior instead of parser-only sugar

## Ground Rules

- No OOP framing in docs or implementation.
- No inheritance wording.
- No class/object ownership model.
- Methods remain sugar for receiver-qualified routines.
- Records remain data.
- Routines remain separate from data declarations.
- Do not preserve conflicting legacy wording if it suggests otherwise.

## Slice Tracker

- [x] Slice 1: rewrite `book/src/500_items/200_routines/300_methods.md` to make the procedural receiver-sugar model explicit and remove conflicting wording
- [x] Slice 2: rewrite `book/src/500_items/300_constructs/200_structs.md` so records are documented strictly as data plus receiver-qualified routines
- [x] Slice 3: rewrite `book/src/500_items/300_constructs/100_aliases.md` so alias/extension examples stay procedural and non-OOP
- [x] Slice 4: clean `book/src/200_expressions/300_exp/400_access.md`, `book/src/500_items/400_standards.md`, and `book/src/500_items/500_generics.md` of conflicting OOP/dispatch wording
- [x] Slice 5: harden record-method semantic coverage for local receiver-qualified routines
- [x] Slice 6: harden record-method semantic coverage for imported and qualified receiver-qualified routines plus diagnostics
- [x] Slice 7: implement named arguments for free calls
- [ ] Slice 8: implement named arguments for method calls
- [ ] Slice 9: implement default parameters for free calls
- [ ] Slice 10: implement default parameters for method calls
- [ ] Slice 11: implement variadic calls and call-site unpack for free calls
- [ ] Slice 12: implement variadic calls and call-site unpack for method calls

## Phase 1: Correct The Book

Goal:

- make the documentation consistently procedural and non-OOP
- remove misleading language before adding more niceties

Work:

- rewrite method docs so they say:
  - methods are receiver-qualified routines
  - `value.method(x)` is sugar for `method(value, x)`
  - records are data, not classes
  - no inheritance, object-owned methods, or OOP dispatch
- clean any chapters that still imply:
  - classes
  - inheritance
  - object-method ownership
  - method dispatch as an OOP system
- audit routine, record, alias, expression-access, and standards chapters for conflicting wording
- if a chapter discusses later-milestone design in a way that conflicts with current V1, add explicit version notes or remove the conflicting text

Likely files:

- `book/src/500_items/200_routines/300_methods.md`
- `book/src/500_items/300_constructs/200_structs.md`
- `book/src/500_items/300_constructs/100_aliases.md`
- `book/src/200_expressions/300_exp/400_access.md`
- `book/src/500_items/400_standards.md`
- any other book pages found by the wording audit

Exit criteria:

- the book consistently describes methods as procedural sugar
- there is no remaining OOP/inheritance wording in active V1 chapters

## Phase 2: Harden Record Methods In V1

Status today:

- record methods already exist end-to-end in parser, typecheck, and lowering
- this phase is for hardening, clarifying, and closing gaps, not inventing OOP features

Goal:

- make record methods a clearly supported, tested V1 feature

Work:

- audit parser/typecheck/lowering/backend behavior for receiver-qualified routines on records
- verify method resolution for:
  - local record methods
  - imported record methods
  - qualified imported record methods
  - overloaded methods by receiver type
- verify diagnostics for:
  - missing method
  - ambiguous method
  - wrong arity
  - wrong argument type
  - invalid receiver type
- verify docs and tests use record methods as procedural sugar
- decide whether non-record receivers stay in V1 or whether record-focused method support should be the canonical documented surface

Likely code areas:

- `lang/compiler/fol-parser`
- `lang/compiler/fol-resolver`
- `lang/compiler/fol-typecheck/src/exprs/calls.rs`
- `lang/compiler/fol-lower/src/exprs/expressions.rs`
- `lang/compiler/fol-lower/src/exprs/calls.rs`
- `lang/execution/fol-backend`

Likely tests:

- `test/parser/test_parser_parts/method_receivers_and_branching.rs`
- `test/typecheck/test_typecheck_foundation.rs`
- `test/typecheck/test_typecheck_workspace_imports.rs`

Exit criteria:

- record methods are documented as first-class V1 sugar
- semantic and lowering coverage for record methods is solid
- no docs suggest record methods are OOP features

## Phase 3: Implement Named Call Arguments

Status today:

- parser preserves named arguments
- semantic call binding is still positional

Goal:

- make named arguments real for function calls, method calls, and invoke calls in V1

Work:

- define the canonical V1 binding rule:
  - positional arguments bind first
  - once a named argument appears, all remaining arguments must be named
  - duplicate named arguments are errors
  - unknown parameter names are errors
- add semantic reordering from call arguments to declared parameters
- update method-call binding too, while preserving receiver as the first implicit bound input
- update lowering so named arguments are consumed at call sites and emitted in canonical parameter order
- update diagnostics for:
  - unknown name
  - duplicate named arg
  - missing required arg
  - positional after named

Likely code areas:

- parser tests already exist; semantic work is mainly:
- `lang/compiler/fol-typecheck/src/exprs/calls.rs`
- `lang/compiler/fol-lower/src/exprs/calls.rs`
- `lang/compiler/fol-lower/src/exprs/expressions.rs`

Exit criteria:

- named arguments work for ordinary calls and method calls
- lowering no longer treats `NamedArgument` as an unconsumed structural leftover

## Phase 4: Implement Default Parameters

Status today:

- parser preserves parameter default expressions
- call checking still requires exact arity

Goal:

- allow omitted trailing/defaultable parameters in V1 calls

Work:

- define canonical V1 default semantics:
  - omitted parameters use declaration defaults
  - required parameters must still be supplied
  - named args can skip to later defaulted parameters
  - defaults are evaluated in the routine declaration environment model chosen for V1
- decide and document whether defaults are:
  - lowered at call sites
  - or materialized in routine wrappers
- update typecheck arity rules accordingly
- update lowering to synthesize omitted default args
- verify method calls also support defaults
- add diagnostics for:
  - missing non-defaulted arg
  - duplicate/default conflict cases

Likely code areas:

- `lang/compiler/fol-typecheck/src/exprs/calls.rs`
- `lang/compiler/fol-lower/src/exprs/calls.rs`
- `lang/compiler/fol-lower/src/exprs/expressions.rs`
- possibly declaration metadata plumbing if defaults need explicit lowering support

Exit criteria:

- default parameters are real V1 behavior, not parser-only syntax

## Phase 5: Implement Variadic Calls

Status today:

- parser lowers variadic parameter syntax into sequence-like parameter types
- runtime call behavior is not yet implemented as true variadic binding

Goal:

- make variadic routine parameters callable in V1

Work:

- define canonical V1 variadic semantics:
  - final variadic parameter collects trailing actual args
  - explicit sequence passing vs packed trailing args must be decided
  - interaction with named args must be specified
- update typecheck to:
  - accept extra trailing args for final variadic parameter
  - typecheck each collected arg against the variadic element type
- update lowering to:
  - pack trailing args into the expected sequence representation
  - preserve method-call receiver handling correctly
- verify arity/type diagnostics for variadics

Likely code areas:

- `lang/compiler/fol-typecheck/src/exprs/calls.rs`
- `lang/compiler/fol-lower/src/exprs/calls.rs`
- container/runtime support if needed for packed sequence materialization

Exit criteria:

- variadic calls work end-to-end for `fun`, `pro`, and methods where applicable

## Phase 6: Implement Call-Site Unpack

Status today:

- parser preserves `...arg`
- semantics/lowering do not consume it yet

Goal:

- support unpack at call sites as the companion feature to variadics

Work:

- define canonical V1 unpack semantics:
  - only valid in call argument position
  - only valid where the callee can accept expanded arguments
  - interaction with named arguments must be specified
- update typecheck to validate unpack operand type
- update lowering to expand unpack operands into the canonical variadic packing path
- support unpack for:
  - free calls
  - method calls
  - invoke calls if invoke remains in V1 scope

Exit criteria:

- unpack is real call behavior, not parser-only syntax

## Phase 7: Unify Call Binding

Goal:

- avoid three half-implemented systems for positional, named, defaulted, and variadic calls

Work:

- build one shared call-binding layer used by:
  - free calls
  - qualified calls
  - method calls
  - invoke calls where applicable
- represent a canonical bound-argument plan before lowering
- reuse that plan for diagnostics and lowering

Why this phase matters:

- named args, defaults, variadics, and unpack are one feature family
- implementing them separately will create drift and regressions

## Phase 8: Tests And Book Updates For Each Feature

For every feature phase above:

- update parser tests if syntax changes
- add resolver tests if symbol/reference behavior changes
- add typecheck tests for success and failure cases
- add lowering tests
- add frontend/integration tests where behavior becomes user-visible
- update book examples to prefer the new canonical V1 forms

Important rule:

- no feature is complete until docs and semantic tests match

## Phase 9: Follow-Up Niceties After Core Call Binding

Only after Phases 1-8 are stable:

- method calls with named/default/variadic arguments in all supported forms
- anonymous routine captures if we still want them in V1
- inquiry clauses as real executable V1 behavior, or else explicitly mark them as non-runtime/parser-only until implemented

## Explicit Non-Goals

Do not mix this plan with:

- standards/contracts/extensions
- generics
- async/await/spawn/channels/select
- rolling/comprehensions
- optional chaining
- pointer/borrowing systems work
- V2, V3, or V4 milestones

## Recommended Execution Order

1. Fix the book and remove conflicting OOP wording.
2. Harden and clarify record methods as the canonical procedural method model.
3. Build one shared call-binding design.
4. Implement named arguments.
5. Implement default parameters.
6. Implement variadic calls.
7. Implement call-site unpack.
8. Expand method-call coverage for the full call-binding system.
9. Revisit captures and inquiries only after the above is complete.

## Done Means

This plan is done only when:

- the book is consistent
- record methods are clearly procedural sugar
- named/default/variadic/unpack calls work end-to-end
- diagnostics are clear
- lowering/backend paths are covered
- no conflicting OOP framing remains in active V1 docs
