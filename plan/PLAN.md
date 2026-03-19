# Unified Diagnostics Pipeline

Last updated: 2026-03-19

## Problem

Every compiler stage produces structured errors with stable codes, locations,
labels, notes, and helps. Every error type implements `ToDiagnostic`. But the
conversion to `Diagnostic` happens too late and in the wrong places:

1. The parser erases type information at its public API boundary by returning
   `Vec<Box<dyn Glitch>>`. This forces downstream code to manually downcast
   back to `ParseError` just to call `.to_diagnostic()`.

2. The frontend has **two separate** compilation paths with different error
   handling strategies:
   - `direct.rs` uses a `DiagnosticReport` but feeds it through a 5-type
     downcast chain (`lower_compiler_glitch()`)
   - `compile/mod.rs` uses `lower_resolver_errors()`, `lower_typecheck_errors()`,
     `lower_lowering_errors()` which **destroy** all diagnostic structure by
     calling `.to_string()` and stuffing the result into a `FrontendError`

3. The editor has its own parallel downcast chain (`glitch_to_diagnostic()`)
   that duplicates the frontend's.

4. `FrontendError` does not implement `ToDiagnostic`. It carries no diagnostic
   code, no location, no labels. Structured compiler errors that pass through
   `FrontendError` lose everything.

## Root Cause

The `Glitch` trait lives in `fol-types` (the lowest crate). `ToDiagnostic` and
`Diagnostic` live in `fol-diagnostics` (one level up). So `Glitch` cannot carry
a `to_diagnostic()` method without creating a circular dependency.

But every concrete error type that implements `Glitch` also implements
`ToDiagnostic`. The type erasure into `Box<dyn Glitch>` is what forces the
downcast chains.

## Solution

Convert errors to `Diagnostic` at the production site — before type erasure
happens. Then pass `Diagnostic` objects through the pipeline instead of boxed
trait objects.

The conversion chain becomes:

```
Parser     → Vec<Diagnostic>   (converts ParseError at the API boundary)
Package    → PackageError      (already typed, caller calls .to_diagnostic())
Resolver   → Vec<Diagnostic>   (converts ResolverError at the API boundary)
Typecheck  → Vec<Diagnostic>   (converts TypecheckError at the API boundary)
Lower      → Vec<Diagnostic>   (converts LoweringError at the API boundary)
Build      → Diagnostic        (converts BuildEvaluationError at the API boundary)
Frontend   → DiagnosticReport  (collects Diagnostics, renders to CLI/JSON)
Editor/LSP → Vec<LspDiagnostic>(adapts Diagnostics to LSP wire format)
```

No downcast chains. No string flattening. No fallback `E9999` codes.

## Current State

### What already works

- All compiler error types implement both `Glitch` and `ToDiagnostic`
- `DiagnosticReport` already aggregates and renders diagnostics
- `fol-diagnostics::lsp` already adapts `Diagnostic → LspDiagnostic`
- The editor's semantic analysis path (resolver, typecheck) already calls
  `.to_diagnostic()` directly on typed errors
- Diagnostic codes are stable: P1xxx, K1xxx, R1xxx, T1xxx, L1xxx, K11xx

### What needs to change

| Location | Problem | Fix |
|----------|---------|-----|
| `fol-parser` public API | Returns `Vec<Box<dyn Glitch>>` | Return `Vec<Diagnostic>` |
| `fol-frontend/direct.rs` | `lower_compiler_glitch()` downcast chain | Use `Diagnostic` directly |
| `fol-frontend/compile/mod.rs` | `lower_*_errors()` flattens to strings | Use `DiagnosticReport` |
| `fol-editor/analysis.rs` | `glitch_to_diagnostic()` downcast chain | Use `Diagnostic` directly |
| `FrontendError` | No `ToDiagnostic`, no codes | Implement `ToDiagnostic` with F1xxx codes |
| `Glitch` trait | Used for polymorphic storage but forces erasure | Reduce usage to parser internals |

## Implementation Slices

