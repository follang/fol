# FOL Editor Polish Plan

Last updated: 2026-03-20

## Goal

Take `fol-editor` from "usable and shipped" to "boring, trustworthy, and hard
to regress":

- formatting should feel intentional, not just minimally stable
- editor responses should stay predictable under ugly real editing flows
- the public `fol tool` surface should have stronger behavior guarantees
- more of the semantic contract should be locked with focused tests
- performance regressions should become easier to detect before they land

This is a follow-up plan after the first editor delivery plan reached 100%.

It is based on:

- the now-shipped `fol-editor` feature set
- remaining future-work in `plan/future-work/fol-editor.md`
- the current editor/LSP/frontend tests in-tree
- the need for much heavier regression coverage around editing behavior

## Current State

Already shipped:

- diagnostics, hover, definition, completion, references, rename
- semantic tokens, signature help, code actions
- whole-document formatting
- public `fol tool` exposure for the shipped editor features
- compiler-backed LSP behavior with per-version semantic caching

Remaining quality gaps:

- formatting is real, but still shallow and line-oriented
- range formatting is still unsupported
- rename safety is intentionally narrow
- code actions are intentionally narrow
- there is still not enough test density around malformed edits, cache
  invalidation, cross-feature interactions, and CLI/LSP parity

## Product Direction

The next editor phase should move in this order:

1. Lock the shipped behavior much harder.
2. Improve formatting quality and correctness.
3. Expand safe semantic features only where the safety boundary is explicit.
4. Add more performance and regression probes before adding broad new surface.

Do not add speculative features just because the LSP supports a method name.

## Workstreams

### Slice 1: Formatter Quality Pass

The current formatter is a valid first subsystem, but it still needs to become
less fragile and more language-aware.

Work:

- [x] decide and document the intended formatting style per major surface:
  declarations, records, calls, `when`, imports, and build files
- [x] tighten indentation and brace-depth handling around nested literals,
  `when` bodies, and mixed inline/block constructs
- [x] normalize trailing whitespace, blank-line behavior, and final newline rules
- [x] ensure formatting does not drift across repeated runs on already-formatted
  files
- [x] add formatter fixtures for:
  records, entries, `when`, nested literals, imports, aliases, build files,
  and representative error-tolerant input

Exit condition:

- formatting output is stable across a representative fixture corpus
- formatter fixtures cover both source files and `build.fol`
- formatter idempotence is locked explicitly

### Slice 2: Range Formatting Decision

Range formatting should only exist if it can be correct enough to trust.

Work:

- [x] evaluate whether range formatting can be implemented without corrupting
  surrounding structure
- [x] if yes, ship a safe first range-formatting boundary and lock it with tests
- [x] if no, keep it explicitly unsupported and document the reason
- [x] add tests proving the chosen behavior through LSP request handling

Exit condition:

- there is one explicit range-formatting policy:
  supported with constraints, or rejected intentionally

### Slice 3: Formatting Surface Parity

Formatting now exists in LSP and CLI, but parity needs stronger guarantees.

Work:

- [x] add tests proving `fol tool format` and `textDocument/formatting` produce the
  same output for the same file
- [x] lock CLI summary/detail shapes for formatted vs already-formatted files
- [x] add tests for formatting on files outside a discovered workspace
- [x] verify formatting behavior against `build.fol`
- [x] add integration tests proving formatting does not mutate unrelated files

Exit condition:

- public CLI formatting and LSP formatting stay behaviorally aligned

### Slice 4: Rename Safety Expansion

Rename should grow only where correctness is explicit.

Work:

- [x] define the next safe rename boundary beyond same-file locals, if one exists
- [x] evaluate same-package rename safety for unambiguous top-level items
- [x] reject cross-package and ambiguous cases explicitly
- [x] add regression tests for:
  locals, parameters, same-file top-levels, imported names, same-package
  namespaces, and refusal paths
- [x] lock exact failure messages/notes where the refusal contract matters

Exit condition:

- rename support is broader only where full edit sets are proven correct
- unsafe cases still fail before partial edits are produced

### Slice 5: Code Action Depth Without Guessing

Code actions should grow from compiler truth, not editor-side heuristics.

Work:

- [x] inventory compiler diagnostics that already carry structurally obvious fixes
- [x] add one narrow code-action slice at a time from real diagnostic suggestions
- [x] avoid speculative actions that require semantic guessing
- [x] add tests that prove code actions only appear for matching diagnostics and
  produce deterministic edits
- [x] add refusal/empty-result tests for nearby but unsupported diagnostic shapes

Exit condition:

- every shipped code action is compiler-backed, exact, and test-locked

