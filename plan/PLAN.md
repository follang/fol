# FOL V1 Hardening Plan

Last updated: 2026-03-21

## Goal

Take the entire V1 language from "compiles basic programs" to "every V1 feature
parses, typechecks, lowers, generates code, and is tested end-to-end." No stubs,
no silent fallthrough, no panic-on-valid-input, no half-implemented pipelines.

This plan is based on a full codebase scan across all 15 crates covering:

- parser AST → resolver → typecheck → lower → backend pipeline completeness
- lexer correctness bugs
- backend code generation gaps
- panic/unwrap/unreachable safety
- test coverage gaps
- tooling robustness

## Severity Legend

- **P0**: Blocks correct compilation of V1 programs
- **P1**: Causes crashes or incorrect behavior on valid V1 input
- **P2**: Missing coverage, quality gaps, or deferred V1 surface
- **P3**: Cleanup, consistency, test quality

---

## Phase 1: Critical Pipeline Gaps (P0)

These are constructs that parse (and sometimes typecheck) but **fail to lower
or generate code**. They are the most critical blockers for a working V1.

### 1.1 Binary Operator Lowering — DONE

**Work**:
- [x] implement lowering for arithmetic operators (Add, Sub, Mul, Div, Mod, Pow)
- [x] implement lowering for comparison operators (Eq, Ne, Lt, Le, Gt, Ge)
- [x] implement lowering for logical operators (And, Or, Xor, Not via unary)
- [x] implement backend emission for each new lowered binary instruction
- [x] add end-to-end tests for each operator family
- [ ] implement lowering for membership operators (In, Has) — deferred, rejected at typecheck
- [ ] implement lowering for type operators (Is, As, Cast) — deferred, rejected at typecheck
- [ ] implement lowering for Pipe operator — deferred, rejected at typecheck

### 1.2 Unary Operator Lowering — DONE

**Work**:
- [x] implement lowering for Neg (numeric negation)
- [x] implement lowering for Not (boolean negation)
- [x] implement backend emission for each
- [x] add end-to-end tests
- [ ] implement lowering for Ref (reference taking) — V3 systems milestone
- [ ] implement lowering for Deref (dereference) — V3 systems milestone

### 1.3 Invoke Expression Pipeline

**Problem**: `AstNode::Invoke` (general function invocation) parses but
needs typecheck and lowering support. Direct function invocation is broken.

**Work**:
- [ ] implement typecheck for Invoke expressions
- [ ] implement lowering for Invoke expressions
- [ ] implement backend emission for invocations
- [ ] add tests for direct calls, chained calls, calls with error returns

### 1.4 Anonymous Routine Pipeline

**Problem**: `AnonymousFun`, `AnonymousPro`, `AnonymousLog` have full typecheck
support but zero lowering support. Closures/lambdas parse and typecheck
successfully, then fail during code generation.

**Work**:
- [ ] implement lowering for AnonymousFun
- [ ] implement lowering for AnonymousPro
- [ ] implement lowering for AnonymousLog
- [ ] implement backend emission for anonymous routines (Rust closures)
- [ ] add tests for lambda capture, passing as arguments, returning

### 1.5 Cast Instruction Backend — DONE

**Work**:
- [x] implement Cast instruction rendering in backend
- [x] verify cast policy matches V1 type system

---

## Phase 2: Critical Compiler Bugs (P1)

These are bugs that cause incorrect behavior or crashes on valid input.

### 2.1 Lexer: Block Comment — FALSE POSITIVE

**Status**: Not a bug. `bump()` advances AND pushes the new current char
into the token content, so one `bump()` after the `*/` loop correctly
consumes both `*` and `/`. Verified with test.

- [x] add lexer test for block comments followed by code

### 2.2 Lexer: Out-of-Bounds Array Access in Stages 2-3 — DONE

**Work**:
- [x] fix bounds to cap at `SLIDER - 1` instead of `SLIDER`
- [x] remove dead code from all lexer stages

### 2.3 Frontend Dispatch: False Unreachable Claims — DONE

