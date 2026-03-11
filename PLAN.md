# FOL Front-End Implementation Plan

Last rebuilt: 2026-03-11
Scope: harden `fol-stream`, `fol-lexer`, and `fol-parser` before any next-stage compiler work

## 0. Goal

Make the current front-end robust enough that the next stage can trust:

- source order
- source identity
- token contracts
- literal fidelity
- AST shape
- parser invariants
- front-end diagnostics

## 1. Out Of Scope

Do not use this plan for:

- semantic analysis
- symbol resolution
- type checking
- ownership enforcement
- runtime
- interpreter
- backend

## 2. Working Rules

- [ ] No new major syntax during this plan unless required to fix a current mismatch.
- [ ] Every fix gets a regression test.
- [ ] Fix stream first, then lexer, then parser.
- [ ] Keep changes surgical unless a contract is fundamentally wrong.
- [ ] If behavior stays weird but intentional, document it.
- [ ] If behavior is accidental, remove it.

## 3. Current Blocking Problems

### Stream

- [ ] Folder traversal order is not deterministic.
- [ ] Source identity is not defined tightly enough.
- [ ] Package detection behavior is too ad hoc.
- [ ] Namespace rules are not written down as a contract.
- [ ] Location guarantees across file boundaries are implicit.

### Lexer

- [ ] Token payload rules are not explicit.
- [ ] String, character, raw-string, and raw-character handling is not clean.
- [ ] Comment treatment is too informal.
- [ ] Malformed lexical input behavior is not defined tightly enough.
- [ ] Numeric token fidelity is not strong enough for later phases.
- [ ] Stage responsibilities are too implicit.

### Parser

- [ ] Top-level `Program.declarations` is structurally contaminated by routine body leakage.
- [ ] Literal lowering is narrower than lexer support.
- [ ] AST invariants are not documented or enforced enough.
- [ ] Parser still mixes syntax work with some semantic-ish checks.
- [ ] Unsupported combinations are not always rejected explicitly.

## 4. Phase Order

- [ ] Phase 1: stream hardening
- [ ] Phase 2: lexer contract hardening
- [ ] Phase 3: parser structural hardening
- [ ] Phase 4: front-end contract freeze

Do not start Phase 2 before Phase 1 is green.
Do not start Phase 3 before Phase 2 is green.
Do not declare the front-end ready until Phase 4 is complete.

## 5. Phase 1: Stream Hardening
Target area:

- `fol-stream/src/lib.rs`
- `test/stream/test_stream.rs`
- `test/stream/test_namespace.rs`
- `test/stream/test_mod_handling.rs`

### 5.1 Deterministic traversal

- [x] Sort directory entries before recursive traversal.
- [x] Pick one ordering rule and keep it everywhere.
- [x] Use the same rule for root files and nested files.
- [x] Make sure recursion preserves that rule.
- [ ] Add tests that assert exact source order, not just source presence.

Done when:
- [ ] Repeated runs over the same folder produce the same source order.
- [ ] Tests fail if traversal order regresses.

### 5.2 Source identity

- [ ] Define what uniquely identifies a source at the stream boundary.
- [ ] Decide whether canonical absolute path is the primary identity.
- [ ] Decide how `package` and `namespace` participate in identity.
- [ ] Write down whether the same file can appear twice under different logical identities.
- [ ] Make tests assert the chosen behavior.

Done when:
- [ ] A future resolver could use stream output without guessing what a source "is".

### 5.3 Package detection

- [ ] Decide whether current `Cargo.toml` lookup stays as-is for now.
- [ ] Define fallback behavior when no manifest exists.
- [ ] Define behavior for single-file input outside a package root.
- [ ] Define behavior for nested manifests or workspace-like layouts.
- [ ] Add tests for the supported cases.

Done when:

- [ ] Package naming behavior is deliberate and tested.

### 5.4 Namespace contract

- [ ] Define root namespace behavior.
- [ ] Define nested namespace behavior.
- [ ] Define valid namespace component rules.
- [ ] Define interaction with `.mod` skipping.
- [ ] Decide whether invalid directory names are ignored or rejected.
- [ ] Add tests for edge cases.

