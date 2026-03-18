# FOL V1 Hardening Plan

Last updated: 2026-03-18

## Context

Round 1 (build reset) and Round 2 (fol-build execution crate) are complete.

Round 3 is a hardening pass across the entire compiler pipeline, from
stream through backend, including the package system, build executor,
frontend tooling, editor, and cross-cutting concerns. The goal is to
eliminate panics on user input, close validation gaps, fix bugs found
during deep audit, and unify project-level inconsistencies before V1.

## Audit Summary

A full audit of every crate produced 80+ findings across 6 areas. They are
grouped into 10 slices ordered by severity and pipeline position.

## No Backwards Compatibility

Same policy as before. If something is replaced, the old thing is deleted.

---

## Round 3 Slices

- [ ] Slice 1. Lexer: off-by-one fix, missing keywords, typos
- [ ] Slice 2. Parser: loop limits, depth guards, error recovery
- [ ] Slice 3. Resolver: unwrap/expect removal, scope validation, path safety
- [ ] Slice 4. Typechecker: catch-all elimination, type table safety, error taxonomy
- [ ] Slice 5. Lowering: panic removal, test fixes, type validation
- [ ] Slice 6. Backend: codegen string escaping, generated code safety
- [ ] Slice 7. Package: lockfile version check, git hostname validation, metadata parsing
- [ ] Slice 8. Build executor: recursion limits, scope bounds, name validation, panic removal
- [ ] Slice 9. Frontend and editor: CLI dispatch safety, binary output capture, diagnostics
- [ ] Slice 10. Cross-cutting: version unification, dependency cleanup, legacy removal

---

### Slice 1 — Lexer: off-by-one fix, missing keywords, typos

**Severity**: CRITICAL

Three bugs in the lexer must be fixed before anything else.

#### 1a. Off-by-one in sliding window peek/seek

**Files**: `fol-lexer/src/lexer/stage2/elements.rs` (lines 47-52, 59-64),
`fol-lexer/src/lexer/stage3/elements.rs` (lines 68-73, 80-85)

When `ignore=true` and a space is encountered, `u` is incremented without
rechecking bounds. After `u += 1`, `u` can equal `SLIDER`, causing an
out-of-bounds panic on `self.next_vec()[u]`.

Fix: check `u + 1 < SLIDER` before incrementing, or recheck after increment.

#### 1b. Missing keyword handlers in stage1 lexer

**File**: `fol-lexer/src/lexer/stage1/element.rs` (lines 414-468)

The `alpha()` function's keyword match statement is missing entries for:

- `"get"` → `BUILDIN::Get` (CRITICAL — keyword exists in enum, Display, and
  parser but the lexer never produces it)
- `"nor"` → `BUILDIN::Nor` (MEDIUM)
- `"at"` → `BUILDIN::At` (MEDIUM)

Add the missing match arms.

#### 1c. Typo: "yeild" should be "yield"

**Files**: `fol-lexer/src/token/buildin/mod.rs` (line 44 enum, line 104 Display),
`fol-lexer/src/lexer/stage1/element.rs` (line 450 match arm)

The enum variant is `Yeild` and the keyword string is `"yeild"`. Fix the
spelling to `Yield` / `"yield"` everywhere. This is a breaking change to
any existing FOL code using `yeild`, but per policy that is acceptable.

Exit criteria: no off-by-one panic possible, all defined keywords are lexable,
`yield` is spelled correctly.

---

### Slice 2 — Parser: loop limits, depth guards, error recovery

**Severity**: HIGH

#### 2a. Silent truncation on hardcoded loop limits

**File**: `fol-parser/src/ast/parser_parts/primary_expression_parsers.rs` (line 50)

Record initializer field parsing loops `for _ in 0..256`. If a record has
more than 256 fields, the remaining fields are silently dropped.

