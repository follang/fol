# FOL Front-End Final Hardening Plan

Last rebuilt: 2026-03-11
Scope: only `fol-stream`, `fol-lexer`, `fol-parser`, and the tests/docs that define their contracts

## 0. Verified Baseline

- `make build` passes.
- `make test` passes.
- Current test count at rebuild time: `1` unit test and `1018` integration tests.
- This plan is based on a fresh rescan of `fol-stream/src`, `fol-lexer/src`, `fol-parser/src`, `test/stream`, `test/lexer`, `test/parser`, `book/src`, `FRONTEND_CONTRACT.md`, and `PROGRESS.md`.

## 1. Goal

Make the stream, lexer, and parser authoritative enough that the next stage can start without another structural front-end rescan.

That means:

- source/package/namespace rules are explicit and FOL-native
- lexer behavior matches the chosen spec instead of historical accidents
- parser AST preserves all syntax facts the next stage will need
- parser diagnostics are stable under malformed input
- docs, tests, and implementation stop disagreeing on front-end topics

## 2. Out Of Scope

Do not add or plan work here for:

- whole-program name resolution
- type checking
- ownership or borrow analysis
- runtime behavior
- code generation
- interpreter work
- import resolution beyond front-end surface parsing and source discovery contracts

## 3. What The Rescan Confirmed

### Stream

- Deterministic folder traversal is implemented and tested.
- Source identity is tighter than before, but package naming is still driven by `Cargo.toml` probing in `fol-stream/src/lib.rs`.
- Package discovery is therefore still Rust-host-specific, not FOL-native.
- Namespace derivation still silently drops invalid path components.
- Namespace validation is not aligned to the book’s identifier rules.
- Namespace validation currently accepts Unicode alphanumeric components even though the book’s identifier definition is ASCII-oriented.
- `FileStream::from_sources` still eagerly reads all source text.

### Lexer

- The lexer stages are real and heavily tested.
- Stage 0 still collects the entire character stream into a `Vec`.
- Cross-file token separation is still enforced by injecting a synthetic newline between touching files.
- Comments in the implementation use slash styles (`//`, `/* */`), but the book defines backtick-delimited comments.
- Backticks are still tokenized as `Operator::ANY`, which conflicts with the book’s comment syntax.
- Literal taxonomy is still not aligned to the book: current code treats double quotes as `Stringy`, single quotes as `Quoted`, and then lowers them as string vs single-char/string heuristics, while the book defines cooked double-quoted character/string forms and raw single-quoted character/string forms.
- Escape processing is not implemented; escapes are preserved verbatim.
- Cooked-string line continuation behavior from the book is not implemented.
- Imaginary numeric literals from the book are still out of scope in the code.
- Lexer identifier scanning allows forms the book rejects, especially `_` alone and repeated underscores.
- `KEYWORD::is_void()` still treats `Illegal` as void, which is unsafe for parser-facing behavior.
- Literal enum names no longer carry the old `Deciaml` and `Hexal` misspellings.
- `stage2::make_comment()` is dead code.
- The stage wrapper files still contain `TODO: Handle better .ok()` markers around window mutation.

### Parser

- The parser surface is broad and the root-body leakage issue is already fixed.
- Parser-side `report` semantic validation is gone, which is correct for this phase.
- The parser still discards method receiver types after parsing them.
- `fun (Type)name(...)` and plain `fun name(...)` currently lower to the same AST shape.
- `log` declarations still lower through `AstNode::FunDecl`, so routine kind is lost in the AST.
- Qualified names and paths are still flattened into `::`-joined strings in several AST surfaces.
- Parser-owned duplicate and conflict checks still compare raw strings, not book-defined identifier equality.
- Method receiver rules in the code and the book do not match: the book says user-defined named types only, while the parser currently accepts builtin scalar receiver types.
- The parser still accepts many executable root-level statements and expressions at file scope.
- That file-scope behavior is test-backed, but it is still a major contract decision that later phases will have to live with.

### Docs

- `FRONTEND_CONTRACT.md` is closer to reality than `PROGRESS.md`.
- `PROGRESS.md` still contains stale front-end claims from before the latest hardening work.
- The local book and the code still disagree on several front-end topics: comment syntax, literal quote semantics, escape handling, imaginary literals, identifier rules, and method receiver restrictions.

## 4. Things That Are Not Feasible To Carry Forward