Done when:

- [ ] Namespace derivation is fully specified by tests.

### 5.5 Location guarantees

- [ ] Document row and column origin.
- [ ] Confirm newline handling rules.
- [ ] Confirm row/column reset behavior when switching files.
- [ ] Define location meaning for synthetic EOF handling if downstream uses it.
- [ ] Add explicit tests for file boundary transitions.

Done when:

- [ ] Later diagnostics can trust stream locations without special-casing.

### 5.6 Eager loading

- [ ] Decide whether eager source loading is accepted for this cycle.
- [ ] If accepted, mark it as intentional.
- [ ] If not accepted, schedule a contained follow-up after parser hardening.

Done when:

- [ ] Eager loading is no longer accidental design.

### 5.7 Stream acceptance gate

- [ ] Source order is deterministic.
- [ ] Source identity rules are explicit.
- [ ] Package rules are explicit.
- [ ] Namespace rules are explicit.
- [ ] File-boundary location behavior is tested.

## 6. Phase 2: Lexer Contract Hardening
Target area:

- `fol-lexer/src/lexer/stage0/*`
- `fol-lexer/src/lexer/stage1/*`
- `fol-lexer/src/lexer/stage2/*`
- `fol-lexer/src/lexer/stage3/*`
- `fol-lexer/src/token/*`
- `test/lexer/test_lexer.rs`
- add or expand fixtures under `test/lexer/`

### 6.1 Token payload policy

- [ ] Decide what `con()` means for every token family.
- [ ] Decide whether delimiters stay in string-like payloads.
- [ ] Decide whether number payloads preserve original spelling.
- [ ] Decide whether whitespace and comments normalize to placeholder content or preserve source content.
- [ ] Write tests that assert payload shape directly.

Done when:

- [ ] Parser code no longer depends on undocumented payload quirks.

### 6.2 Literal taxonomy

- [ ] Decide the intended meaning of double quotes.
- [ ] Decide the intended meaning of single quotes.
- [ ] Decide the intended meaning of backticks, if kept.
- [ ] Separate character-like and string-like forms if the language requires it.
- [ ] Decide whether raw-vs-cooked form needs separate token kinds.

Done when:

- [ ] The lexer no longer conflates unrelated literal families.

### 6.3 Escape handling

- [ ] Decide whether escapes are validated in the lexer.
- [ ] Define accepted escape sequences for the current front-end scope.
- [ ] Define behavior for invalid escapes.
- [ ] Define behavior for unterminated quoted content.
- [ ] Define multiline continuation behavior if supported.
- [ ] Add positive and negative tests.

Done when:

- [ ] Escape behavior is explicit and test-backed.

### 6.4 Comment policy

- [ ] Decide whether normal comments remain fully ignorable.
- [ ] Decide whether doc comments are represented separately or explicitly deferred.
- [ ] If deferred, make that explicit in the front-end contract.
- [ ] Add tests for line comments, block comments, and comment-boundary spacing behavior.

Done when:

- [ ] Comment treatment is policy, not parser convenience.

### 6.5 Numeric fidelity

- [ ] Audit decimal, float, hex, octal, and binary tokenization.
- [ ] Define leading-dot float behavior.
- [ ] Decide whether negative numbers stay parser-level unary operations.
- [ ] Decide whether imaginary numbers are out of scope for this cycle.
- [ ] Add tests for every supported numeric family.

Done when:

- [ ] Lexer numeric output is precise enough for parser literal lowering.

### 6.6 Stage responsibilities

- [ ] Write down what each stage owns.
- [ ] Keep stage0 about raw character windowing only.
- [ ] Keep stage1 about first-pass token classification only.
- [ ] Keep stage2 about token folding and normalization only.
- [ ] Keep stage3 about final disambiguation only.
- [ ] Remove stage overlap if it causes hidden coupling.

Done when:

- [ ] A maintainer can explain each stage without hand-waving.

### 6.7 Illegal token strategy

