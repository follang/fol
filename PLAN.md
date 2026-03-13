# FOL Diagnostics And Errors Plan

Last rebuilt: 2026-03-13
Scope: `fol-diagnostics` and the error/diagnostic surfaces of the existing compiler crates, with the goal of making diagnostics strong enough before the typechecker phase starts producing much larger semantic error volume

## 0. Purpose

- The current compiler pipeline already produces exact parser, package, and resolver failure locations.
- That is good enough for infrastructure confidence, but not yet good enough for a mature compiler user experience.
- The next problem is not "can we report an error at all?"
- The next problem is "can we report errors in a structured, stable, actionable, multi-location, future-proof way?"
- This plan focuses on diagnostics and error representation only.
- It does not implement typechecking, lints, C ABI, or backend work.

## 1. What Was Scanned For This Plan

This plan is based on the current implementation, not wishful design.

### 1.1 Code Surfaces Checked

- `fol-diagnostics/src/lib.rs`
- CLI integration in `src/main.rs`
- parser errors in `fol-parser/src/ast/parser.rs`
- package errors in `fol-package/src/errors.rs`
- resolver errors in `fol-resolver/src/errors.rs`
- current diagnostics integration tests in `test/run_tests.rs`

### 1.2 Main Scan Outcome

Diagnostics are present, but still minimal:

- exact single locations exist
- JSON output exists
- human-readable output exists
- package and resolver errors carry structured error kinds and optional origins

But the system is still weak in the ways that matter for a full compiler:

- no structured multi-span diagnostics
- no stable modern error-code mapping
- no snippet renderer using source lines and spans
- no producer-side contract for notes, suggestions, or related labels
- parser/package/resolver location conversion is duplicated
- warning/info infrastructure exists but is barely real
- tests mostly prove existence, not quality

## 2. Current State Summary

### 2.1 What Already Works

- `DiagnosticReport`
- JSON serialization for diagnostics
- human-readable summary output
- primary file/line/column/length support
- package and resolver errors already expose diagnostic locations
- CLI already passes parser/package/resolver errors through one report

### 2.2 What Is Too Primitive

- only one location per diagnostic
- optional `help` string but no structured notes/labels
- human renderer prints no source snippets
- `length` is collected but largely unused in human rendering
- current error-code extraction is based on parsing message strings
- most modern errors collapse to `E0000`
- there is no stable compiler-wide error catalog
- there is no reusable diagnostic-builder surface

### 2.3 Why This Matters Before Typechecking

Typechecking will multiply error volume and complexity:

- incompatible assignment
- wrong call arity
- wrong argument type
- wrong return type
- wrong report type
- branch mismatch
- unsupported semantic surfaces
- related declaration/reference sites

If diagnostics stay primitive, typechecker errors will become noisy, inconsistent, and difficult to stabilize.

## 3. Main Decision

We keep `fol-diagnostics` as the central diagnostics crate, but we expand it from a thin formatter into a structured diagnostics model.

The crates keep owning their own semantic error kinds:

- parser owns parse errors
- package owns package errors
- resolver owns resolver errors
- future typechecker owns type errors

But all of them should be able to lower into one richer diagnostics contract.

## 4. Goals

The diagnostics layer must support all of the following:

- stable diagnostic codes
- exact primary spans
- related spans
- notes
- help text
- optional suggestions/fix hints
- human rendering with snippets
- machine-readable JSON with the same structured information
- low-friction producer-side APIs

## 5. Non-Goals

This plan does not:

- add lints as a full warning framework
- implement typechecker errors themselves
- implement IDE/LSP transport
- implement source caching for the whole compiler outside diagnostics needs
- redesign every existing error type beyond what is needed for diagnostics lowering

## 6. Problems To Fix

### 6.1 Error Code System Is Stale

Current problem:

- `extract_error_code(...)` still keys off old string fragments
- package and resolver errors mostly become `E0000`

Target:

- every modern compiler error family must map to a stable explicit code
- codes must not depend on message wording

### 6.2 Human Output Ignores Actual Span Data

