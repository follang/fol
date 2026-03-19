# FOL Diagnostics Overhaul Plan

Last updated: 2026-03-19

## Problems Found

1. **Error cascade** — A single parse error (e.g. `fun[exp]` instead of `fun(exp)`)
   produces 20+ "Expected declaration at file root" errors, one per remaining token.
   The parser has no sync-to-next-declaration after a failed declaration parse.

2. **No dedup anywhere** — DiagnosticReport, LSP, and CLI all pass every error through
   without deduplication. Same message at adjacent tokens = wall of identical errors.

3. **Ugly error message prefixes** — Display impl prepends "ResolverUnresolvedName: ",
   "TypecheckIncompatibleType: " etc. to every message. Redundant since the diagnostic
   code (R1003, T1003) already identifies the kind. Users see:
   `error: ResolverUnresolvedName: could not resolve 'x'` instead of
   `error[R1003]: could not resolve 'x'`

4. **Error classification is message-based** — `ParseErrorKind::classify()` does
   substring matching on error text ("at file root" → FileRoot). Fragile — any message
   change silently changes the diagnostic code.

5. **33 locationless parse errors** — Safety-bound-exceeded and constraint errors use
   `file: None, line: 0, column: 0`. These produce diagnostics with no navigable location.

6. **No error limit** — 100 errors = 100 errors displayed. No "...and N more" cutoff.

7. **Diagnostic code not shown in human output** — The rendered output shows
   `error: message` but never shows the code (P1001, R1003, etc.). Users can't look
   up error codes.

## Slices

### Slice 1 — Parser error recovery (the cascade fix)

After a declaration parse fails, skip tokens to the next declaration keyword or
newline-then-keyword, instead of falling through token-by-token to the catch-all.

- [ ] 1a. Add `sync_to_next_declaration(tokens)` method — skip tokens until the next
      declaration-start keyword (`fun`, `var`, `def`, `typ`, `pro`, `log`, `seg`,
      `ali`, `imp`, `lab`, `con`, `use`) or EOF. Stop BEFORE the keyword (don't consume it).
      (`program_parsing.rs`)
- [ ] 1b. In every `Err(error)` arm of declaration parsing in
      `parse_top_level_entries_with_surface`, after pushing the error, call
      `sync_to_next_declaration(tokens)` instead of relying on `bump_if_no_progress`
- [ ] 1c. Test: `fun[exp] emit(...) = { ... }` → exactly 1 error (P1001), not 20+
- [ ] 1d. Test: two broken declarations separated by a good one → 2 errors, middle
      declaration still parsed correctly

### Slice 2 — Cascade suppression in DiagnosticReport

Safety net for all pipeline stages. Even with parser recovery, edge cases can cascade.

- [ ] 2a. In `DiagnosticReport::add_diagnostic()`, skip if the new diagnostic has
      the same code AND same line as the most recently added diagnostic
      (`fol-diagnostics/src/lib.rs`)
- [ ] 2b. Add max error limit: after 20 errors of the same code, suppress further
      errors of that code and append a "...and N more" info diagnostic at report output
- [ ] 2c. Tests: same code/same line → 1 diagnostic; different lines → 2; limit → capped

### Slice 3 — Clean error messages (remove kind prefixes)

Error messages should be the human-readable message, not "ResolverUnresolvedName: msg".

- [ ] 3a. Change `ResolverError::Display` to output just `self.message` instead of
      `"{kind_label}: {message}"` (`fol-resolver/src/errors.rs:104-107`)
- [ ] 3b. Same for `TypecheckError::Display` (`fol-typecheck/src/errors.rs`)
- [ ] 3c. Same for `PackageError::Display` (`fol-package/src/errors.rs`)
- [ ] 3d. Same for `LoweringError::Display` (`fol-lower/src/errors.rs`)
- [ ] 3e. Same for `BuildEvaluationError::Display` (`fol-build/src/eval/error.rs`)
- [ ] 3f. Update any tests that assert on the prefixed format

### Slice 4 — Show diagnostic codes in human output

Users should see `error[R1003]:` not just `error:` so they can look up codes.

- [ ] 4a. In `render_human.rs`, change `"{prefix}: {message}"` to
      `"{prefix}[{code}]: {message}"` when code is not EUNKNOWN
- [ ] 4b. Update render tests to expect the new format

### Slice 5 — Structural ParseErrorKind (replace substring matching)

Replace the fragile `ParseErrorKind::classify(message)` with a field on `ParseError`.

- [ ] 5a. Add `kind: ParseErrorKind` field to `ParseError` struct
- [ ] 5b. Add `ParseError::from_token_with_kind(token, kind, message)` constructor
- [ ] 5c. Update `ParseError::from_token()` to set kind from an explicit parameter
      or default to `Syntax`
- [ ] 5d. At each error creation site in parser_parts/, set the correct kind directly
      instead of relying on message text classification
- [ ] 5e. Remove `ParseErrorKind::classify()` — no longer needed
- [ ] 5f. Update `to_diagnostic()` to use `self.kind` directly

### Slice 6 — Fix locationless parse errors

Fix the 33 `ParseError` constructions that use `file: None, line: 0`.

- [ ] 6a. For safety-bound-exceeded errors (routine_header_parsers, binding_declaration_parsers,
      rolling_expression_parsers, etc.), pass `tokens` and use `tokens.curr(false)` or
      the last valid token for location
- [ ] 6b. For constraint errors (duplicate parameter names, invalid patterns), use the
      offending token's location
- [ ] 6c. Test: every parse error has `line > 0` and `file.is_some()` when parsing a
      real file

### Slice 7 — LSP diagnostic improvements

- [ ] 7a. Deduplicate LSP diagnostics by (line, code) before sending to editor
      (`lsp/analysis.rs`)
- [ ] 7b. Include diagnostic code in LSP diagnostic message so editors show it
- [ ] 7c. Test: parse cascade → at most 1 LSP diagnostic per line per code