- [ ] Define when the lexer emits `Illegal`.
- [ ] Define when the lexer returns an error instead.
- [ ] Make malformed-input handling consistent across literal families.
- [ ] Add negative tests for malformed lexical input.

Done when:

- [ ] Parser-facing error behavior is predictable.

### 6.8 Bootstrap and EOF cleanup

- [ ] Review the synthetic bootstrap behavior used by current tests.
- [ ] Reduce or isolate parser-visible startup artifacts if possible.
- [ ] Keep EOF behavior explicit and stable.

Done when:

- [ ] Tests no longer need unexplained lexer workarounds.

### 6.9 Lexer acceptance gate

- [ ] Token payload policy is fixed.
- [ ] Literal families are explicit.
- [ ] Comment policy is explicit.
- [ ] Malformed lexical input behavior is explicit.
- [ ] Stage responsibilities are explicit.

## 7. Phase 3: Parser Structural Hardening
Target area:

- `fol-parser/src/ast/mod.rs`
- `fol-parser/src/ast/parser.rs`
- `fol-parser/src/ast/parser_parts/*`
- `test/parser/test_parser.rs`
- `test/parser/test_parser_parts/*`
- add focused fixtures under `test/parser/` where needed

### 7.1 Fix `Program` contamination

- [ ] Remove top-level leakage of `fun` body statements into `Program.declarations`.
- [ ] Remove top-level leakage of `pro` body statements into `Program.declarations`.
- [ ] Audit whether any other declaration family leaks child nodes upward.
- [ ] Add explicit AST-shape regression tests for top-level declarations.

Done when:

- [ ] Top-level program shape contains only real top-level nodes.

### 7.2 Literal lowering

- [ ] Align parser literal lowering with lexer-supported literal families.
- [ ] Support correct AST lowering for currently supported strings.
- [ ] Support correct AST lowering for booleans.
- [ ] Support correct AST lowering for nil.
- [ ] Support correct AST lowering for decimal integers.
- [ ] Decide and implement lowering strategy for floats.
- [ ] Decide and implement lowering strategy for hex, octal, and binary values.
- [ ] Add tests that assert exact AST literal values.

Done when:

- [ ] Supported lexer literals become correct AST literals without ad hoc gaps.

### 7.3 AST invariants

- [ ] Define what may appear in `Program.declarations`.
- [ ] Define what may appear inside routine bodies.
- [ ] Define how quoted names are represented.
- [ ] Define how qualified paths are represented.
- [ ] Define invariants for grouped declarations.
- [ ] Add tests that lock those invariants down.

Done when:

- [ ] Later phases can consume AST shape without reverse-engineering parser behavior.

### 7.4 Name and path normalization

- [ ] Normalize named-label extraction.
- [ ] Normalize quoted-name extraction.
- [ ] Normalize qualified path extraction.
- [ ] Keep type-path handling and value-path handling deliberate.
- [ ] Add tests for quoted and qualified path forms.

Done when:

- [ ] Path and name AST encoding is predictable.

### 7.5 Declaration normalization

- [ ] Audit `fun`, `pro`, and `log` for shared structure.
- [ ] Audit alias, type, standard, implementation, and grouped declarations for structural consistency.
- [ ] Reject unsupported mixes explicitly instead of relying on incidental failure.
- [ ] Add targeted tests for current unsupported combinations.

Done when:

- [ ] Declaration families have stable AST shape and explicit failure modes.

### 7.6 Parser boundary cleanup

- [ ] Identify which current parser checks are purely structural.
- [ ] Keep structural checks in the parser.
- [ ] Stop adding semantic-like checks during this hardening cycle unless they are required to preserve AST correctness.
- [ ] Mark anything semantic-adjacent that stays temporarily.

Done when:

- [ ] Parser responsibility is narrower and clearer.

### 7.7 Statement vs expression boundary

- [ ] Audit top-level statement parsing.
- [ ] Audit block-body parsing.
- [ ] Audit call-vs-invoke-vs-assignment entry points.
- [ ] Audit control-flow surfaces that appear both as expressions and statements.
- [ ] Add shape tests where ambiguity currently exists.