Current problem:

- human output prints only message + file:line:column
- `length` is not used to underline anything

Target:

- human output should render:
  - source line
  - caret/underline span
  - multi-line label support where feasible
  - related labels/notes

### 6.3 Diagnostic Model Is Too Flat

Current problem:

- one optional location
- one optional help
- no related sites

Target:

- structured primary label
- zero or more secondary labels
- zero or more notes
- zero or more helps
- optional suggestion records

### 6.4 Producer Integration Is Ad Hoc

Current problem:

- parser locations are manually downcast in the CLI
- package/resolver duplicate location access patterns

Target:

- one producer-side lowering trait or builder path
- parser/package/resolver/typechecker should all lower through the same structured contract

### 6.5 Tests Are Too Shallow

Current problem:

- current tests prove that JSON/human exist
- they do not freeze quality or structure deeply

Target:

- snapshot-like human rendering tests
- structured JSON tests
- multi-location tests
- stable error-code tests
- CLI integration tests for real end-to-end formatting

## 7. Proposed Diagnostics Model

### 7.1 DiagnosticCode

Add a stable code type, likely:

- `DiagnosticCode(&'static str)` or `String`

The important rule:

- code comes from error kind mapping, not string parsing

Examples:

- `P0001` parse family
- `K0001` package family
- `R0001` resolver family
- later `T0001` typechecker family

The exact prefix scheme can vary, but it must be systematic.

### 7.2 DiagnosticSpan

Current `DiagnosticLocation` is close, but insufficient as the only label object.

Add something like:

- `DiagnosticSpan`
  - `file`
  - `line`
  - `column`
  - `length`

This remains the transport shape for precise locations.

### 7.3 DiagnosticLabel

Add something like:

- `DiagnosticLabel`
  - `span`
  - `message`
  - `is_primary`

This is what allows:

- "duplicate declared here"
- "first declaration here"
- "candidate also found here"
- "call target declared here"

### 7.4 DiagnosticNote

Add structured text buckets rather than overloading one `help` field:

- `notes: Vec<String>`
- `helps: Vec<String>`

This allows:

- concise technical note
- actionable help
- migration guidance

### 7.5 DiagnosticSuggestion

Not every phase will use this immediately, but the model should allow it:

- `span`
- `replacement`
- `message`
- `applicability` optional later

V1 does not need auto-fix application, only representation.

### 7.6 Diagnostic

Likely final shape:

- `severity`
- `code`
- `message`
- `primary_label`
- `labels`
- `notes`
- `helps`
- `suggestions`

JSON should serialize all of this directly.

## 8. Producer-Side Contract

### 8.1 Keep Error Types Local

Each compiler phase still owns its native error kind enum:

- `ParseError`
- `PackageError`
- `ResolverError`
- future `TypecheckError`

### 8.2 Add A Lowering Trait

Add something like:

- `ToDiagnostic`

This should allow each error type to describe:

- severity
- stable code
- main message
- labels
- notes
- helps

### 8.3 Keep `Glitch` Compatibility For Now

The current compiler already passes `&dyn Glitch` around.

Do not break the whole compiler at once.

Instead:

- add richer lowering beside the old path
- migrate producers gradually
- retire message-based extraction after the major producers are migrated

## 9. Human Renderer Requirements

The human renderer should support:

- severity prefix
- diagnostic code display
- main message
- `--> file:line:column`
- source line rendering
- underline/caret rendering using `length`
- related labels after the primary label
- notes/help blocks with consistent prefixes
- summary footer

It should also degrade cleanly when:

- file cannot be read
- line is missing
- span length is absent
- location is missing entirely

## 10. JSON Renderer Requirements

JSON output should preserve the same structure as the rich in-memory diagnostic:

- code
- severity
- message
- labels
- notes
- helps
- suggestions

Important rule:

- JSON should not be a lossy "machine summary" of the human renderer
- human and JSON should both come from the same structured diagnostic model

## 11. Source Snippet Strategy

Human rendering with source snippets requires source retrieval.