**File**: `fol-parser/src/ast/parser_parts/expression_atoms_and_literal_lowering.rs`
(lines 40, 91) — `skip_layout` and `skip_ignorable` limit to 128 iterations.

Fix: either raise a parser error when the limit is hit, or remove the limit
and rely on the depth guard instead.

#### 2b. Parser depth guard overflow in release builds

**File**: `fol-parser/src/ast/parser.rs` (lines 137-147)

`ParseDepthGuard` increments depth with `+ 1` without overflow check. The
`debug_assert` on line 144 only fires in debug builds. In release, the counter
wraps silently.

Fix: use `checked_add(1)` and return a parser error on overflow.

#### 2c. Unclosed block comment produces confusing error

**File**: `fol-lexer/src/lexer/stage1/element.rs` (lines 109-119)

An unclosed `/* ... ` produces an `Illegal` token instead of a clear
"unterminated block comment" diagnostic.

Fix: add a specific error variant or diagnostic message for unterminated
comments.

Exit criteria: loop limits produce errors instead of silent truncation,
depth guard cannot overflow, unterminated comments produce clear diagnostics.

---

### Slice 3 — Resolver: unwrap/expect removal, scope validation, path safety

**Severity**: HIGH

#### 3a. expect() on scope lookup after mutation

**File**: `fol-resolver/src/model.rs` (line 699)

`insert_mounted_symbol` calls `.expect("mounted symbol target scope should exist")`
on a scope lookup that could fail if scope state becomes inconsistent.

Fix: return a resolver error instead of panicking.

#### 3b. Missing source unit validation in traverse

**File**: `fol-resolver/src/traverse.rs` (line 69)

`program.source_unit(source_unit_id).expect(...)` — if a source unit is deleted
between collection and traversal, this panics.

Fix: return a diagnostic error.

#### 3c. Namespace scope lookup with expect()

**File**: `fol-resolver/src/collect.rs` (line 58)

Namespace scope is looked up with `.expect()` assuming it always exists.

Fix: propagate as resolver error.

#### 3d. Import path validation gap

**File**: `fol-resolver/src/imports.rs` (lines 264-278)

`resolve_directory_path()` does not validate that path segments don't contain
`..` or absolute path components in relative import context.

Fix: reject path segments containing `..`, `/`, or that start with `/`.

#### 3e. Silent symbol skip in mount_visible_symbols_from_scope

**File**: `fol-resolver/src/model.rs` (lines 628-654)

If a foreign symbol ID becomes invalid, the resolver silently `continue`s
without any diagnostic.

Fix: emit a warning diagnostic.

#### 3f. Build stdlib scope return value ignored

**File**: `fol-resolver/src/inject.rs` (line 21)

`init_build_stdlib_scope()` returns `Option<ScopeId>` but the return value
is not checked.

Fix: if build stdlib injection fails for a package with build units, return
a resolver error.

Exit criteria: zero `.expect()` or `.unwrap()` calls on resolver data
influenced by user input, import paths validated against traversal.

---

### Slice 4 — Typechecker: catch-all elimination, type table safety, error taxonomy

**Severity**: HIGH

#### 4a. Catch-all pattern silently types unknown nodes as None

**File**: `fol-typecheck/src/exprs.rs` (line 448)

```rust
_ => {
    for child in node.children() {
        let _ = type_node(typed, resolved, context, child)?;
    }
    Ok(TypedExpr::none())
}
```

Any unhandled AST node type gets `None` type. New language features could
silently pass typechecking.

Fix: log/warn on unknown node types, or enumerate all expected node types
explicitly.

#### 4b. Routine scope fallback without validation

**File**: `fol-typecheck/src/exprs.rs` (line 244)

`.unwrap_or(context.scope_id)` silently uses parent scope if routine scope
mapping fails.

Fix: return a typecheck error if the routine scope cannot be resolved.

#### 4c. Silent failures in multi-package type hydration

**File**: `fol-typecheck/src/session.rs` (lines 216-238, 269-274)