- It is not feasible to keep `Cargo.toml` as the de facto package authority if FOL packages are a language concern.
- It is not feasible to keep backticks as generic `ANY` tokens if the book uses backticks for comments.
- It is not feasible to keep discarding receiver types in the parser AST and still expect the next stage to understand methods.
- It is not feasible to keep lowering `log` through `FunDecl` and still claim routine kind survives parsing.
- It is not feasible to keep flattening all qualified paths into raw strings if later stages need segment-aware diagnostics or resolution.
- It is not feasible to keep treating `Illegal` as `Void` in parser-facing logic.
- It is not feasible to keep raw string equality for parser-owned duplicate checks if the language defines identifier equality differently.
- It is not feasible to keep synthetic in-band newlines as the long-term cross-file boundary model.

## 5. Working Rules

- [ ] Treat the local book as the authority unless we explicitly decide to diverge.
- [ ] If code and docs disagree, close the decision the same day instead of freezing both.
- [ ] Do not add new post-parse behavior while closing this plan.
- [ ] Every behavior change gets a regression test.
- [ ] Every intentional front-end compromise gets written to `FRONTEND_CONTRACT.md`.
- [ ] Every removed compromise also removes the matching stale test assumptions.
- [ ] Prefer preserving syntax facts over recomputing or guessing them later.
- [ ] Do not flatten structured syntax into strings unless there is a strong reason.
- [ ] Do not keep host-tool assumptions in the front-end contract unless they are explicit compatibility layers.

## 6. Implementation Order

- [ ] Phase 0: decision freeze for the remaining front-end ambiguities
- [ ] Phase 1: stream contract cleanup
- [ ] Phase 2: lexer book-alignment and error-model cleanup
- [ ] Phase 3: parser AST-fidelity cleanup
- [ ] Phase 4: docs/test contract freeze

Do not start Phase 2 before Phase 1 decisions that affect source/package/boundary handling are settled.

Do not start Phase 3 before Phase 2 decisions that affect literal and identifier behavior are settled.

Do not declare the front-end ready until Phase 4 is complete.

## 7. Phase 0: Decision Freeze

Target files:

- `PLAN.md`
- `FRONTEND_CONTRACT.md`
- `PROGRESS.md`
- relevant `book/src` pages if we choose to align docs instead of code

### 7.1 Package Authority

- [x] Decide whether package identity is defined by entry root, explicit override, or a future FOL manifest layer.
- [x] Decide whether `Cargo.toml` support stays only as an optional compatibility path.
- [x] Decide whether detached folders and detached files should keep current fallback naming or be made stricter.

### 7.2 Comment Authority

- [ ] Decide whether backtick comments from the book are authoritative.
- [ ] Decide whether slash comments remain temporarily supported during migration or are removed immediately.
- [ ] Decide whether doc comments must survive lexing as recoverable metadata or can remain deferred longer.

### 7.3 Literal Authority

- [ ] Decide whether the book’s cooked-double-quote and raw-single-quote model is authoritative.
- [ ] Decide whether raw-vs-cooked needs to survive in the AST or whether parser lowering can normalize it away.
- [ ] Decide whether imaginary literals must be implemented now or explicitly removed from the front-end scope docs.

### 7.4 Root Surface Authority

- [ ] Decide whether file scope is declaration-only.
- [ ] Decide whether file scope is script-like.
- [ ] If dual-mode is desired, decide how that mode is represented explicitly instead of incidentally.

### 7.5 Method And Routine Authority

- [ ] Decide whether method receiver types are restricted to user-defined named types as the book says.
- [ ] Decide whether `log` becomes a first-class routine kind in the AST.
- [ ] Decide whether qualified paths remain string-encoded temporarily or are promoted now.

Acceptance for Phase 0:

- [ ] Every unresolved front-end ambiguity above has one chosen direction.
- [ ] `FRONTEND_CONTRACT.md` reflects those choices before implementation begins.

## 8. Phase 1: Stream Contract Cleanup

Target files:

- `fol-stream/src/lib.rs`
- `test/stream/test_stream.rs`
- `test/stream/test_namespace.rs`
- `test/stream/test_mod_handling.rs`
- `test/run_tests.rs`

### 8.1 Replace Host-Specific Package Detection

