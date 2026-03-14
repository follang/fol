# FOL Project Progress

Last scan: 2026-03-14
Scan basis: repository code, active tests, current docs, and a fresh `make build` + `make test`
Authority rule for this file: code and active tests win over older docs, plans, and historical assumptions

## 0. Purpose

- This file answers one question: what is actually implemented right now.
- This file is a repo-backed status ledger for the current workspace head.
- For the current phase, the priority is repository truth: stream, lexer, parser,
  package loading, resolver, type checking, diagnostics, CLI behavior, and the
  current V1 compiler boundary.
- This file does not plan later semantic or backend work.

## 1. Scan Method

- Scanned the active workspace manifests and source inventory.
- Scanned all active Rust modules under:
- `fol-types`
- `fol-stream`
- `fol-lexer`
- `fol-parser`
- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-diagnostics`
- `src`
- Scanned active tests under:
- `test/stream`
- `test/lexer`
- `test/parser`
- `test/resolver`
- `test/typecheck`
- `test/run_tests.rs`
- Rescanned current user-facing docs:
- `README.md`
- `VERSIONS.md`
- relevant `book/src` pages for lexical rules, methods, literals, and recoverable errors
- Rechecked the current implementation against the existing progress ledger and the
  active V1 typecheck milestone record.
- Ran:
- `make build`
- `make test`

## 2. Snapshot Metrics

- Workspace member crates: `8`
- Root binary crate: `1`
- Active Rust source lines scanned: `32648`
- Core compiler Rust lines scanned:
- `fol-types`: `259`
- `fol-stream`: `570`
- `fol-lexer`: `2406`
- `fol-parser`: `15857`
- `fol-package`: `2779`
- `fol-resolver`: `4542`
- `fol-typecheck`: `3931`
- `fol-diagnostics`: `1206`
- Root CLI and root-local source: `397`
- Active parser fixtures: `1283`
- Active lexer tests: `85`
- Active stream tests: `54`
- Parser-focused Rust tests under `test/parser`: `1108`
- Resolver-focused Rust tests under `test/resolver`: `100`
- Typecheck-focused Rust tests under `test/typecheck`: `67`
- Observed current unit test run: `5` unit tests, green
- Observed current integration run: `1446` integration tests, green

## 3. Current Headline Status

- `fol-stream`: implemented, actively used, and now explicit about failure and namespace validity
- `fol-lexer`: implemented, actively used, and materially hardened on malformed-input and helper-path behavior
- `fol-parser`: large front-end surface implemented, heavily hardened, and now much closer to a stable AST contract
- `fol-package`: implemented as the package-loading and package-definition boundary before resolver
- `fol-resolver`: implemented for the current whole-program name-resolution contract
- `fol-typecheck`: implemented for a large local `V1` semantic subset and wired into the CLI
- `fol-diagnostics`: implemented, structured, and wired into the CLI
- Root CLI: implemented as parse-and-package-prepare-and-resolve-and-typecheck driver
- Stream + lexer + parser: stable and consumed by package loading and resolver
- Package loading and package preparation: implemented for `loc`, `std`, and installed `pkg`
- Whole-program name resolution: implemented for the current contract
- Whole-program type checking: substantial, but still reopened before it can be called the full `V1` typechecker
- Immediate active phase: finish the reopened `V1` typechecker across imported package graphs
- Ownership and borrowing enforcement: missing
- Standard or protocol conformance analysis: missing
- Backend, interpreter, or code generation: missing
- Runtime semantics: missing

## 4. Validation Baseline

- `make build`: passed
- `make test`: passed
- Current observed totals:
- `5` unit tests passed
- `1446` integration tests passed
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

- The earlier front-end contract sync work was completed during the stream/lexer/parser hardening phase.
- `README.md` and `PROGRESS.md` now describe resolver and typechecking as implemented stages instead of future phases.
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
- `use std` imports resolve against explicit configured std roots.
- `use pkg` imports resolve against explicit configured package-store roots.
- Installed `pkg` roots now require both `package.yaml` and `build.fol`.
- Stray `package.fol` files are ignored during `pkg` loading and do not satisfy package metadata requirements.
- `package.yaml` is metadata-only and is not part of ordinary package source loading.
- `build.fol` defines pkg dependency and export records with `def` and is not part of
  ordinary package source loading.
- Consumer-visible `pkg` imports now mount only the roots and namespaces exported by
  `build.fol`, instead of exposing every exported symbol under the package root.
- Unsupported import kinds fail explicitly instead of silently degrading.
- Imported exported values, routines, and named types are now visible through plain
  lookup after supported imports instead of requiring explicit qualification only.
- Qualified alias-root resolution now works even when the local alias spelling does
  not match the imported namespace root spelling.
- File-private `hid` declarations now resolve everywhere inside their own source
  unit where ordinary scope rules allow them, and still fail outside that file.
- Built-in `str` now exits named-type lookup instead of surfacing as an unresolved
  user-defined symbol.

### 5.6 Diagnostics Milestone

- `fol-diagnostics` now models structured diagnostics with:
- one primary label
- zero or more secondary labels
- notes
- helps
- suggestions
- Human diagnostics now render source snippets, underline primary spans, surface
  related labels as note-style entries, and fall back cleanly when source text
  cannot be loaded.
- JSON diagnostics now preserve the richer structure directly, including primary
  location, secondary labels, notes, helps, and suggestions.
- Parser, package, and resolver now lower into stable producer-owned diagnostic
  codes instead of relying on message-derived fallback guessing.
- The CLI now routes compiler glitches through one shared lowering boundary
  instead of keeping per-producer downcast ladders in the entrypoint.
- Duplicate, ambiguity, and duplicate-package-field diagnostics now preserve
  related sites structurally instead of only embedding them in prose.
- Warning and info report paths are now first-class and renderer-tested even
  though current compiler producers still emit mostly errors.
- Resolver diagnostics now retain exact locations where the parser exposes them,
  including plain unresolved identifiers, plain free calls, plain named types,
  and structured competing-declaration sites.

### 5.7 Package Loading Milestone

- `fol-package` now exists as a workspace crate and sits between parser output and
  resolver package consumption.
- `fol-package` owns `package.yaml` metadata parsing.
- `fol-package` owns `build.fol` extraction for dependency, export, and inert
  native-artifact placeholder records.
- `fol-package` owns package-session caching, cycle detection, shared dependency
  dedupe, and directory/store loading.
- Entry packages are now prepared through `fol-package` before resolution instead
  of being handed directly from parser output into the resolver.
- `loc` and `std` imports resolve as exact directories through `fol-package`.
- `pkg` imports resolve as installed package roots with required `package.yaml`
  plus `build.fol`, while stray `package.fol` files stay ignored.
- Control files remain excluded from ordinary package source parsing.
- `build.fol` export declarations are now lowered into concrete prepared export
  mounts before resolver namespace mounting.
- Installed-store dependency strings are now represented as `PackageLocator`
  records instead of opaque raw strings.
- Future remote or git-like locators now fail with explicit placeholder
  diagnostics instead of silently looking like valid installed-store paths.
- Reserved native-artifact definitions such as `header`, `object`, `static_lib`,
  and `shared_lib` are preserved as inert package-build records for future C ABI
  work and are not active resolver semantics today.
- The CLI now treats parse-clean but resolution-bad programs as failing compiles.
- The CLI now accepts both `--std-root` and `--package-store-root` so the current
  `loc/std/pkg` resolver contract is available end to end.
- Recursive `pkg` dependencies now load through `build.fol` dependency records, and
  repeated shared package roots are deduped through canonical package identity.
- Integration coverage now includes full happy-path resolution, cross-file import
  resolution, exact resolver-location propagation through JSON diagnostics, and
  non-null location guarantees for plain unresolved and ambiguous name cases.

### 5.8 Typecheck Milestone

- `fol-typecheck` now exists as a workspace crate and is wired into the root CLI
  after resolver.
- The typechecker installs canonical builtin `V1` scalar types and interns
  normalized semantic type shapes.
- `ResolvedProgram` now lowers into a typed shell with typed symbol, reference,
  and syntax-node facts.
- Declaration signatures are checked across:
- bindings
- aliases
- record and entry members
- routine parameters
- routine returns
- routine error types
- cross-file and qualified named-type references
- Core expression typing now covers:
- literals
- plain and qualified identifiers
- block/final-body expressions
- assignments
- free calls
- method calls
- field access
- index access
- basic slice access
- Routine/control typing now covers:
- `return`
- `report`
- `when` result agreement
- loop guard basics
- `never`-aware early-exit handling for `panic`, `return`, and `report`
- Aggregate typing now covers:
- array / vector / sequence literals
- set / map literals
- record construction
- entry value surfaces
- optional and error shell compatibility at the currently implemented `V1` surfaces
- The initial `V1` operator, coercion, and cast contract is now explicit and
  test-backed.
- `V2` and `V3` surfaces now fail explicitly during typechecking instead of
  silently passing unchecked.
- Ordinary typechecking now rejects `build.fol` package-definition files as out
  of scope for source-program semantics.
- CLI integration coverage now includes successful typechecked compiles, human
  typecheck failures, and exact JSON typecheck diagnostics.
- The reopened current blockers are:
- imported exported symbols from `loc`, `std`, and `pkg` are still not fully
  typed end to end
- imported method lookup still needs graph-aware parity
- So the current active limitation is no longer parser or resolver coverage; it
  is finishing the full `V1` typechecker boundary.

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

- No material contradiction was found in the scanned active repo docs after the latest doc sync.
- `README.md` still speaks at a high level rather than pinning exact compiler contracts, which is acceptable because `PROGRESS.md`, `VERSIONS.md`, and the test suite carry the exact repo-backed detail.

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

## 9. What Is Explicitly Out Of Scope For The Current V1 Milestone

- `V2` language semantics such as standards, protocols, blueprints, extensions, and generics
- `V3` systems semantics such as ownership, borrowing, eventuals, coroutines, channels, pointers, and C ABI
- runtime behavior
- interpreter or backend
- code generation
- optimization

These remain later-stage work. They are no longer reasons to keep the current
`V1` stream/parser/package/resolver/typecheck milestone open.

## 10. Current Readiness Call

### 10.1 What Is Ready

- The project has a real front-end pipeline:
- `fol-stream -> fol-lexer -> fol-parser -> fol-package -> fol-resolver -> fol-typecheck`
- `fol-diagnostics` now sits alongside that pipeline as the shared reporting layer.
- The pipeline is not toy-only anymore.
- Stream, lexer, parser, package loading, resolver, diagnostics, and the current
  `V1` typechecker behavior are explicit enough to move to later `V1` compiler
  stages without another deep stability pass first.
- Current validation is green and large enough to trust ordinary refactors much more than before.

### 10.2 What Is Not Implemented Yet

- Full-language semantic analysis is still missing.
- Later `V1` compiler stages after typechecking are still missing.
- Runtime or backend behavior is still missing.

### 10.3 Bottom-Line Status

- Stream: strong and contract-stable
- Lexer: strong and contract-stable
- Parser: broad, hardened, source-layout-aware, and contract-stable enough to move on
- Package loading: implemented and broad enough for the current `loc/std/pkg` contract
- Resolver: implemented and broad enough for the current name-resolution milestone
- Typechecker: implemented for the current `V1` semantic subset and enforced through the CLI
- Diagnostics: structured, stable, and contract-backed
- Current compiler core: ready to move beyond the post-resolution semantic boundary and into later `V1` compiler stages

## 11. Next Recommended Focus

- Treat the current `fol-typecheck` milestone as real compiler infrastructure,
  not as a placeholder crate.
- Keep the next work inside `V1`: later semantic and lowering stages that can
  eventually carry a `V1` program toward a binary-producing pipeline.
- Treat diagnostics as infrastructure-complete for parser/package/resolver/typecheck
  and extend it only when later compiler stages need richer producer lowering.
- Treat any remaining stream/lexer/parser/package/resolver/typecheck work as
  opportunistic cleanup unless a real new bug appears.
- Use [`PROGRESS.md`](./PROGRESS.md), [`VERSIONS.md`](./VERSIONS.md),
  [`PLAN.md`](./PLAN.md), and the test suite as the frozen reference point for
  the next stage.