### Slice 6: Workspace Symbols

Workspace symbols are a natural next semantic query if they can reuse existing
compiler facts safely.

Work:

- [x] define the first supported workspace-symbol scope:
  current package only, or current workspace members only
- [x] implement `workspace/symbol` without introducing a parallel semantic engine
- [x] sort and rank results deterministically
- [x] add tests for symbol visibility, namespace qualification, and ordering
- [x] document the scope boundary clearly

Exit condition:

- `workspace/symbol` is real, scoped, and deterministic

### Slice 7: Cache And Invalidation Hardening

The current split between diagnostics and semantic snapshots is better, but
there are still many invalidation edges to lock down.

Work:

- [x] add tests for repeated mixed request sequences:
  diagnose -> hover -> complete -> format -> rename -> close -> reopen
- [x] add tests for stale snapshot invalidation after ranged edits near symbol
  boundaries
- [x] add tests for multi-file sessions where only one file changes
- [x] add tests that differentiate diagnostic cache reuse from semantic cache reuse
- [x] add more stage-counter assertions around unchanged formatting and navigation
  requests

Exit condition:

- cache invalidation behavior is locked across mixed real-editor flows

### Slice 8: Error-Tolerant Editing Matrix

The editor needs stronger behavior guarantees while users are in broken states.

Work:

- [x] add completion, hover, signature-help, formatting, and code-action tests on
  parse-broken input
- [x] add tests for partially typed declarations, broken `when` blocks, and
  incomplete calls
- [x] add tests proving the server returns safe empty/null responses where needed
  instead of crashing or leaking stale data
- [x] add tests for recovery after broken text becomes valid again

Exit condition:

- broken intermediate text is a locked, ordinary case for the server

### Slice 9: Build File And Non-Standard File Coverage

`build.fol` already works in important paths. The test matrix should treat that
as a first-class contract.

Work:

- [x] add formatting fixtures and LSP formatting tests for `build.fol`
- [x] add signature-help, code-action, and rename refusal tests where relevant on
  `build.fol`
- [x] add CLI integration coverage for editor commands against build-entry files
- [x] ensure docs stay aligned if behavior differs intentionally from ordinary
  source files

Exit condition:

- build-file behavior is covered across more than diagnostics/navigation only

### Slice 10: Public Surface Contract Tests

The frontend/editor boundary now deserves stronger contract locking.

Work:

- [x] add CLI parsing tests for every shipped editor subcommand variant and edge
  flag combination
- [x] add summary-shape tests for every public editor command
- [x] add tests proving unsupported future commands stay rejected
- [x] add JSON/plain output snapshot-style assertions for representative editor
  commands and failures
- [x] lock `initialize` capability exposure against the shipped public docs

Exit condition:

- the public `fol tool` editor surface is hard to accidentally drift

### Slice 11: Test Structure Cleanup

The next wave of tests should make the suite easier to maintain, not noisier.

Work:

- [x] split giant mixed-purpose editor tests when they start covering unrelated
  behavior
- [x] introduce shared fixture/setup helpers only where they reduce repetition
  without hiding intent
- [x] keep one behavior per regression test where practical
- [x] add naming discipline so failures immediately describe the broken contract

Exit condition:

- editor tests stay denser without turning into unreadable mixed suites

## Testing Strategy

Add and keep these layers:

- formatter fixture snapshots
- LSP request/response tests per method
- mixed lifecycle tests covering cache invalidation and recovery
- frontend integration tests through `fol tool`
- parity tests between CLI and LSP for overlapping features
- performance smoke tests using the existing stage counters
- refusal-path tests for unsupported or intentionally unsafe behavior

Priority testing gaps to close first:

- formatting fixture depth
- malformed incremental edit sequences
- cache invalidation after mixed requests
- CLI/LSP formatting parity
- rename refusal coverage
- code-action diagnostic matching coverage

## Documentation Work

Update these docs as polish slices land:

- `book/src/050_tooling/200_tool_commands.md`
- `book/src/050_tooling/300_editor.md`
- `book/src/050_tooling/500_lsp.md`
- `plan/future-work/fol-editor.md`

Specifically:

- keep docs aligned with the real public surface
- move newly shipped items out of future-work immediately
- document explicit refusal boundaries, not just success cases
- document formatter scope and any intentional range-formatting limitation

## Immediate Next Slice

Start with Slice 1 and Slice 3 together:

- deepen formatter fixtures and idempotence coverage
- lock CLI/LSP formatting parity

That is the highest-signal polish work because formatting is newly shipped and
still the easiest place for visible regressions to slip in.