- [x] Remove `Cargo.toml` probing from the default package-name algorithm in `fol-stream/src/lib.rs`.
- [x] If compatibility support for Cargo remains, move it behind an explicit compatibility branch or helper.
- [x] Stop treating Rust project layout as the front-end’s package truth.
- [x] Replace the current `"unknown"` fallback with a deterministic, FOL-defined fallback.
- [x] Preserve the explicit package override behavior.
- [x] Add tests for detached folder, detached file, nested folder, and explicit override package identity.
- [x] Add tests that prove package naming no longer depends on host build files unless compatibility mode is requested.

### 8.2 Tighten Namespace Validation

- [x] Align namespace-component validation with the chosen identifier rules.
- [x] Make namespace validation ASCII-aware if we align it to the book’s identifier grammar.
- [x] Decide whether invalid namespace components are hard errors or collected diagnostics.
- [x] Stop silently dropping invalid namespace components without an observable outcome.
- [x] Add tests for dots, hyphens, leading digits, leading underscore, single underscore, repeated underscores, and mixed-case names.
- [x] Add tests for non-ASCII folder names to prove the chosen policy.

### 8.3 Separate Logical Boundary Handling From Fake Source Characters

- [ ] Remove long-term reliance on synthetic newline injection between adjacent files.
- [ ] Introduce an explicit source-boundary concept instead of pretending a real newline existed.
- [ ] Preserve the guarantee that tokens from adjacent files never merge accidentally.
- [ ] Add cases where one file ends with an identifier and the next begins with an identifier, number, string delimiter, comment delimiter, or operator.
- [ ] Ensure source-boundary handling does not affect user-visible location reporting as if the boundary came from the file.

### 8.4 Revisit The Loading Model

- [x] Decide whether `FileStream` remains a preload-based source set or becomes a truly lazy stream.
- [x] If eager loading remains for now, document that explicitly and stop calling it a stronger streaming guarantee than it is.
- [ ] If lazy loading is introduced, preserve deterministic traversal and file-boundary resets.
- [ ] Remove duplicated whole-input collection across `fol-stream` and lexer stage 0.
- [x] Add one large multi-file regression to cover the chosen loading model.

### 8.5 Preserve Raw And Canonical Identity Separately

- [x] Decide whether source identity needs both display names and canonical comparison keys.
- [x] If identifier canonicalization applies to package and namespace names, add that key explicitly instead of overloading raw strings.
- [x] Keep canonical file path identity separate from presentation strings.
- [x] Add tests covering renamed entry paths, override-driven identity changes, and raw-vs-canonical identity behavior.

Acceptance for Phase 1:

- [x] Package detection is FOL-defined.
- [x] Namespace validation is explicit.
- [ ] Cross-file boundaries are explicit instead of synthetic source text.
- [x] Loading behavior is intentional, documented, and tested.

## 9. Phase 2: Lexer Book-Alignment And Error Cleanup

Target files:

- `fol-lexer/src/token/mod.rs`
- `fol-lexer/src/token/literal/mod.rs`
- `fol-lexer/src/token/help.rs`
- `fol-lexer/src/lexer/stage0/elements.rs`
- `fol-lexer/src/lexer/stage1/element.rs`
- `fol-lexer/src/lexer/stage1/elements.rs`
- `fol-lexer/src/lexer/stage2/element.rs`
- `fol-lexer/src/lexer/stage2/elements.rs`
- `fol-lexer/src/lexer/stage3/element.rs`
- `fol-lexer/src/lexer/stage3/elements.rs`
- `test/lexer/*`
- `test/run_tests.rs`

### 9.1 Enforce The Chosen Identifier Rules

- [ ] Reject `_` as a standalone identifier if the book remains authoritative.
- [x] Reject repeated underscore runs if the book remains authoritative.
- [ ] Decide whether identifier canonicalization belongs in the lexer crate, a shared front-end helper, or parser-side utilities.
- [x] Add a single front-end identifier normalization helper that later parser checks can reuse.
- [x] Decide whether keyword recognition is exact-case only or uses the same canonicalization policy.
- [x] Add lexer fixtures for valid and invalid identifier edges.
- [x] Add tests for case and underscore variants that should or should not be equivalent.

### 9.2 Replace The Current Comment Model

- [ ] Remove the impossible overlap between backticks-as-`ANY` and backticks-as-comments.
- [ ] If the book wins, implement backtick-delimited comment spans.
- [ ] Treat one-line and multiline backtick comments as the same delimited syntax family instead of emulating slash comments.
- [ ] Implement doc-comment detection for the `[doc]` prefix if it remains in scope.
- [ ] Decide whether doc comments become a token family, lexer side channel, or deferred metadata stream.
- [ ] Remove slash comment logic if it is no longer part of the front-end contract.
- [ ] Delete dead `stage2::make_comment()` if comment handling no longer needs it.
- [ ] Add fixtures for normal comments, doc comments, multiline comments, and unterminated comment spans.
- [ ] Add tests proving comment delimiters inside quoted literals do not start comments.

