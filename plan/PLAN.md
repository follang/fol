# Unified Diagnostics Pipeline

Last updated: 2026-03-19

## Problems

### 1. Glitch trait forces type erasure and downcasting

The `Glitch` trait in `fol-types` erases concrete error types into
`Box<dyn Glitch>`. Every consumer that needs structured error data must then
downcast back to the original type. This creates:

- a 5-type downcast chain in `fol-frontend/direct.rs` (`lower_compiler_glitch`)
- a 4-type downcast chain in `fol-editor/analysis.rs` (`glitch_to_diagnostic`)
- 2 downcast sites in `fol-package` to recover `ParseError` location data
- downcast sites in integration tests

All concrete error types already implement `ToDiagnostic`. The type erasure
is unnecessary.

### 2. Two frontend compilation paths with different error handling

- `direct.rs`: uses `DiagnosticReport` + downcast chain
- `compile/mod.rs`: uses `lower_*_errors()` which flatten structured errors
  to plain strings via `.to_string()`, losing codes, locations, and labels

### 3. Colors baked into compiler library crates

The `colored` crate is a dependency in 5 compiler/library crates:

- `fol-types` — `logit!` macro uses `.red()`
- `fol-stream` — error messages use `.red()`
- `fol-lexer` — every token `Display` impl uses `.black().on_red()` etc.
- `fol-parser` — dependency inherited but not directly used
- `fol-diagnostics` — `render_human.rs` applies colors unconditionally

Colors should only exist in `fol-frontend` where the output mode is known.
Library crates should produce plain text.

### 4. Legacy error types with no purpose

`fol-types` defines `Flaw`, `Typo`, `Slip` — old error enums used only in
the lexer (3 call sites). Also defines type aliases `Con<T>`, `Vod`, `Errors`
that wrap `Box<dyn Glitch>`. These are legacy abstractions that spread the
trait object pattern.

## Solution

1. Replace `Box<dyn Glitch>` with concrete error types everywhere
2. Convert compiler errors to `Diagnostic` at the production site
3. Strip colors from all compiler/library crates
4. Move colored rendering to `fol-frontend` only

## Current Inventory

### Glitch implementations (10 types)

| Type | Crate | Also implements ToDiagnostic | Used as Box\<dyn Glitch\> |
|------|-------|------------------------------|---------------------------|
| BasicError | fol-types | No | Yes (stream I/O) |
| Flaw | fol-types | No | Yes (lexer stage1) |
| Typo | fol-types | No | Yes (lexer stage3) |
| Slip | fol-types | No | Dead code |
| ParseError | fol-parser | Yes | Yes (parser public API) |
| PackageError | fol-package | Yes | No (returned typed) |
| ResolverError | fol-resolver | Yes | No (returned typed) |
| TypecheckError | fol-typecheck | Yes | No (returned typed) |
| LoweringError | fol-lower | Yes | No (returned typed) |
| BuildEvaluationError | fol-build | Yes | No (returned typed) |

### Glitch-dependent type aliases in fol-types

```rust
pub type Con<T> = Result<T, Box<dyn Glitch + 'static>>;
pub type Vod = Result<(), Box<dyn Glitch + 'static>>;
pub type Errors = Vec<Box<dyn Glitch>>;
```

Used in ~30 lexer signatures (`Con<Element>`, `Con<Part<char>>`, `Vod`).

### Color usage in compiler crates

| Crate | Files | Usage |
|-------|-------|-------|
| fol-types | mod.rs | `logit!` macro: `.red()`, `terminal_size` |
| fol-stream | lib.rs | Error messages: `.red()` |
| fol-lexer | 9 files | Every Display impl: `.black().on_red()`, `.white().on_black()` |
| fol-parser | — | Cargo.toml dep only, no direct usage |
| fol-diagnostics | render_human.rs | `.red().bold()`, `.yellow().bold()`, `.blue().bold()` |

## Implementation Slices

### Slice 1: Strip colors from compiler library crates ✅

Remove the `colored` and `terminal_size` dependencies from all compiler crates.
Library code must produce plain text.

Work:

