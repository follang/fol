# FOL Source Layout And Package Scope Alignment Completion Record

Completed: 2026-03-12
Status: complete
Active next-phase plan: `PLAN_NEXT.md`

## 1. Purpose

- This file is no longer an active work plan.
- It records what the finished source-layout and package-scope alignment phase delivered.
- The active implementation plan has moved to `PLAN_NEXT.md` for `fol-resolver`.

## 2. Completed Scope

This completed phase covered:

- `fol-stream`
- `fol-lexer`
- `fol-parser`
- root CLI integration
- front-end contract and status docs

This completed phase did not cover:

- whole-program name resolution
- type checking
- ownership or borrowing analysis
- runtime or backend work

## 3. Executed Slice Set

All planned slices for this phase are complete:

- Phase 0: `0.1`, `0.2`
- Phase 1: `1.1`, `1.2`, `1.3`
- Phase 2: `2.1`, `2.2`, `2.3`, `2.4`
- Phase 3: `3.1`, `3.2`, `3.3`, `3.4`
- Phase 4: `4.1`, `4.2`, `4.3`
- Phase 5: `5.1`, `5.2`
- Phase 6: `6.1`, `6.2`, `6.3`
- Phase 7: `7.1`, `7.2`

Completed slice count: `23 / 23`

## 4. Final Front-End Outcome

The finished front-end boundary now guarantees:

- `parse_package(...)` returns a structured `ParsedPackage` instead of only a flattened root.
- Parsed packages preserve physical source units with per-file path, package, namespace, and source order.
- Successful syntax keeps parser-owned origin data through `SyntaxNodeId` and `SyntaxIndex`.
- Declaration-oriented package parsing enforces book-aligned file roots.
- The legacy mixed-root `AstParser::parse()` path remains only as a compatibility shim.
- The CLI now compiles through the declaration-oriented parsed package shape.
- Top-level parsed items now carry explicit declaration visibility/scope metadata for later resolver work.
- Cross-file boundary failures are locked with exact file/line/column coverage, including synthetic boundary-token locations.
- Root comments and parser-visible inline comments survive in the AST under the current documented model.

## 5. Validation Baseline

The final validation run for this completed phase is:

- `make build`: passed
- `make test`: passed
- unit tests: `1` passed
- integration tests: `1250` passed

## 6. Remaining In-Scope Work

- none

This source-layout alignment phase is complete for the current stream + lexer + parser scope.

## 7. Next Phase

The active next-phase plan is now `PLAN_NEXT.md`.

That next phase should implement:

- `fol-resolver`
- package/shared-scope name resolution
- namespace resolution
- import target resolution
- `exp` / `hid` enforcement
- later semantic handoff toward type checking