**Work**:
- [x] refactor dispatch to destructure command in match arms directly
- [x] eliminate false unreachable! calls

### 2.4 Backend: Global Mutation Panic Risk — DONE

**Work**:
- [x] replace `.expect("global lock")` with `.unwrap_or_else(|e| e.into_inner())`
      in both render.rs and helpers.rs

### 2.5 Backend: Panic Terminator Format String — FALSE POSITIVE

**Status**: Not a bug. `"{{}}"` inside `format!()` correctly produces `"{}"` in
the generated Rust code. The double-brace escaping is the correct way to emit
format strings via `format!()`. Confirmed by existing test.

---

## Phase 3: High-Priority Pipeline Gaps (P2)

Features that parse but lack semantic or codegen support, blocking important
V1 use cases.

### 3.1 TemplateCall Expression

**Work**:
- [ ] implement typecheck for TemplateCall
- [ ] implement lowering for TemplateCall
- [ ] implement backend emission
- [ ] add end-to-end tests

### 3.2 AvailabilityAccess Expression

**Work**:
- [ ] implement typecheck for AvailabilityAccess (should return bool)
- [ ] implement lowering for AvailabilityAccess
- [ ] implement backend emission (Rust `.is_some()` equivalent)
- [ ] add tests with opt values

### 3.3 SliceAccess Expression

**Work**:
- [ ] implement lowering for SliceAccess
- [ ] implement backend emission for slicing
- [ ] add tests for vec/seq slicing

### 3.4 Backend: Unsized Array Type Rendering

**Work**:
- [ ] implement unsized array rendering (Rust `Vec<T>` or `&[T]`)
- [ ] or ensure lowering never produces unsized arrays for V1 and add
  a lowering-phase rejection with proper diagnostic

### 3.5 Backend: Heterogeneous Set Rendering

**Work**:
- [ ] implement heterogeneous set rendering
- [ ] or reject at typecheck level with proper diagnostic if V1 doesn't
  support heterogeneous sets

### 3.6 Backend: Unhandled Type Variants

**Work**:
- [ ] audit all LoweredType variants
- [ ] implement rendering for each V1-admitted variant
- [ ] convert catch-all to exhaustive match

### 3.7 Entry Variant Construction

**Work**:
- [ ] implement entry variant construction lowering
- [ ] implement backend emission
- [ ] add tests for entry creation and field access

### 3.8 Iteration Loops (when/loop lowering)

**Work**:
- [ ] implement loop lowering
- [ ] implement backend loop emission
- [ ] add tests for counted, conditional, and collection loops

---

## Phase 4: Panic/Crash Hardening (P1-P2)

Replace all panic paths in non-test code with proper error propagation.

### 4.1 Intrinsics Catalog Panics — JUSTIFIED INVARIANTS

**Status**: The 7 `panic!` calls in fol-intrinsics assert catalog
consistency (e.g., "intrinsic X must exist in the catalog"). These are
compile-time invariants, not runtime user-input failures. Converting to
Result would add error propagation overhead through the entire intrinsic
lookup chain without practical benefit.

### 4.2 Lower Session Package Panic — JUSTIFIED INVARIANT

**Status**: The `panic!` at line 820 asserts that a package that was
already resolved and type-checked must still be present during lowering.
This is an internal invariant violation, not a user-input failure.

### 4.3 Parser Unreachable Calls — DONE

**Work**:
- [x] add descriptive messages to all unreachable! calls

### 4.4 Typecheck Unreachable Calls — JUSTIFIED INVARIANTS

**Status**: The `unreachable!` calls in literals.rs and operators.rs
are true invariants documenting that prior phases guarantee certain
conditions. They already have descriptive messages.

### 4.5 Backend Unreachable Calls — JUSTIFIED INVARIANT

**Status**: The `unreachable!` in emit/build.rs (line 154) correctly
asserts that `emit_generated_crate_skeleton()` always returns
`RustSourceCrate`. The function only creates source files, making
this a true invariant. The message is descriptive.