- `fol-types`: remove `colored` and `terminal_size` from Cargo.toml, rewrite
  `logit!` macro without colors (or delete it if unused outside tests)
- `fol-stream`: remove `colored` from Cargo.toml, strip `.red()` from error
  messages in `lib.rs`
- `fol-lexer`: remove `colored` from Cargo.toml, rewrite all token Display
  impls to use plain text labels (e.g. `BUILDIN:fun` instead of colored blocks)
- `fol-parser`: remove `colored` from Cargo.toml
- `fol-diagnostics`: remove `colored` from Cargo.toml, make `render_human.rs`
  output plain text (colors will be added by the frontend in a later slice)

Keep `colored` only in `fol-frontend`.

Exit condition:

- `cargo tree -p fol-lexer | grep colored` returns nothing
- `cargo tree -p fol-diagnostics | grep colored` returns nothing
- all compiler crate tests pass with plain text output

### Slice 2: Add color support to fol-frontend diagnostic rendering ✅

After Slice 1, `render_human.rs` in `fol-diagnostics` produces plain text.
The frontend needs to apply colors at the presentation layer.

Work:

- add a render option or wrapper in `fol-frontend` that colorizes diagnostic
  output before printing
- or: make `render_human.rs` accept a color flag/trait so the frontend can
  opt in
- keep `fol-diagnostics` color-free as a library

Exit condition:

- `fol tool lsp` and CLI human output still show colored diagnostics
- `fol-diagnostics` crate has no color dependency

### Slice 3: Replace lexer error types with a concrete enum ✅

The lexer uses `Flaw`, `Typo`, `Slip` through `Box<dyn Glitch>` via the
`Con<T>` and `Vod` aliases. Replace with a concrete enum.

Work:

- define `LexerError` enum in `fol-lexer` (or `fol-types`):
  ```rust
  enum LexerError {
      ReadingBadContent(String),
      GettingNoEntry(String),
      GettingWrongPath(String),
      LexerSpaceAdd(String),
      ParserMismatch(String),
  }
  ```
- replace `Con<T>` with `Result<T, LexerError>` across lexer stages
- replace `Vod` with `Result<(), LexerError>`
- delete `Flaw`, `Typo`, `Slip`
- delete `Con<T>`, `Vod`, `Errors` type aliases
- delete the `catch!` macro (just use `LexerError::Variant(msg)`)
- update `fol-stream` to return `Result<T, StreamError>` or `Result<T, String>`
  instead of `Result<T, Box<dyn Glitch>>`

Exit condition:

- no `Box<dyn Glitch>` in fol-lexer or fol-stream
- `Con<T>`, `Vod`, `Errors` are deleted

### Slice 4: Parser returns Vec\<Diagnostic\> at the public boundary ✅

The parser internally collects `Vec<Box<dyn Glitch>>` during error recovery.
These are always `ParseError`. Change the public API to return `Vec<Diagnostic>`.

Work:

- change `parse_package()` return from `Result<T, Vec<Box<dyn Glitch>>>` to
  `Result<T, Vec<Diagnostic>>`
- same for `parse_script_package()`, `parse_decl_package()`
- add a private conversion: in the public methods, map each boxed error to
  `Diagnostic` via downcast-to-ParseError (this is the LAST downcast, owned
  by the parser itself)
- internally the parser can keep `Vec<Box<dyn Glitch>>` for now, or switch to
  `Vec<ParseError>` if straightforward
- update all callers:
  - `fol-package` session and build modules
  - `fol-frontend/direct.rs`
  - `fol-editor/analysis.rs`

Exit condition:

- parser public API returns `Vec<Diagnostic>`
- no downstream code downcasts parser output

### Slice 5: Remove Glitch implementations from compiler error types

After Slices 3-4, the only remaining `Glitch` users are `BasicError` (used by
`fol-stream` before Slice 3) and the parser's internal error collection.

Work:

- remove `impl Glitch for ParseError`
- remove `impl Glitch for PackageError`
- remove `impl Glitch for ResolverError`
- remove `impl Glitch for TypecheckError`
- remove `impl Glitch for LoweringError`
- remove `impl Glitch for BuildEvaluationError`
- remove `impl Glitch for BasicError`
- if the parser still uses `Box<dyn Glitch>` internally, switch its internal
  error collection to `Vec<ParseError>`