Three places where package context loss during type import produces generic
"Internal" errors without identifying which package or symbol failed.

Fix: include package identity and symbol name in error messages.

#### 4d. Unchecked type table access with generic errors

**File**: `fol-typecheck/src/decls.rs` (lines 691-706)

Two different error messages for the same class of failure (symbol table loss).

Fix: unify error format, include symbol ID and context.

#### 4e. TypecheckErrorKind has only 4 variants for 12+ scenarios

The error taxonomy is too coarse. `Internal` is used for many distinct failure
modes that should be distinguishable.

Fix: add error variants: `ScopeResolutionFailed`, `TypeImportFailed`,
`SymbolTableCorrupted`, `UnsupportedSyntax`.

Exit criteria: no catch-all `_ =>` that silently assigns None type,
all type errors include context (package, symbol, location).

---

### Slice 5 — Lowering: panic removal, test fixes, type validation

**Severity**: HIGH

#### 5a. Panics in test code with wrong variable names

**File**: `fol-backend/src/control.rs` (lines 114-119, 238)

Test declares `let _table` but uses `table` (without underscore). This is
a compilation error in the test.

Fix: rename `_table` to `table`.

#### 5b. Set member type validation incomplete

**File**: `fol-lower/src/exprs.rs` (lines 3756-3765)

For heterogeneous sets with literal indexing, the fallback checks if all
members are the same type. If they aren't and the index is out of range,
the wrong type variant may be accessed.

Fix: emit a lowering error for heterogeneous set indexing with non-constant
index.

Exit criteria: no panics in test code, type validation for set indexing.

---

### Slice 6 — Backend: codegen string escaping, generated code safety

**Severity**: CRITICAL

#### 6a. String escaping bug in panic terminator codegen

**File**: `fol-backend/src/control.rs` (lines 50-55)

The panic terminator generates `panic!("{}", ...)` but the format string
`"{}"` is itself a format placeholder being rendered through another
`format!()`. The generated Rust code may be malformed.

Fix: verify the double-format produces correct Rust, or simplify to direct
string interpolation.

#### 6b. Dead code placeholders in operand rendering

**File**: `fol-backend/src/instructions.rs` (lines 644-645)

`LoweredOperand::Local(_)` and `::Global(_)` emit `/*local*/` and `/*global*/`
comment strings instead of valid Rust code.

Fix: return a codegen error for unimplemented operand types.

#### 6c. Excessive .clone() in generated container operations

**File**: `fol-backend/src/instructions.rs` (lines 302-311, 1495+)

Every container index access clones both the index and the result. This adds
unnecessary allocation overhead.

Fix: generate move semantics where ownership allows, avoid double-clone.

#### 6d. unreachable!() in type mapping

**File**: `fol-backend/src/types.rs` (line 358)

`LoweredBuiltinType::Str => unreachable!(...)` — if this path is ever hit,
it panics instead of producing a codegen error.

Fix: return `Err(BackendError::...)`.

Exit criteria: generated Rust code compiles correctly for all panic/error
paths, no `unreachable!()` in production codegen paths.

---

### Slice 7 — Package: lockfile version check, git hostname validation, metadata parsing

**Severity**: MEDIUM-HIGH

#### 7a. Lockfile version not validated for compatibility

**File**: `fol-package/src/lockfile.rs` (lines 72-144)

After parsing the lockfile version number, no check verifies it matches the
expected version. Future format changes would silently read incompatible
lockfiles.

Fix: add `if version != LOCKFILE_VERSION { return Err(...) }`.

#### 7b. Git locator hostname not validated

**File**: `fol-package/src/locator.rs` (lines 110-187)

SSH and HTTPS locators parse the host but don't validate it for DNS compliance.
A malicious locator could contain injection characters.

Fix: validate hostname format (alphanumeric, dots, hyphens only).