### 4.6 Editor/LSP JSON Serialization Expects — DONE

**Work**:
- [x] replace .expect() calls with proper error propagation
- [x] return LSP error responses on serialization failure

### 4.7 Parser Syntax Tracking Masking — DONE

**Work**:
- [x] replace `unwrap_or_default()` with `.expect("syntax tracking must be active")`

---

## Phase 5: Typecheck Silent Catch-All (P2)

### 5.1 Audit Typecheck Catch-All — DONE

**Work**:
- [x] enumerate all AST expression node kinds
- [x] verify each has explicit handling or an explicit "unsupported in V1" error
- [x] convert `_` catch-all to exhaustive match or explicit rejection
- [x] replace binary operator catch-all with explicit rejections
- [x] add operator type-checking error path tests (10 tests)

---

## Phase 6: Dead Code Cleanup (P3)

### 6.1 Lexer Dead Code — DONE

**Work**:
- [x] remove all commented-out code blocks (done as part of 2.2 lexer fix)

---

## Phase 7: Test Coverage Hardening (P2)

### 7.1 Lexer Tests — PARTIALLY DONE

**Work**:
- [x] add tests for block comments (block_comment_adjacent fixture)
- [ ] add tests for string literals with escapes
- [ ] add tests for numeric literals (int, float, hex, binary, octal)
- [ ] add tests for operator sequences (disambiguation)
- [ ] add tests for Unicode identifiers
- [ ] add tests for maximum token length
- [ ] add tests for empty input and whitespace-only input
- [ ] add tests for unterminated strings and comments

Note: Many of these already exist in the extensive lexer test suite
(test/lexer/test_lexer_comments.rs has 20+ tests, test_lexer_literals.rs,
test_lexer_keywords.rs, test_lexer_errors.rs).

### 7.2 Stream Tests

**Work**:
- [ ] add tests for multi-file module resolution
- [ ] add tests for `.mod` directory handling
- [ ] add tests for namespace isolation
- [ ] add tests for missing file error paths
- [ ] add tests for empty files

### 7.3 Typecheck Error Path Tests — PARTIALLY DONE

**Work**:
- [x] add tests for invalid operator type combinations (10 tests)
- [ ] add test for each TypecheckErrorKind variant (20+ kinds)
- [ ] add tests for type mismatches in assignments
- [ ] add tests for type mismatches in function arguments
- [ ] add tests for type mismatches in return types
- [ ] add tests for invalid container element types
- [ ] add tests for invalid opt/err shell usage
- [ ] add tests for recursive type definitions

### 7.4 Formal V1 End-to-End Tests

**Work**:
- [ ] add test app for arithmetic and comparison operators
- [ ] add test app for boolean logic
- [ ] add test app for string operations
- [ ] add test app for container operations (vec, seq, set, map)
- [ ] add test app for optional values (opt) with check/report
- [ ] add test app for error handling (err) with check/report/||
- [ ] add test app for records with methods
- [ ] add test app for entries
- [ ] add test app for multi-package workspace with cross-package calls
- [ ] add test app for closures/anonymous routines (after Phase 1.4)
- [ ] add test app for loops (after Phase 3.8)

### 7.5 Resolver Error Tests

**Work**:
- [ ] add test for forward reference errors
- [ ] add test for import cycle detection
- [ ] add test for symbol shadowing
- [ ] add test for visibility boundary violations
- [ ] add test for duplicate declarations

### 7.6 Build System Negative Tests

**Work**:
- [ ] add test for invalid build step configurations
- [ ] add test for circular step dependencies
- [ ] add test for missing dependency handling
- [ ] add test for invalid build.fol entry signatures
- [ ] add test for artifact generation failure recovery

### 7.7 Editor LSP Failure Tests

**Work**:
- [ ] add tests for malformed LSP requests
- [ ] add tests for documents with syntax errors
- [ ] add tests for concurrent edit sequences
- [ ] add tests for document close during analysis
- [ ] add tests for workspace with missing files

---

## Phase 8: Explicitly Deferred (Not V1)

