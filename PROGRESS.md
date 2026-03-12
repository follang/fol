# FOL Project Progress

Last scan: 2026-03-13
Scan basis: repository code, active tests, current docs, and a fresh `make build` + `make test`
Authority rule for this file: code and active tests win over older docs, plans, and historical assumptions

## 0. Purpose

- This file answers one question: what is actually implemented right now.
- This file is a repo-backed status ledger for the current workspace head.
- For the current phase, the priority is repository truth: stream, lexer, parser,
  resolver, diagnostics, CLI behavior, and the handoff boundary into post-resolution
  semantic work.
- This file does not plan later semantic or backend work.

## 1. Scan Method

- Scanned the active workspace manifests and source inventory.
- Scanned all active Rust modules under:
- `fol-types`
- `fol-stream`
- `fol-lexer`
- `fol-parser`
- `fol-resolver`
- `fol-diagnostics`
- `src`
- Scanned active tests under:
- `test/stream`
- `test/lexer`
- `test/parser`
- `test/resolver`
- `test/run_tests.rs`
- Rescanned current front-end contract and user-facing docs:
- `FRONTEND_CONTRACT.md`
- `README.md`
- relevant `book/src` pages for lexical rules, methods, literals, and recoverable errors
- Rechecked the current implementation against the existing progress ledger and the
  active resolver milestone record.
- Ran:
- `make build`
- `make test`

## 2. Snapshot Metrics

- Workspace member crates: `6`
- Root binary crate: `1`
- Active Rust source lines scanned: `22339`
- Core compiler Rust lines scanned:
- `fol-types`: `259`
- `fol-stream`: `570`
- `fol-lexer`: `2406`
- `fol-parser`: `15771`
- `fol-resolver`: `2779`
- `fol-diagnostics`: `267`
- Root CLI and root-local source: `178`
- Active parser fixtures: `1281`
- Active lexer tests: `85`
- Active stream tests: `54`
- Parser-focused Rust tests under `test/parser`: `1101`
- Resolver-focused Rust tests under `test/resolver`: `59`
- Observed current unit test run: `1` unit test, green
- Observed current integration run: `1313` integration tests, green

## 3. Current Headline Status

- `fol-stream`: implemented, actively used, and now explicit about failure and namespace validity
- `fol-lexer`: implemented, actively used, and materially hardened on malformed-input and helper-path behavior
- `fol-parser`: large front-end surface implemented, heavily hardened, and now much closer to a stable AST contract
- `fol-resolver`: implemented for the current whole-program name-resolution milestone
- `fol-diagnostics`: implemented and wired into the CLI
- Root CLI: implemented as parse-and-resolve driver
- Stream + lexer + parser: stable and consumed by resolver
- Whole-program name resolution: implemented for the current milestone
- Immediate active phase: post-resolution semantic analysis and type checking
- Whole-program type checking: missing
- Ownership and borrowing enforcement: missing
- Standard or protocol conformance analysis: missing
- Backend, interpreter, or code generation: missing
- Runtime semantics: missing

## 4. Validation Baseline

- `make build`: passed
- `make test`: passed
- Current observed totals:
- `1` unit test passed
- `1313` integration tests passed
- Observed active failures: `0`

## 5. What Has Been Completed So Far

### 5.1 Stream Hardening

- Folder traversal order is deterministic.
- `.mod` directories are skipped during source collection.
- Entry-root package detection replaced earlier Cargo-manifest dependence.
- Canonical source identity is separated from raw call-site spelling.
- Detached file and detached folder package fallback rules are explicit and tested.
- File boundary location resets are explicit and tested.
- Single-file root-namespace behavior is explicit and tested.
- Nested namespace derivation is explicit and tested.
- Package names are validated instead of being accepted blindly.
- Invalid namespace path components are rejected instead of being silently dropped.
- Recursive directory traversal failures are surfaced instead of being ignored.
- `fol_stream::sources(...)` now propagates initialization failure instead of returning an empty source set.
- Cross-file ordering survives into the lexer instead of being merged accidentally.
- Eager source loading is now an intentional contract instead of an accidental behavior.

