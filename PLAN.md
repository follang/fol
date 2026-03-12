# FOL Front-End Follow-Up Plan

Last rebuilt: 2026-03-12
Derived from: `PROGRESS.md` at the current workspace head
Scope: only remaining in-scope work for `fol-stream`, `fol-lexer`, `fol-parser`, and front-end contract docs

## 0. Current Position

- `make build` passes.
- `make test` passes.
- Current observed totals: `1` unit test and `1212` integration tests, all green.
- Stream, lexer, and parser no longer have known correctness blockers that justify another deep rescan before moving on.
- This plan therefore tracks only the remaining in-scope cleanup, contract-freeze, and shape-improvement work.

## 1. In Scope

- `fol-stream`
- `fol-lexer`
- `fol-parser`
- `FRONTEND_CONTRACT.md`
- front-end-facing `README.md` wording if needed
- relevant book pages only when they must move with a stream/lexer/parser contract change

## 2. Out Of Scope

Do not add work here for:

- whole-program resolution
- type checking
- ownership analysis
- standard or protocol conformance
- runtime behavior
- interpreter or backend work
- code generation
- optimization

## 3. Should Do

These are the remaining items worth closing if we want the front-end surface to be cleaner and more frozen before later phases build on it.

### 3.1 Stream: Replace Regex-Based `.mod` Detection

Current state:

- `.mod` directories are skipped correctly, but detection still uses a regex.

Required work:

- Replace regex-based `.mod` matching with a direct suffix or path-component check.
- Keep traversal order and `.mod` skipping behavior unchanged.
- Remove the regex dependency from this path if it becomes unused.

Acceptance:

- `.mod` directories are still skipped.
- Non-`.mod` directories are still traversed.
- Existing stream tests stay green.

### 3.2 Lexer: Freeze Slash-Comment Policy

Current state:

- Backticks are the primary comment form.
- Slash comments are still supported as explicit compatibility syntax.

Required work:

- Make a final front-end decision:
- keep slash comments as supported compatibility syntax and document that as frozen behavior
- or remove slash comments and update lexer, tests, and docs together
- Ensure the chosen policy is explicit in `FRONTEND_CONTRACT.md` and the scanned lexer docs.

Acceptance:

- There is one clear slash-comment policy in code, tests, and docs.
- No lexer ambiguity remains about whether slash comments are temporary or permanent.

### 3.3 Lexer + Parser: Preserve Comments And Doc Comments Past Lexing

Current state:

- Stage 1 classifies comments explicitly as backtick, doc, slash-line, and slash-block.
- Stage 2 currently normalizes all comment kinds back to `Void(Space)`.
- The parser does not retain comments or doc comments in the AST.

Why this matters now:

- Future doc-comment parsing should not require re-scanning raw source text outside the front-end pipeline.
- If comments are discarded before AST construction, later work has no structured place to attach docstrings or preserve comment trivia.

Required work:

- Decide the parser-facing representation for retained comments:
- preserve comment/doc-comment tokens through the lexer boundary and let the parser lower them
- or attach them as explicit trivia/comment nodes during parsing
- Keep ordinary comments and doc comments distinct.
- Preserve raw comment spelling so future doc-comment parsing can run on the retained content.
- Ensure retained comments do not break existing parsing of declarations, expressions, or body statements.
- Add tests covering:
- root-level retained comments
- body-level retained comments
- doc comments adjacent to declarations
- ordinary comments remaining non-semantic even when preserved

Acceptance:

- Comments and doc comments survive past the lexer into a parser-visible or AST-visible form.
- Doc comments remain distinguishable from ordinary comments.
- Future doc-comment parsing can build on retained front-end structures instead of re-lexing source files.

### 3.4 Parser: Clarify The Mixed-Root Program Carrier

Current state:

- `AstNode::Program { declarations }` is intentionally mixed and script-like, but the field name still suggests declaration-only contents.

Required work:

- Decide whether to rename this root field or explicitly freeze the existing name as intentional.
- If the name stays, document the mixed-root contract where later consumers will see it.
- If the name changes, update parser tests and any front-end contract docs in the same slice.

Acceptance:

- Later phases do not have to guess whether `Program.declarations` can contain non-declaration nodes.
- The AST root naming is either corrected or explicitly documented as stable.

### 3.5 Parser: Decide The Long-Term `use` Path Storage Shape

Current state:

- `UseDecl` now keeps structured path segments, but it also still carries raw path text.

Required work:

- Decide whether both representations are intentionally needed.
- If both stay, document why.
- If only structured segments are needed, remove the redundant raw storage and update tests/docs together.

Acceptance:

- The `use` path contract is explicit.
- Later import work does not have to guess which field is authoritative.

### 3.6 Parser: Quarantine Or Relocate `AstNode::get_type()`

Current state:

- `AstNode::get_type()` is still present even though whole-program semantic analysis does not exist yet.

Required work:

- Decide whether this helper belongs on the AST at all.
- Either narrow it, move it, or clearly mark it as a non-semantic convenience helper.
- Avoid letting later semantic work accidentally depend on heuristic parser-era typing behavior.

Acceptance:

- The ownership boundary between parsing and later typing work is clearer.
- `get_type()` is either quarantined, relocated, or intentionally documented.

## 4. Nice To Have

These items are in scope, but they are not strong enough to block moving on.

### 4.1 Stream: Remove Per-File `chars().collect()` Duplication

Current state:

- `FileStream::next_char` still rebuilds per-file `Vec<char>` buffers from already-loaded strings.

Nice-to-have work:

- Reuse already-prepared character storage or otherwise avoid repeated per-file re-collection.
- Keep behavior identical and avoid changing source identity or location semantics.

Acceptance:

- Stream behavior stays identical.
- The implementation no longer pays obvious per-file character-buffer duplication cost.

### 4.2 Docs: Tighten README Front-End Precision

Current state:

- `README.md` is directionally correct, but it is less precise than `FRONTEND_CONTRACT.md`.

Nice-to-have work:

- Make the README point more clearly at the frozen front-end contract.
- Avoid duplicating detailed rules in multiple places if `FRONTEND_CONTRACT.md` is the authoritative document.

Acceptance:

- README no longer risks sounding more precise or less precise than the actual front-end contract.
- The authoritative contract location is obvious.

## 5. Execution Order

Recommended order if we choose to close this plan before later phases:

1. `3.1` stream `.mod` detection cleanup
2. `3.2` lexer slash-comment policy freeze
3. `3.3` comment/doc-comment preservation
4. `3.4` parser mixed-root carrier decision
5. `3.5` parser `use` path storage decision
6. `3.6` parser `get_type()` boundary cleanup
7. `4.1` stream character-buffer cleanup
8. `4.2` README precision cleanup

## 6. Completion Rule

This plan is done when:

- every `Should Do` item is either implemented or intentionally frozen by explicit documentation
- any chosen behavior changes land with tests in the same slice
- no out-of-scope semantic work is mixed into the effort
- `make build` and `make test` remain green after each slice