#### 7c. Metadata inline comment stripping doesn't handle escaped quotes

**File**: `fol-package/src/metadata.rs` (lines 293-307)

`strip_inline_comment()` doesn't handle backslash-escaped quotes inside
strings. A value like `"path with \" inside"` is mis-parsed.

Fix: handle `\\` and `\"` escape sequences in the scanner.

#### 7d. Package name validation missing length limit

**File**: `fol-package/src/metadata.rs` (lines 358-374)

`is_valid_package_name()` has no length constraint. An extremely long name
could cause allocation issues.

Fix: add `if name.len() > 256 { return false }`.

#### 7e. Git revision sanitization path traversal

**File**: `fol-package/src/paths.rs` (lines 37-46)

`sanitize_revision_segment()` replaces non-alphanumeric chars with `_` but
a carefully crafted revision could still produce unexpected paths. The current
implementation is safe (replaces `/` with `_`), but should be documented and
tested against traversal vectors.

Fix: add explicit test cases for `../`, `..\\`, `\x00` inputs.

Exit criteria: lockfile version mismatch produces clear error, git hostnames
are validated, metadata parsing handles edge cases.

---

### Slice 8 — Build executor: recursion limits, scope bounds, name validation, panic removal

**Severity**: HIGH

#### 8a. No recursion depth limit in executor

**File**: `fol-build/src/executor.rs`

`eval_expr()`, `exec_stmt()`, and `exec_body()` call each other recursively
without any depth limit. A deeply nested build script could overflow the stack.

Fix: add `recursion_depth: usize` field to `BuildBodyExecutor`, check against
`MAX_EVAL_DEPTH` (e.g., 256) at each recursive entry point.

#### 8b. No recursion depth limit in graph cycle detection

**File**: `fol-build/src/graph.rs` (lines 576-613)

`visit_step_dependencies()` uses recursive DFS without depth limiting.

Fix: add `MAX_GRAPH_DEPTH` constant, check `stack.len()` at each recursive call,
or convert to iterative DFS.

#### 8c. Unbounded scope growth

**File**: `fol-build/src/executor.rs` (lines 130-142)

`scope: BTreeMap` and `helpers: BTreeMap` grow without limit.

Fix: add `MAX_SCOPE_SIZE` (e.g., 10,000) and check before each insert.

#### 8d. No validation of option names or step names

**Files**: `fol-build/src/executor.rs` (lines 567-581, 650-690)

Names extracted from user build code go directly into scope maps without
validation. Reserved names, empty strings, or names with special characters
are accepted.

Fix: validate names against `[a-z][a-z0-9_-]*` pattern before insertion.

#### 8e. Panics in artifact output processing

**File**: `fol-build/src/artifact.rs` (lines 357, 363, 369)

Three `panic!()` calls in match arms for unexpected output types.

Fix: return `BuildEvaluationError` instead.

#### 8f. unreachable!() in executor type matching

**File**: `fol-build/src/executor.rs` (line 1126)

Fix: return proper error.

Exit criteria: build executor cannot stack-overflow on any input, all names
validated, zero panics in production paths.

---

### Slice 9 — Frontend and editor: CLI dispatch safety, binary output capture, diagnostics

**Severity**: HIGH

#### 9a. Multiple unwrap() on cli.command in dispatch path

**File**: `fol-frontend/src/lib.rs` (lines 427-470)

After checking `cli.command.is_none()`, code calls `.unwrap()` on command
references throughout the dispatch function.

Fix: use pattern matching or `let Some(cmd) = ... else { ... }`.

#### 9b. Binary execution stderr not captured on failure

**File**: `fol-frontend/src/direct.rs` (lines 297-308)

Process execution uses `.status()` instead of `.output()`, discarding stderr
from the executed binary. Users don't see their program's error messages.

Fix: use `.output()` and forward stderr to the user.

**File**: `fol-frontend/src/compile.rs` (lines 196-201)