### Slice 1: Parser returns Diagnostic at the public boundary

The parser internally uses `Vec<Box<dyn Glitch>>` for error collection during
recovery. All of these are `ParseError` in practice. Change the public API
to convert at the boundary.

Work:

- change `parse_package()` return type from `Result<T, Vec<Box<dyn Glitch>>>`
  to `Result<T, Vec<Diagnostic>>`
- same for `parse_script_package()` and `parse_decl_package()`
- add a private helper that maps `Vec<Box<dyn Glitch>>` → `Vec<Diagnostic>`
  by downcasting to `ParseError` (with `from_glitch` fallback)
- keep internal parser methods using `Box<dyn Glitch>` — only change the
  public return type
- update all callers:
  - `fol-package` (`parse_directory_package_syntax`)
  - `fol-frontend/direct.rs` (`compile_file`)
  - `fol-editor/analysis.rs` (`parse_single_file_diagnostics`,
    `parse_directory_diagnostics`)

Exit condition:

- `parse_package()` returns `Vec<Diagnostic>`
- no downstream code needs to downcast `Box<dyn Glitch>` from parser output

### Slice 2: Frontend direct mode uses Diagnostic natively

The `direct.rs` compilation path already uses `DiagnosticReport`. But it feeds
errors through `add_compiler_glitch()` → `lower_compiler_glitch()` which is
a 5-type downcast chain.

After Slice 1, parser errors arrive as `Vec<Diagnostic>`. The remaining error
types (resolver, typecheck, lower) are already typed and have `.to_diagnostic()`.

Work:

- in `compile_file()`:
  - parser errors: iterate `Vec<Diagnostic>`, call `report.add_diagnostic()`
  - package error: call `error.to_diagnostic()`, then `report.add_diagnostic()`
  - resolver errors: iterate, call `.to_diagnostic()` + `report.add_diagnostic()`
  - typecheck errors: same
  - lowering errors: same
- delete `add_compiler_glitch()`
- delete `lower_compiler_glitch()`

Exit condition:

- `direct.rs` has no downcast chain
- `add_compiler_glitch` and `lower_compiler_glitch` are deleted

### Slice 3: Frontend workspace mode preserves diagnostic structure

The `compile/mod.rs` path converts structured compiler errors into flat strings:

```rust
fn lower_resolver_errors(errors: Vec<ResolverError>) -> FrontendError {
    FrontendError::new(
        FrontendErrorKind::CommandFailed,
        errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"),
    )
}
```

This destroys codes, locations, labels, notes, helps — everything.

Work:

- change `compile_member_workspace()` to collect a `DiagnosticReport` instead
  of returning `FrontendError` on compilation failure
- the function can return `Result<LoweredWorkspace, DiagnosticReport>` or
  accept a `&mut DiagnosticReport` parameter
- delete `lower_resolver_errors()`, `lower_typecheck_errors()`,
  `lower_lowering_errors()`
- update callers in `build_member_workspace()`, `emit_lowered_with_config()`,
  and the build route execution path

Exit condition:

- `compile/mod.rs` has no string-flattening error helpers
- workspace compilation errors preserve full diagnostic structure

### Slice 4: Editor drops the downcast chain

After Slice 1, parser output is `Vec<Diagnostic>`. The editor no longer needs
`glitch_to_diagnostic()`.

Work:

- in `parse_single_file_diagnostics()`: parser now returns `Vec<Diagnostic>`
  directly — just use it
- in `parse_directory_diagnostics()`: same
- delete `glitch_to_diagnostic()`

Exit condition:

- `glitch_to_diagnostic` is deleted
- editor analysis has zero downcast logic

### Slice 5: FrontendError implements ToDiagnostic

`FrontendError` is the frontend's own error type for workspace, CLI, and I/O
failures. It carries no diagnostic code and formats as
`"FrontendWorkspaceNotFound: message"`.

Work:

- assign stable diagnostic codes:
  - F1001: InvalidInput
  - F1002: WorkspaceNotFound
  - F1003: PackageFailed
  - F1004: CommandFailed
  - F1099: Internal