### 9.3 Rebuild Quoted Literal Taxonomy

- [ ] Stop using current delimiter meaning as the long-term literal model.
- [ ] Rebuild token taxonomy around the chosen cooked/raw quote policy.
- [ ] Preserve enough token metadata to distinguish delimiter kind and cooked/raw behavior without reparsing raw text later.
- [ ] Decide whether character-vs-string distinction happens in the lexer or parser.
- [ ] If the book remains authoritative, support cooked double-quoted character/string forms and raw single-quoted character/string forms.
- [ ] Add an explicit policy for one-element vs multi-element lowering.
- [ ] Add an explicit policy for escape processing in cooked literals.
- [ ] Add an explicit policy for raw literals not processing escapes.
- [ ] Add an explicit policy for multiline cooked strings.
- [ ] Implement backslash-line-break continuation and indentation trimming if the book remains authoritative.
- [ ] Add fixtures for cooked char, cooked string, raw char, raw string, multiline cooked string, escaped quote, escaped backslash, and unicode escape spellings.

### 9.4 Finish Numeric Literal Support

- [x] Rename `Deciaml` to `Decimal`.
- [x] Rename `Hexal` to the final chosen spelling.
- [ ] Keep the payload-preserving behavior during the rename.
- [ ] Decide and implement front-end support for imaginary numeric literals from the book.
- [ ] Decide and implement the parser-facing token kind for imaginary literals.
- [ ] Decide how invalid prefixed numerics behave: one illegal token, split tokenization, or immediate lexer error.
- [ ] Add tests for invalid hex, octal, binary, underscore placement, leading-dot float adjacency, and imaginary suffixes.
- [ ] Keep unary minus parser-side.

### 9.5 Clean Up Illegal And Error Semantics

- [x] Remove `Illegal` from `KEYWORD::is_void()`.
- [ ] Audit every parser path that currently assumes `is_void()` also means malformed input.
- [ ] Make malformed quoted spans follow one consistent policy.
- [ ] Make malformed comments follow one consistent policy.
- [ ] Make malformed numeric spans follow one consistent policy.
- [ ] Ensure raw unsupported characters still produce lexer errors instead of silent token conversion.
- [x] Add nested-context fixtures where an illegal token appears inside calls, blocks, type references, and parameter lists.

### 9.6 Remove Stage Wrapper Debt

- [x] Replace `TODO: Handle better .ok()` window mutations with explicit bounded operations in stage 0.
- [x] Do the same in stage 1.
- [x] Do the same in stage 2.
- [x] Do the same in stage 3.
- [x] Remove stale TODO comments once the window semantics are explicit.
- [x] Add a focused regression test for each stage wrapper after the cleanup.

### 9.7 Revisit Stage 0 Collection

- [x] Decide whether stage 0 remains a buffering adapter or becomes truly streaming.
- [x] If buffering remains, document that it is an implementation choice rather than a stream contract.
- [ ] If streaming is introduced, preserve the existing window API guarantees or replace them intentionally.
- [ ] Keep source locations exact across the chosen model.

Acceptance for Phase 2:

- [ ] Comment syntax matches the chosen authority.
- [ ] Literal quote behavior matches the chosen authority.
- [ ] Illegal tokens are no longer treated as whitespace.
- [ ] Identifier rules are explicit and test-backed.
- [ ] Numeric families are complete for the chosen front-end scope.

## 10. Phase 3: Parser AST-Fidelity Cleanup

Target files:

- `fol-parser/src/ast/mod.rs`
- `fol-parser/src/ast/parser.rs`
- `fol-parser/src/ast/parser_parts/program_and_bindings.rs`
- `fol-parser/src/ast/parser_parts/routine_declaration_parsers.rs`
- `fol-parser/src/ast/parser_parts/routine_headers_and_type_lowering.rs`
- `fol-parser/src/ast/parser_parts/primary_expression_parsers.rs`
- `fol-parser/src/ast/parser_parts/statement_parsers.rs`
- `fol-parser/src/ast/parser_parts/use_declaration_parsers.rs`
- `fol-parser/src/ast/parser_parts/type_definition_parsers.rs`
- `fol-parser/src/ast/parser_parts/rolling_expression_parsers.rs`
- relevant parser tests under `test/parser/test_parser_parts`