Done when:

- [ ] Statement/expression boundaries are test-backed and intentional.

### 7.8 Parser diagnostics

- [ ] Normalize "expected X" wording.
- [ ] Normalize missing-close-token diagnostics.
- [ ] Normalize duplicate and unknown-option diagnostics where parser owns them.
- [ ] Add negative tests for important failure classes.

Done when:

- [ ] Parser diagnostics are consistent enough to freeze the front-end contract.

### 7.9 Parser module ownership

- [ ] Define which grammar area each `parser_parts` file owns.
- [ ] Identify overlap that causes maintenance risk.
- [ ] Refactor only where ownership is unclear enough to block maintenance.

Done when:

- [ ] Parser structure is easier to work in without broad churn.

### 7.10 Parser acceptance gate

- [ ] `Program` root shape is fixed.
- [ ] Literal lowering is complete for supported literal families.
- [ ] AST invariants are explicit and tested.
- [ ] Unsupported combinations fail intentionally.
- [ ] Parser diagnostic behavior is more consistent.

## 8. Phase 4: Front-End Contract Freeze
Target area:

- `PLAN.md`
- `PROGRESS.md`
- possibly `README.md` only if needed to stop lying about current front-end state

### 8.1 Stream contract summary

- [ ] Write a short stream contract summary:
- [ ] source ordering
- [ ] source identity
- [ ] package detection
- [ ] namespace derivation
- [ ] location guarantees

### 8.2 Lexer contract summary

- [ ] Write a short lexer contract summary:
- [ ] token payload meaning
- [ ] literal categories
- [ ] comment policy
- [ ] malformed-input policy

### 8.3 Parser contract summary

- [ ] Write a short parser contract summary:
- [ ] root AST invariants
- [ ] declaration invariants
- [ ] literal lowering guarantees
- [ ] parser-owned validations

### 8.4 Front-end readiness check

- [ ] Re-run the full front-end suite.
- [ ] Confirm no component still relies on undefined behavior from the previous phase.
- [ ] Record remaining front-end debt that is consciously deferred.

Done when:

- [ ] Stream, lexer, and parser each have a written contract.
- [ ] The current front-end can be handed to the next stage without guessing.

## 9. Test Matrix

### Stream tests

- [ ] exact source ordering
- [ ] namespace corner cases
- [ ] package fallback cases
- [ ] file boundary location tracking
- [ ] file/folder mismatch behavior

### Lexer tests

- [ ] exact token payload checks
- [ ] exact literal-family checks
- [ ] malformed literal checks
- [ ] comment and doc-comment checks
- [ ] EOF/bootstrap behavior checks

### Parser tests

- [ ] exact top-level AST shape
- [ ] exact literal AST lowering
- [ ] exact name/path AST shape
- [ ] unsupported combination failure tests
- [ ] parser diagnostic consistency checks where stable enough

### Cross-phase tests

- [ ] stream -> lexer order stability
- [ ] lexer -> parser literal continuity
- [ ] multi-file location continuity into parser diagnostics

## 10. Stop Conditions

Do not move to the next compiler step until all of these are true:

- [ ] stream order is deterministic
- [ ] source identity is explicit
- [ ] lexer token payload policy is fixed
- [ ] lexer literal taxonomy is fixed for supported forms
- [ ] parser root AST shape is fixed
- [ ] parser literal lowering is complete for supported forms
- [ ] stream, lexer, and parser contracts are written down
- [ ] regression tests cover the corrected behavior

## 11. Definition Of Done

This plan is done only when a maintainer can answer all of these without reading implementation internals:

- [ ] In what order are sources streamed from a folder?
- [ ] What exactly does a token payload contain?
- [ ] Which literal families exist today, and how are they tokenized?
- [ ] How are comments treated?
- [ ] What exactly belongs in `Program.declarations`?
- [ ] Which parser checks are structural and which are intentionally deferred?

If those answers still require code archaeology, the front-end is not hardened enough yet.