### 5.2 Lexer Hardening

- Stage 0 no longer collects one giant whole-stream character vector before lexing.
- Explicit cross-file boundary markers replaced synthetic fake newlines.
- Boundary tokens preserve incoming-file identity and location.
- Token payload contracts are explicit and test-backed.
- Comments are explicitly classified internally as:
- backtick
- doc
- slash-line
- slash-block
- Backticks are the authoritative comment form.
- Slash comments remain explicit compatibility behavior.
- Cooked and raw quoted literal families are distinct at the lexer boundary.
- Numeric family support is explicit and hardened:
- decimal
- float
- hexadecimal
- octal
- binary
- Leading-dot and trailing-dot floats are supported and tested.
- Malformed numeric literals converge on explicit illegal-token behavior.
- Unterminated quoted literals and unterminated block-comment forms surface as `Illegal`.
- Unsupported non-ASCII characters and unsupported ASCII control characters still hard-fail lexing.
- Repeated underscore runs inside identifiers lower to `Illegal`.
- Keyword recognition is exact-case only.
- Stage 2 and stage 3 reverse-look-behind helpers now use the correct previous-window side when ignoring whitespace.
- Stage 2 and stage 3 recovery-style `jump()` paths no longer panic after stream drain.
- `Location::visualize()` now degrades gracefully when source files or rows are unavailable.

### 5.3 Parser Hardening

- Top-level routine body leakage into `Program.declarations` was removed.
- `log` declarations now survive as `AstNode::LogDecl` instead of being collapsed into `FunDecl`.
- Named routine receiver types are retained in the AST.
- Anonymous logical parsing is explicit instead of being routed through function-only internals.
- Quoted names now remove only matching outer delimiters instead of over-trimming.
- Qualified value, call, type, and inquiry paths survive as structured `QualifiedPath`.
- Parser-owned duplicate checks were hardened to canonical identifier comparison across audited surfaces.
- Illegal-token routing was broadened across many parser-owned contexts so malformed tokens fail at the offending token instead of drifting into generic separator or `Expected X` errors.
- File-boundary tokens are now parser-visible hard separators instead of ignorable whitespace.
- Cross-file continuation of declarations, routine headers, and `use` paths is rejected.
- `return` is rejected outside routine contexts.
- `break` is rejected outside loop contexts.
- `yeild` is rejected at file scope and outside routine contexts.
- Numeric overflow no longer falls back to `AstNode::Identifier`.
- Oversized decimal and prefixed numeric literals now fail explicitly at parse time.
- `use` declarations now preserve structured path segments instead of only opaque flattened text.
- Empty `use` path segments now report dedicated separator-focused diagnostics.
- Method receiver diagnostics now match the actual parser contract.
- The stale report-era literal-lowering module name was removed.
- The parser now exposes `parse_package(...)` as a structured package-aware entry point.
- Successful top-level parse nodes now retain syntax origins through parser-owned IDs and a syntax index.
- Parser output can now preserve first-class source units with per-file path, package, namespace, and ordered items.
- Parsed top-level items now also retain explicit declaration visibility/scope metadata for resolver-owned scope building.
- Comments and doc comments now remain AST-visible beyond standalone root/body sibling nodes.
- Inline expression, postfix, call-argument, and container-element comments are preserved through `AstNode::Commented { leading_comments, node, trailing_comments }`.
- Cross-file boundary failures are now locked with exact-location tests on both the synthetic boundary token (`column 0`) and the first real token of the incoming file (`column 1`).
- Parser failure-shape coverage is much broader:
- unknown options
- conflicting options
- unsupported declaration-family combinations
- missing-close diagnostics
- representative `Expected X` diagnostics
- numeric overflow diagnostics
- invalid control-flow placement diagnostics
- cross-file continuation diagnostics