### 10.1 Settle The File-Scope Root Shape

- [ ] Decide whether `AstNode::Program` contains declarations only.
- [ ] If file-scope executable statements remain supported, stop treating them as incidental declarations.
- [ ] Introduce an explicit root representation for mixed item/statement programs if mixed file scope remains.
- [ ] If declaration-only file scope is chosen, reject top-level calls, loops, conditionals, literals, and bare identifiers with explicit diagnostics.
- [ ] Update `top_level_control_flow_and_calls.rs` and any other root-shape tests to match the chosen contract.
- [ ] Update `FRONTEND_CONTRACT.md` so later phases do not have to infer module shape.

### 10.2 Preserve Method Receiver Types In The AST

- [x] Add receiver type storage to routine declarations or introduce dedicated method declaration nodes.
- [x] Preserve receiver types for `fun` methods.
- [x] Preserve receiver types for `pro` methods.
- [x] Preserve receiver types for `log` methods if `log` remains a distinct routine kind.
- [x] Preserve receiver types for nested type-member routines.
- [x] Add tests that assert the receiver type survives AST lowering instead of only proving the syntax parses.
- [x] Ensure receiver type diagnostics keep correct source spans.
- [x] Ensure quoted, qualified, and bracketed receiver types keep the chosen structure.

### 10.3 Stop Losing Logical Routine Identity

- [x] Stop lowering `log` declarations through `AstNode::FunDecl`.
- [x] Introduce either `LogDecl` or one shared routine node with an explicit kind field.
- [x] Apply the same decision to anonymous logicals so named and anonymous logicals use one consistent representation.
- [x] Add tests asserting logical routine identity survives AST lowering.
- [x] Update docs so no contract still describes `log` as a temporary `FunDecl`.

### 10.4 Replace String-Joined Qualified Paths

- [ ] Introduce a structured path representation for value paths.
- [ ] Introduce a structured path representation for type paths.
- [ ] Decide whether both surfaces share one path node or use separate but aligned types.
- [ ] Preserve original segment spelling for diagnostics.
- [ ] Preserve normalized comparison keys separately if needed.
- [ ] Stop flattening `io::console::writer` into one opaque string where later phases need segments.
- [ ] Keep `use` import source text separate from value/type path nodes.
- [ ] Update tests for qualified value paths, qualified type paths, and quoted path segments.

### 10.5 Apply Book Identifier Equality To Parser-Owned Validations

- [x] Replace raw-string uniqueness checks with canonical identifier comparison for parameter names.
- [x] Do the same for capture names.
- [x] Do the same for generic names.
- [x] Do the same for use-declaration names.
- [x] Do the same for record field names.
- [x] Do the same for entry variant names.
- [x] Do the same for duplicate type-member names.
- [x] Do the same for rolling binding names.
- [x] Do the same for any other parser-owned duplicate set discovered during the audit.
- [x] Keep original spelling in diagnostics even when comparison uses canonical keys.
- [x] Add collision tests such as `value_name` vs `ValueName`, `Foo_Bar` vs `foobar`, and `A__B` if repeated underscores remain invalid.

### 10.6 Align Literal And Quoted-Name Lowering With The New Lexer Model

- [x] Replace `trim_matches`-based name lowering with a dedicated unquote helper that matches the final lexer contract.
- [ ] Lower cooked character/string literals according to the chosen literal authority.
- [ ] Lower raw character/string literals according to the chosen literal authority.
- [ ] Implement imaginary literal lowering if Phase 2 adds imaginary token support.
- [ ] Decide whether raw-vs-cooked must survive in the AST or can be normalized away after value lowering.
- [x] Add tests for quoted names using both quote families if both remain valid name surfaces.

### 10.7 Tighten Parser Error Surfaces After The Lexer Cleanup

- [ ] Add a nested illegal-token diagnostic matrix after `Illegal` and `Void` are separated.
- [ ] Ensure malformed tokens are reported at the offending token instead of a following separator.
- [ ] Re-run representative `Expected X` and `Expected closing ...` matrices after AST and lexer changes.
- [ ] Keep unsupported combination failures explicit.
- [ ] Remove any tests that only preserve known-bad legacy parser compromises.

Acceptance for Phase 3:

