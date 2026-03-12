# FOL Front-End Follow-Up Plan

Last rebuilt: 2026-03-12
Scope: `fol-stream`, `fol-lexer`, `fol-parser`, and front-end contract docs only

## 0. Current Position

- `make build` passes.
- `make test` passes.
- Current observed totals: `1` unit test and `1219` integration tests, all green.
- The planned stream/lexer/parser follow-up work from this front-end hardening cycle is complete.

## 1. Completed Items

### 3.1 Stream: Replace Regex-Based `.mod` Detection

- Done.
- `.mod` directories are skipped through direct suffix handling.
- The old regex dependency for that path was removed and the lockfile was synced.

### 3.2 Lexer: Freeze Slash-Comment Policy

- Done.
- Backticks remain the authoritative comment spelling.
- Slash line and slash block comments are now explicitly frozen as compatibility syntax in code, tests, and docs.

### 3.3 Lexer + Parser: Preserve Comments And Doc Comments Past Lexing

- Done for the current front-end contract.
- Comment and doc-comment kinds survive lexing as parser-visible tokens.
- Standalone root-level and routine-body comments are retained as `AstNode::Comment { kind, raw }`.
- Raw comment spelling is preserved for future doc-comment parsing work.
- The current contract intentionally keeps richer inline-trivia attachment out of scope for this phase.

### 3.4 Parser: Clarify The Mixed-Root Program Carrier

- Done.
- `AstNode::Program { declarations }` is explicitly documented as an intentionally mixed, source-ordered root surface.

### 3.5 Parser: Decide The Long-Term `use` Path Storage Shape

- Done.
- `UseDecl.path` remains the raw accepted spelling.
- `UseDecl.path_segments` remains the structured representation for later import work.
- The dual-field contract is now documented as intentional.

### 3.6 Parser: Quarantine Or Relocate `AstNode::get_type()`

- Done.
- The real parser-era helper is now `syntactic_type_hint()`.
- `get_type()` remains only as a compatibility shim, explicitly documented as non-semantic.

### 4.1 Stream: Remove Per-File `chars().collect()` Duplication

- Done.
- `FileStream` now precomputes one reusable char buffer per source during construction.
- Source switches no longer rebuild per-file char buffers while draining the stream.

### 4.2 Docs: Tighten README Front-End Precision

- Done.
- `README.md` now points readers to `FRONTEND_CONTRACT.md` as the authoritative front-end contract.

## 2. Remaining In-Scope Work

- None required before moving past stream + lexer + parser.

## 3. Notes

- Later semantic analysis, type checking, ownership, runtime, and backend work remain out of scope for this plan.
- If future documentation tooling needs richer comment attachment than standalone root/body comment nodes, that can be modeled as a later parser/AST extension rather than a blocker for the completed front-end hardening pass.