These features are intentionally deferred with explicit error messages in the
typechecker. They should NOT be worked on for V1 but their rejection paths
should be verified.

### Beyond V1 (properly rejected at typecheck):
- AsyncStage / AwaitStage (V3 systems milestone)
- Spawn (V3 systems milestone)
- ChannelAccess (V3 systems milestone)
- Select (V3 systems milestone)
- Rolling / comprehensions
- Range expressions
- Yield
- PatternAccess

### Work:
- [x] verify each deferred feature returns clear, user-facing error message
- [x] existing test covers 5/8 deferred features (v1_boundary_rejects_v3_expression_surfaces)
- [ ] add tests for Rolling, PatternAccess, Yield rejection paths

---

## Execution Order

```
Phase 1 (Pipeline Gaps)     ──── most critical, enables real programs
  ├─ 1.1 Binary operators      ✓ DONE
  ├─ 1.2 Unary operators       ✓ DONE
  ├─ 1.3 Invoke                  OPEN
  ├─ 1.4 Anonymous routines      OPEN
  └─ 1.5 Cast instruction      ✓ DONE

Phase 2 (Compiler Bugs)     ──── fix crashes and incorrect behavior
  ├─ 2.1 Block comment bug      ✓ FALSE POSITIVE (verified correct)
  ├─ 2.2 Lexer bounds bug       ✓ DONE
  ├─ 2.3 Dispatch unreachable   ✓ DONE
  ├─ 2.4 Global mutation panic  ✓ DONE
  └─ 2.5 Panic format string    ✓ FALSE POSITIVE (verified correct)

Phase 3 (Pipeline Gaps P2)  ──── expand V1 surface
  ├─ 3.1-3.7 Expression gaps     OPEN
  └─ 3.8 Iteration loops         OPEN

Phase 4 (Panic Hardening)   ──── eliminate crash paths
  ├─ 4.1 Intrinsics panics      ✓ JUSTIFIED INVARIANTS
  ├─ 4.2 Lower session panic    ✓ JUSTIFIED INVARIANT
  ├─ 4.3 Parser unreachable     ✓ DONE
  ├─ 4.4 Typecheck unreachable  ✓ JUSTIFIED INVARIANTS
  ├─ 4.5 Backend unreachable    ✓ JUSTIFIED INVARIANT
  ├─ 4.6 Editor panics          ✓ DONE
  └─ 4.7 Parser masking         ✓ DONE

Phase 5 (Catch-All Audit)   ──── no silent fallthrough
  └─ 5.1 Typecheck catch-all    ✓ DONE

Phase 6 (Dead Code)         ──── cleanup
  └─ 6.1 Lexer dead code        ✓ DONE

Phase 7 (Test Coverage)     ──── lock everything down
  ├─ 7.1 Lexer tests             PARTIALLY DONE
  ├─ 7.2 Stream tests            OPEN
  ├─ 7.3 Typecheck error tests   PARTIALLY DONE
  ├─ 7.4 Formal E2E tests        OPEN
  ├─ 7.5 Resolver error tests    OPEN
  ├─ 7.6 Build negative tests    OPEN
  └─ 7.7 Editor LSP tests        OPEN

Phase 8 (Deferred Verify)   ──── confirm rejection paths
  └─ Verification                PARTIALLY DONE (5/8 tested)
```

## Exit Criteria

V1 is hardened when:

1. Every AST construct that V1 admits has an explicit path through
   typecheck → lower → backend → working binary
2. Every AST construct that V1 does NOT admit produces a clear,
   user-facing error at the earliest possible stage
3. Zero `unreachable!()`, `panic!()`, or `.unwrap()` in non-test compiler
   code without a documented invariant justification
4. Zero silent `_` catch-all match arms in typecheck or lowering
5. Every V1 feature has at least one end-to-end test proving it compiles
   and runs correctly
6. Every V1 rejection path has at least one test proving it fails with
   the right diagnostic
7. The lexer produces correct token streams for all valid V1 syntax
8. The backend generates correct Rust for all lowered V1 IR