### 5.4 Contract And Docs Cleanup

- `FRONTEND_CONTRACT.md` now matches the hardened stream, lexer, and parser behavior much more closely.
- `README.md`, `FRONTEND_CONTRACT.md`, and `PROGRESS.md` now describe resolver as an implemented milestone instead of a future phase.
- The lexical and routine book pages scanned in this pass are aligned with the current front-end behavior.
- Parser-side `report` type checking and forward-signature validation are no longer described as active behavior.
- Method receiver docs now match the current parse-time rejection rule.
- The front-end source-layout alignment plan has now been executed in code and tests for the stream/lexer/parser scope.

### 5.5 Resolver Milestone

- `fol-resolver` now exists as a workspace crate and is wired into the root CLI.
- Resolver-owned IDs and tables are explicit for source units, scopes, symbols,
  references, and imports.
- The resolver builds program, namespace, source-unit, routine, block, loop-binder,
  and rolling-binder scopes.
- Top-level declarations are collected across package, namespace, and file scopes with
  explicit duplicate handling.
- Plain identifiers and free calls resolve through lexical scope chains.
- Qualified identifiers, qualified calls, qualified type names, and inquiry targets
  resolve through namespace and import-alias roots.
- `use loc` imports resolve against the loaded package and namespace scope set.
- Unsupported import kinds fail explicitly instead of silently degrading.
- Resolver diagnostics now retain exact locations where the parser exposes them and
  report competing declaration or candidate sites where useful.
- The CLI now treats parse-clean but resolution-bad programs as failing compiles.
- Integration coverage now includes full happy-path resolution, cross-file import
  resolution, and exact resolver-location propagation through JSON diagnostics.

## 6. Current Front-End State By Layer

### 6.1 fol-stream

Status:
- implemented
- deterministic
- ready for later consumers

What is solid now:
- source discovery for single-file and folder entry
- lexicographic directory traversal
- `.fol` filtering
- `.mod` skipping
- entry-root package detection
- explicit package override validation
- namespace derivation
- namespace-component validation
- source identity separation between canonical identity and raw call spelling
- per-character location tracking
- file-boundary reset behavior
- explicit failure propagation during discovery and initialization

What is still true in code today:
- `FileStream::from_sources` eagerly reads every file body into memory before lexing begins

Risk call:
- Low for correctness on covered behavior
- Low for cleanliness and future maintainability

### 6.2 fol-lexer

Status:
- implemented
- broadly hardened
- stable enough to stop front-end rescanning

What is solid now:
- stage ownership is documented and mostly reflected in code
- cross-file boundaries are explicit
- cooked/raw quote families are explicit
- comment handling is explicit
- numeric-family coverage is explicit
- malformed quoted/comment/numeric spans are consistent
- exact-case keyword recognition is explicit
- unsupported-character hard errors are explicit
- reverse-look-behind ignore helpers are corrected
- drain-path recovery helpers no longer panic
- source-visualization fallbacks no longer crash diagnostics

What is still true in code today:
- slash comments are still supported even though backticks are the authoritative comment form
- raw single-quoted spans still stop at the next single quote even after a backslash, by explicit current contract
- imaginary literal forms are still out of scope

Risk call:
- Low on current covered behavior
- Low-to-medium only if the language intentionally reopens comment or quote contracts later

### 6.3 fol-parser

Status:
- implemented
- large grammar surface
- materially closer to a stable AST boundary

What is solid now:
- broad declaration support
- broad routine support
- literal lowering for supported families
- root/body separation
- receiver retention
- logical routine identity
- structured qualified paths
- structured `use` path segments
- canonical duplicate checking across audited parser-owned surfaces
- broader illegal-token routing
- hard file-boundary separation
- context-sensitive control-flow acceptance
- explicit numeric overflow rejection
- much stronger diagnostic consistency coverage
- exact cross-file boundary diagnostics on the declaration-oriented package parser path
- explicit parsed top-level visibility/scope metadata for resolver-owned scope building