We should not overcomplicate this initially.

V1 approach:

- if `file` exists and is readable, read it on demand
- render the target line
- underline the span using `length`
- if unavailable, fall back to location-only output

Later optimization:

- line cache if performance matters

## 12. Severity Strategy

Current severities already exist:

- `Error`
- `Warning`
- `Info`

Immediate plan:

- fully support them in the data model and renderers
- keep most compiler producers on `Error` for now
- add report APIs for warnings/info even if only a few tests use them initially

## 13. Error Family Mapping

### 13.1 Parser

Need stable mapping from parse failure categories to parser diagnostic codes.

Current parser is weaker here because `ParseError` is mostly message-only plus location.

That means one of two approaches is needed:

- enrich `ParseError` with a parse error kind
- or add a structured parse-to-diagnostic categorizer before typechecker arrives

Preferred:

- add a parse error kind enum

### 13.2 Package

Package already has:

- `InvalidInput`
- `Unsupported`
- `ImportCycle`
- `Internal`

This is a strong base for stable code mapping.

### 13.3 Resolver

Resolver already has:

- `InvalidInput`
- `Unsupported`
- `UnresolvedName`
- `DuplicateSymbol`
- `AmbiguousReference`
- `ImportCycle`
- `Internal`

This is also a strong base for stable code mapping.

### 13.4 Future Typechecker

The diagnostics system must be ready for:

- incompatible types
- arity mismatch
- bad return/report type
- unsupported semantic surface

## 14. Test Strategy

We need much stronger diagnostics-focused tests.

### 14.1 Unit Tests In `fol-diagnostics`

Must cover:

- structured JSON serialization
- human rendering with:
  - primary label
  - secondary labels
  - missing file fallback
  - missing location fallback
  - notes/help rendering
  - summary rendering
- stable code rendering

### 14.2 Producer Tests

For package/resolver/parser:

- error kind to diagnostic code mapping
- exact label placement
- help/note text where applicable

### 14.3 Integration Tests

End-to-end CLI tests should cover:

- parse errors with exact snippets
- package errors with exact locations
- resolver ambiguity/duplicate errors with related labels
- JSON shape stability

## 15. Proposed Crate And File Shape

No new crate is required.

Expected `fol-diagnostics` expansion:

- `fol-diagnostics/src/lib.rs`
- `fol-diagnostics/src/model.rs`
- `fol-diagnostics/src/render_human.rs`
- `fol-diagnostics/src/render_json.rs`
- `fol-diagnostics/src/source.rs`
- `fol-diagnostics/src/codes.rs`

This file split may vary, but these responsibilities must exist.

## 16. Implementation Phases

### Phase 0: Contract Reset

Status: pending

#### 0.1

Status: done

- Freeze the diagnostics scope: this phase is about structured reporting, not new semantic checks.

#### 0.2

Status: done

- Document current diagnostics shortcomings in `PROGRESS.md` and this plan so later work is measured against real gaps.

#### 0.3

Status: done

- Add baseline tests that lock today’s JSON/human output before refactoring the model.

### Phase 1: Rich Diagnostic Model

Status: pending

#### 1.1

Status: done

- Replace the flat `Diagnostic { location, help }` shape with structured labels, notes, helps, and suggestions.

#### 1.2

Status: done

- Introduce stable diagnostic-code representation separate from message parsing.

#### 1.3

Status: done

- Keep backward compatibility helpers long enough so current producers can migrate incrementally.

#### 1.4

Status: done

- Add rich-model serialization tests.

### Phase 2: Human Renderer

Status: pending

#### 2.1

Status: done

- Split human rendering into a dedicated renderer implementation.

#### 2.2

Status: done

- Render primary spans with source lines and underline markers.

#### 2.3

Status: done

- Render secondary labels, notes, and helps consistently.

#### 2.4

Status: done

- Add fallback behavior for unreadable/missing files and missing spans.

#### 2.5

Status: done

- Lock human-output snapshot tests for representative parser/package/resolver diagnostics.