- delete the `Glitch` trait from `fol-types`
- delete `BasicError` if no longer used (or keep as a simple error struct
  without Glitch)
- delete `impl Clone for Box<dyn Glitch>`
- delete the `erriter!`, `noditer!`, `halt!`, `crash!` macros if unused

Exit condition:

- the `Glitch` trait does not exist
- `grep -r "dyn Glitch" lang/` returns nothing
- all tests pass

### Slice 6: Frontend and editor use Diagnostic directly

With parser returning `Vec<Diagnostic>` and all other stages returning typed
errors with `.to_diagnostic()`:

Work:

- `fol-frontend/direct.rs`:
  - parser errors: iterate `Vec<Diagnostic>`, add to report
  - typed errors: call `.to_diagnostic()` directly
  - delete `add_compiler_glitch()`, `lower_compiler_glitch()`
- `fol-frontend/compile/mod.rs`:
  - delete `lower_resolver_errors()`, `lower_typecheck_errors()`,
    `lower_lowering_errors()` — these flatten to strings
  - use `DiagnosticReport` to collect diagnostics properly
- `fol-editor/analysis.rs`:
  - parser returns `Vec<Diagnostic>` — use directly
  - delete `glitch_to_diagnostic()`
- `DiagnosticReport::add_error(&dyn Glitch, ...)`:
  - this method accepted `&dyn Glitch` — replace with
    `add_diagnostic(Diagnostic)` or a concrete fallback

Exit condition:

- no downcast chains in frontend or editor
- no string-flattening error helpers
- `glitch_to_diagnostic` and `lower_compiler_glitch` are deleted

### Slice 7: FrontendError implements ToDiagnostic

`FrontendError` carries no diagnostic code and no location. It formats as
`"FrontendWorkspaceNotFound: message"`.

Work:

- assign stable codes: F1001 (InvalidInput), F1002 (WorkspaceNotFound),
  F1003 (PackageFailed), F1004 (CommandFailed), F1099 (Internal)
- implement `ToDiagnostic` for `FrontendError`
- preserve notes through the diagnostic model
- route frontend error display through diagnostic rendering

Exit condition:

- `FrontendError` has `[F1xxx]` codes
- frontend errors render through the same pipeline as compiler errors

### Slice 8: Integration tests for the unified pipeline

Verify that the same diagnostic flows identically through all output paths.

Work:

- fixture that produces each error family (P, K, R, T, L, F)
- verify CLI human, CLI JSON, and LSP all carry the same code, message,
  location, and labels
- verify no `Box<dyn Glitch>` exists in any public API
- verify no `colored` dependency in any compiler crate

Exit condition:

- one test suite proves end-to-end diagnostic fidelity
- no regression in existing tests

## Architecture After

```
Source → Lexer (LexerError) → Parser → Vec<Diagnostic>
                                ↓
Package (PackageError) → .to_diagnostic() → Diagnostic
Resolver (ResolverError) → .to_diagnostic() → Diagnostic
Typecheck (TypecheckError) → .to_diagnostic() → Diagnostic
Lower (LoweringError) → .to_diagnostic() → Diagnostic
                                ↓
                        DiagnosticReport
                                ↓
                 ┌──────────────┼──────────────┐
                 │              │              │
          CLI Plain      CLI Colored      LSP Adapter
        (fol-diagnostics) (fol-frontend)  (fol-diagnostics)
```

No `Glitch` trait. No `Box<dyn Glitch>`. No downcasting. No colors in libraries.
Every stage owns its concrete error type and converts to `Diagnostic` at its
own boundary. The frontend decides presentation style.

## What This Plan Does Not Cover

- `BackendError` and `RuntimeError` — execution-layer errors, not compiler
  diagnostics. They stay as their own types.
- `EditorError` — transport/session error, not a diagnostic.
- Tree-sitter maintenance and language facts — covered by completed work,
  not reopened.

## Immediate Next Slice

Start with Slice 1 (strip colors). It is the simplest, most independent change
and immediately improves library hygiene.