- [x] No method receiver information is lost in the AST.
- [x] No logical routine kind is lost in the AST.
- [ ] Qualified path structure is preserved instead of flattened away where it matters.
- [ ] Parser-owned duplicate checks use the chosen identifier equality rules.
- [ ] Root shape is explicit and no longer ambiguous to the next stage.

## 11. Phase 4: Docs And Contract Freeze

Target files:

- `FRONTEND_CONTRACT.md`
- `PROGRESS.md`
- `PLAN.md`
- affected `book/src` pages

### 11.1 Keep Docs Aligned To The Final Front-End

- [ ] Rewrite `FRONTEND_CONTRACT.md` sections that still freeze current compromises we intentionally remove.
- [ ] Rewrite `PROGRESS.md` to reflect the actual front-end state after the code changes.
- [ ] Update book pages if we align code to the book and some internal docs still describe old behavior.
- [ ] If we intentionally diverge from the book on any front-end point, record the divergence explicitly instead of letting drift accumulate again.

### 11.2 Lock Final Test And Build State

- [ ] Run `make build`.
- [ ] Run `make test`.
- [ ] Confirm the final stream, lexer, and parser test totals and record them in `PROGRESS.md`.
- [ ] Confirm no test is still freezing an abandoned compromise.

## 12. Suggested Slice Sequence

This is the recommended implementation order once the decision freeze is complete.

1. [ ] Freeze the comment, literal, receiver, package, and root-surface decisions in `FRONTEND_CONTRACT.md`.
2. [ ] Introduce one shared identifier canonicalization helper and test it directly.
3. [x] Apply the canonicalization helper to parser duplicate checks before changing more AST shape.
4. [x] Preserve method receiver types in the AST and add direct retention tests.
5. [x] Introduce explicit logical routine kind in the AST and migrate `log` lowering.
6. [ ] Decide and implement the final root surface contract.
7. [ ] Replace backtick `ANY` behavior with the chosen comment model.
8. [ ] Remove slash-comment dependence or make it explicit compatibility behavior.
9. [ ] Rebuild literal quote taxonomy around the chosen cooked/raw model.
10. [ ] Implement cooked escape handling and cooked multiline continuation if that model is kept.
11. [ ] Add imaginary literal support or explicitly remove it from the front-end scope docs.
12. [ ] Rename typoed literal enum variants and update all call sites.
13. [ ] Remove `Illegal` from `is_void()` and add nested malformed-token regressions.
14. [ ] Replace synthetic in-band file-boundary newlines with an explicit boundary model.
15. [ ] Remove default `Cargo.toml` package detection and replace it with the chosen FOL-native package contract.
16. [ ] Replace string-joined qualified paths with structured path representation.
17. [ ] Re-run the parser diagnostic matrix after all AST and lexer changes.
18. [ ] Refresh `FRONTEND_CONTRACT.md`, `PROGRESS.md`, and any touched book pages.
19. [ ] Run final `make build`.
20. [ ] Run final `make test`.

## 13. Final Readiness Gates

Do not move to the next compiler stage until every gate below is true.

- [ ] Package and namespace behavior are FOL-defined instead of host-tool-defined.
- [ ] Cross-file boundaries are explicit and do not depend on fabricated source characters.
- [ ] Identifier rules are explicit, tested, and used consistently across stream/lexer/parser comparisons.
- [ ] Comment syntax matches the chosen authority and no longer conflicts with backtick tokenization.
- [ ] Literal quote behavior matches the chosen authority.
- [ ] Escape handling matches the chosen authority.
- [ ] Imaginary literal status is resolved instead of sitting in a silent limbo.
- [ ] `Illegal` tokens are never skipped as whitespace.
- [x] Method receiver types survive AST lowering.
- [x] Logical routine kind survives AST lowering.
- [ ] Qualified path structure survives AST lowering where the next stage needs it.
- [ ] Root program shape is explicit.
- [ ] No front-end doc claims are known to contradict the code.
- [ ] `make build` passes.
- [ ] `make test` passes.

## 14. Stop Condition

This plan is complete only when the next stage can consume the front-end without needing to:

- reverse-engineer method declarations from lost syntax
- infer logical routine kind from compromised AST shape
- split qualified names back out of flat strings
- guess whether comments, literals, and identifiers follow the book or the historical implementation
- guess whether file scope is declaration-only or script-like
- guess whether package identity comes from FOL rules or Rust project files