### Phase 3: JSON Renderer

Status: pending

#### 3.1

Status: done

- Split JSON rendering into a dedicated renderer implementation over the rich model.

#### 3.2

Status: done

- Keep JSON shape stable and fully structured, including labels/notes/helps/suggestions.

#### 3.3

Status: done

- Add exact JSON fixture tests for real compiler errors.

### Phase 4: Stable Error Codes

Status: pending

#### 4.1

Status: done

- Add explicit code mapping for package error kinds.

#### 4.2

Status: done

- Add explicit code mapping for resolver error kinds.

#### 4.3

Status: pending

- Introduce a proper parse error kind or equivalent structured mapping for parser diagnostics.

#### 4.4

Status: pending

- Remove or quarantine `extract_error_code(...)` message parsing once modern mappings are in place.

#### 4.5

Status: pending

- Add tests proving no modern compiler errors fall back to unknown generic code by accident.

### Phase 5: Producer Lowering

Status: pending

#### 5.1

Status: pending

- Add a producer-side lowering trait such as `ToDiagnostic`.

#### 5.2

Status: pending

- Migrate package errors to rich diagnostics.

#### 5.3

Status: pending

- Migrate resolver errors to rich diagnostics.

#### 5.4

Status: pending

- Migrate parser errors to rich diagnostics and remove CLI downcast special-casing.

#### 5.5

Status: pending

- Add low-friction helper APIs so future typechecker errors can adopt the system without duplication.

### Phase 6: Multi-Location Reporting

Status: pending

#### 6.1

Status: pending

- Add related labels for duplicate symbol diagnostics.

#### 6.2

Status: pending

- Add related labels for ambiguity diagnostics.

#### 6.3

Status: pending

- Add related labels for package-control diagnostics where a second site matters.

#### 6.4

Status: pending

- Freeze the wording and label strategy for multi-location diagnostics with tests.

### Phase 7: Severity And Help Plumbing

Status: pending

#### 7.1

Status: pending

- Add explicit APIs for warnings and info diagnostics.

#### 7.2

Status: pending

- Start using `help` and `notes` for migration/unsupported guidance where current errors already imply it.

#### 7.3

Status: pending

- Add representative tests for warning/info rendering, even if compiler producers still mostly emit errors.

### Phase 8: CLI Integration

Status: pending

#### 8.1

Status: pending

- Update the CLI to consume rich diagnostics without custom per-error-type special cases.

#### 8.2

Status: pending

- Ensure JSON mode preserves the richer structure without breaking current CLI behavior expectations.

#### 8.3

Status: pending

- Add end-to-end integration tests for human and JSON diagnostics with rich labels.

### Phase 9: Docs And Closeout

Status: pending

#### 9.1

Status: pending

- Update `README.md`, `PROGRESS.md`, and `FRONTEND_CONTRACT.md` to describe the richer diagnostics contract.

#### 9.2

Status: pending

- Update the book only where it talks about compiler-facing error behavior or exact output guarantees.

#### 9.3

Status: pending

- Rewrite this file into a completion record once structured diagnostics are fully adopted across parser/package/resolver.

## 17. Definition Of Done

This plan is complete when all of the following are true:

- `fol-diagnostics` has a structured multi-label diagnostic model
- human output renders source snippets and underlines spans
- JSON output preserves the rich structure
- package and resolver errors map to stable codes directly
- parser diagnostics no longer rely on ad hoc CLI downcasts for location extraction
- modern compiler errors no longer fall back to generic unknown codes by default
- duplicate and ambiguity diagnostics can show related sites structurally
- warning/info paths exist and are tested
- integration tests lock both human and JSON behavior
- docs describe the richer diagnostics contract accurately

## 18. Next Boundary After This Plan

Once diagnostics are in good shape, the next major compiler phase should be `fol-typecheck`.

That is why this diagnostics pass matters now:

- typechecking will produce the first large wave of semantic diagnostics
- this diagnostics plan should finish first or at least reach a strong midpoint before typechecker work begins in earnest