Same pattern — binary output discarded on failure.

#### 9c. Silent path canonicalization fallback

**File**: `fol-frontend/src/fetch.rs` (line 244)

`canonicalize(&root).unwrap_or(root.clone())` silently falls back without
diagnostic. Symlinks or permission issues go unreported.

Fix: log a warning when canonicalization fails.

#### 9d. "ParserMissmatch" typo in error types

**File**: `fol-types/src/error.rs`

Variant name `ParserMissmatch` — should be `ParserMismatch`.

Fix: rename the variant.

#### 9e. DiagnosticCode::unknown() returns "E0000"

**File**: `fol-diagnostics/src/codes.rs` (lines 12-14)

"E0000" could collide with a real error code.

Fix: use a sentinel like "E????" or "EUNKNOWN".

#### 9f. Color output control is global state

**File**: `fol-frontend/src/ui.rs` (lines 27-50)

`colored::control::set_override(...)` sets global state, not thread-safe.

Fix: remove global override, use per-writer color control or accept the
global as a process-level setting (document it).

Exit criteria: CLI dispatch cannot panic, binary stderr is always visible
to the user, all typos fixed.

---

### Slice 10 — Cross-cutting: version unification, dependency cleanup, legacy removal

**Severity**: MEDIUM

#### 10a. Crate version inconsistency

9 crates are at `0.1.4`, 7 crates are at `0.1.0`. All workspace members
should use the same version.

Fix: set all crates to `0.1.4` (or use workspace version inheritance).

#### 10b. colored crate version mismatch

- `fol-lexer`, `fol-types`, `fol-stream`: `colored = "1"`
- `fol-parser`, `fol-diagnostics`: `colored = "1.9"`
- `fol-frontend`: `colored = "2"`

Fix: unify to `colored = "2"` everywhere and update any API differences.

#### 10c. Legacy test directory still exists

**Path**: `test/legacy/` — contains old `.mod` files.

Per project policy: delete entirely.

#### 10d. Dead code attributes

**Files**: `fol-lexer/src/point.rs` (lines 1-2), `fol-types/src/mod.rs` (line 1)

`#![allow(dead_code)]` and `#![allow(unused_variables)]` suppress warnings
for unused code.

Fix: audit and remove dead code, then remove the allow attributes.

#### 10e. .gitignore issues

**File**: `.gitignore`

- Line 3: `".dirnev"` is a typo for `.direnv`
- Line 7: `"docs"` is too broad
- Missing: `*.swp`, `.idea/`, `*.log`

Fix: correct typo, narrow docs pattern, add missing patterns.

#### 10f. Build artifacts in test fixtures

**Path**: `test/app/build/exe_object_config/.fol/build/`

Compiled artifacts are checked into the repository.

Fix: add `.fol/build/` to `.gitignore` and remove from git.

#### 10g. fol-package depends on fol-build (architectural inversion)

The compiler layer depends on the execution layer. This is an architecture
smell but is currently functional because fol-package re-exports build types.

Fix (deferred): document the dependency direction, consider whether fol-package
should only depend on fol-build types (not execution logic).

Exit criteria: all crates at same version, single colored version, no legacy
files, clean .gitignore.

---

## Issue Count by Severity

| Severity | Count | Slices |
|----------|-------|--------|
| CRITICAL | 6 | 1, 6 |
| HIGH | 25 | 2, 3, 4, 5, 8, 9 |
| MEDIUM | 18 | 7, 10 |
| LOW | 8 | scattered |

## Success Definition

Done when:

- Zero `.unwrap()` or `.expect()` on data reachable from user input
- Zero `panic!()` or `unreachable!()` in production (non-test) code paths
- All defined keywords produce correct tokens
- Parser cannot silently truncate input
- Build executor cannot stack-overflow
- All generated Rust code compiles correctly
- All crate versions and dependency versions are unified
- No legacy files remain
- All tests pass