- implement `ToDiagnostic` for `FrontendError`
- preserve notes through the diagnostic model
- route frontend error display through `DiagnosticReport` rendering where
  it reaches the CLI output path

Exit condition:

- `FrontendError` implements `ToDiagnostic`
- frontend errors render with `[F1xxx]` codes through the same pipeline

### Slice 6: Consolidate the two frontend compilation paths

After Slices 2-3, both `direct.rs` and `compile/mod.rs` use `DiagnosticReport`
for error handling. Unify the shared compilation logic.

Work:

- extract a shared compilation function that runs
  parse → package → resolve → typecheck → lower and collects diagnostics
- both `direct.rs` and `compile/mod.rs` call this shared function
- extract shared binary execution helper (currently copy-pasted in 3 places)
- decide whether `direct.rs` should reuse the workspace compilation path
  entirely or remain a separate entry point

Exit condition:

- one compilation function, one error handling strategy

### Slice 7: Reduce Glitch trait surface

After Slices 1-4, `Box<dyn Glitch>` is no longer used at any public API
boundary. It remains used:

- internally in the parser for error collection during recovery
- in `fol-stream` for I/O errors
- in `DiagnosticReport::add_error()` which accepts `&dyn Glitch`

Work:

- evaluate whether `DiagnosticReport::add_error(&dyn Glitch)` should be
  replaced with `add_diagnostic(Diagnostic)` everywhere
- evaluate whether `fol-stream` should return its own typed error instead of
  `Box<dyn Glitch>`
- evaluate whether the parser's internal error collection should use
  `ParseError` directly instead of `Box<dyn Glitch>`
- for each remaining `Glitch` usage, decide: keep, replace, or remove
- remove `Glitch` impls from error types that no longer need them

Exit condition:

- `Glitch` usage is confined to the minimum necessary
- no public API returns `Box<dyn Glitch>`

### Slice 8: Integration tests for the unified pipeline

Add tests that verify the same diagnostic flows identically through CLI, JSON,
and LSP output.

Work:

- write a fixture that produces each error family (P, K, R, T, L, F)
- verify CLI human output, CLI JSON output, and LSP diagnostic output all
  carry the same code, message, location, and labels
- verify that `DiagnosticReport` dedup and LSP dedup produce consistent results
- verify that adding a new compiler error type does not require changes to
  the frontend or editor error handling code

Exit condition:

- a single test suite proves end-to-end diagnostic fidelity

## Architecture After

```
┌─────────────┐     ┌──────────────┐
│  fol-parser  │────>│  Diagnostic  │
│  fol-package │────>│  Diagnostic  │
│  fol-resolver│────>│  Diagnostic  │
│  fol-typecheck────>│  Diagnostic  │
│  fol-lower   │────>│  Diagnostic  │
│  fol-build   │────>│  Diagnostic  │
└─────────────┘     └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │DiagnosticReport│
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
       ┌──────▼──┐  ┌──────▼──┐  ┌─────▼────┐
       │CLI Human│  │CLI JSON │  │LSP Adapter│
       └─────────┘  └─────────┘  └──────────┘
```

Every stage converts its own errors to `Diagnostic` before returning them.
No downstream code needs to know which error type produced the diagnostic.
No downcasting. No string flattening. One pipeline.

## What This Plan Does Not Cover

- `BackendError` and `RuntimeError` — these are execution-layer errors, not
  compiler diagnostics. They do not implement `Glitch` or `ToDiagnostic` and
  that is intentional for now.
- `EditorError` — this is a transport/session error, not a diagnostic. It
  stays as-is.
- Tree-sitter maintenance, language facts manifest, feature checklist — these
  are covered by the completed slices from the previous plan and are not
  reopened here.

## Immediate Next Slice

Start with Slice 1.

Rationale:

- the parser is the only stage that erases type information at its public API
- fixing it eliminates both downcast chains (frontend and editor) in one move
- all callers already handle `Vec<_>` — changing the element type is mechanical