What is still true in code today:
- the preferred structured path is now `parse_package(...)`, but the legacy `AstNode::Program { declarations }` compatibility path is still intentionally mixed and script-like
- `AstNode::UseDecl` now carries only structured path segments for import spelling
- `AstNode::get_type()` is still a heuristic AST helper that looks semantic-adjacent even though whole-program semantic analysis is not implemented
- the parser still treats many keyword tokens as acceptable label surfaces by design, which is test-backed and now part of the current contract

Risk call:
- Low-to-medium for current covered parsing behavior
- Medium only for future AST cleanup choices, not for current parser correctness

## 7. Current Known Discrepancies

These are not active test failures. They are the remaining front-end compromises that are visible in code today.

### 7.1 Code vs Docs

- No material contradiction was found in the scanned front-end contract pages after the latest doc sync.
- `README.md` still speaks at a high level rather than pinning exact parser contract details, which is acceptable but less precise than `FRONTEND_CONTRACT.md`.

### 7.2 Code vs Long-Term Shape

- Lexer still carries compatibility behavior for slash comments even though backticks are the primary documented comment form.
- Parser still exposes a few AST naming and helper choices that are good enough for this phase but not obviously ideal for later semantic ownership:
- mixed-root `Program.declarations`
- heuristic `AstNode::get_type()`
- comments are now retained much more broadly, but truly universal trivia attachment is still a later AST-shape choice rather than a finished contract item

## 8. Current Front-End Debt Worth Tracking, But Not Blocking

This section is intentionally limited to stream, lexer, and parser. None of these
items block the current post-resolver phase.

### 8.1 Stream Follow-Up

- keep eager loading explicit unless there is a deliberate decision to change the front-end loading model

### 8.2 Lexer Follow-Up

- decide whether slash comments remain permanent compatibility syntax or are retired later
- keep the malformed-input contract narrow and explicit if any new quote or comment behavior is added

### 8.3 Parser Follow-Up

- decide whether `Program.declarations` should remain the long-term mixed-root carrier or be renamed later
- either narrow, relocate, or clearly quarantine heuristic AST helpers such as `AstNode::get_type()`

## 9. What Is Explicitly Out Of Scope For This Phase

- type checking
- ownership analysis
- standard or protocol conformance
- runtime behavior
- interpreter or backend
- code generation
- optimization

These remain later-stage work. They are no longer reasons to keep front-end hardening open.

## 10. Current Readiness Call

### 10.1 What Is Ready

- The project has a real front-end pipeline:
- `fol-stream -> fol-lexer -> fol-parser -> fol-resolver -> fol-diagnostics`
- The pipeline is not toy-only anymore.
- Stream, lexer, parser, and resolver behavior are now explicit enough to move to the
  next semantic phase without another deep stability pass first.
- Current validation is green and large enough to trust ordinary refactors much more than before.

### 10.2 What Is Not Implemented Yet

- Semantic analysis is still missing.
- Type checking is still missing.
- Runtime or backend behavior is still missing.

### 10.3 Bottom-Line Status

- Stream: strong and contract-stable
- Lexer: strong and contract-stable
- Parser: broad, hardened, source-layout-aware, and contract-stable enough to move on
- Resolver: implemented and broad enough for the current name-resolution milestone
- Diagnostics baseline: green
- Current compiler core: ready to move beyond name resolution

## 11. Next Recommended Focus

- Create the next plan around type checking, deeper semantic analysis, and
  type-directed member resolution.
- Treat any remaining stream/lexer/parser/resolver work as opportunistic cleanup unless
  a real new bug appears.
- Use `FRONTEND_CONTRACT.md`, [`PROGRESS.md`](./PROGRESS.md), [`PLAN.md`](./PLAN.md),
  and the test suite as the frozen reference point for the next stage.
