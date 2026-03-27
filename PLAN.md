# PLAN: Quoted Import Targets Only

This plan replaces the current mixed import-target surface with one strict rule:

- the thing inside `use ... = { ... }` is always a string literal
- unquoted import targets are deleted
- there is no compatibility path
- there is no dual parser mode
- there is no migration warning period

Examples of the target syntax:

```fol
use std: pkg = {"std"};
use io: pkg = {"std/io"};
use logtiny: pkg = {"logtiny"};
use shared: loc = {"../shared"};
use nested: pkg = {"other/package/nested"};
```

Examples that must become invalid:

```fol
use std: pkg = {std};
use io: pkg = {std/io};
use logtiny: pkg = {logtiny};
use shared: loc = {../shared};
use math: loc = {core::math};
```

## Why

The current surface is inconsistent:

- parser fixtures already accept quoted targets
- some docs already teach quoted targets
- many tests/examples still use unquoted targets
- the parser still contains fallback logic for direct unquoted paths
- import targets are conceptually data, not expression syntax

The language should settle on one representation:

- import target path = string literal payload

That makes the grammar simpler and more explicit:

- `pkg`, `loc`, and any future import providers all use the same target form
- nested package paths stop being a special mini-language in braces
- parser and editor highlighting become easier to reason about
- diagnostics can point at a string target instead of ambiguous segment syntax

## Deep impact

This is not a docs-only cleanup. It goes through:

- lexer token expectations around textual literals in `use`
- parser `use` declaration path handling
- AST/storage of parsed import targets
- parser tests and fixtures
- resolver import target normalization and diagnostics
- frontend and integration fixtures
- lowering and typecheck tests that embed source snippets
- editor/LSP parse fixtures, semantic tests, completion examples
- tree-sitter examples/highlight coverage
- book/docs/examples

The old unquoted form must be deleted completely.

## Final contract

After this plan:

- `use ...: pkg = {"..."};` is required
- `use ...: loc = {"..."};` is required
- the string contents carry the import target
- braces remain part of the `use` surface
- quoted targets may contain `/`
- quoted targets may contain relative loc paths like `../shared`
- quoted targets may contain plain package names like `std` or `json`
- unquoted identifiers, unquoted path segments, and `::`-assembled targets inside
  the braces are rejected

## AST direction

Current parser state already preserves `path_segments` on `AstNode::UseDecl`.
That reflects the old structured path syntax.

Target direction:

- `AstNode::UseDecl` should store a canonical import target string
- if structured segments are still needed internally for a short refactor stage,
  that is an implementation detail only
- the parser contract should become string-first, not segment-first

## Epoch 1: Freeze Contract

### Slice 1
Status: completed

Write the top-level contract into active docs:

- import target in `use` braces is string-only
- unquoted targets are removed
- applies equally to `pkg` and `loc`

### Slice 2
Status: completed

Add/adjust parser-level contract tests that describe the target surface before
implementation changes:

- quoted `pkg`
- quoted `loc`
- quoted nested package path
- quoted relative loc path

### Slice 3
Status: completed

Add explicit negative parser tests for:

- `{std}`
- `{std/io}`
- `{json}`
- `{../shared}`
- `{core::math}`

### Slice 4
Status: completed

Audit and document all known import providers currently in use:

- `pkg`
- `loc`

and pin that they all share the same quoted-target rule.

## Epoch 2: Parser Cutover

### Slice 5
Status: completed

Remove the parser branch that accepts direct unquoted `use` paths in
`parse_use_paths`.

### Slice 6
Status: completed

Tighten `parse_use_paths` so valid targets come only from:

- `{ "..." }`
- `{ '...' }` if single-quoted text literals are still accepted as textual literals

or reject single-quoted string targets too, if the language wants one canonical
string literal style. Choose one and apply it consistently.

### Slice 7
Status: completed

Delete `parse_direct_use_path` if it becomes dead after the cutover.

### Slice 8
Status: completed

Delete or radically simplify `ensure_complete_use_path` if it only existed for
the old segment-assembled form.

### Slice 9
Status: completed

Change parser diagnostics to say:

- expected string literal import target
- import targets must be quoted

instead of segment-oriented messages like “Expected name after `::` in use path”.

### Slice 10
Status: completed

Update parser tests around colonless `use` declarations so they still parse the
source-kind side correctly while enforcing quoted targets.

## Epoch 3: AST And Parser Model Cleanup

### Slice 11
Status: completed

Replace `UsePathSegment`-first parser output with a canonical stored import
target string on `AstNode::UseDecl`.

### Slice 12
Status: completed

If full immediate replacement is too invasive, add the canonical string field
first and migrate all downstream consumers to it before deleting the old segment
field.

### Slice 13
Status: completed

Update parser test helpers that currently reconstruct text from
`path_segments`.

### Slice 14
Status: completed

Remove parser utilities/tests whose only purpose was preserving old structured
path segment separators such as `Slash` vs `DoubleColon`, unless that structure
is still needed elsewhere for unrelated syntax.

### Slice 15
Status: completed

Delete stale AST comments that describe `use` paths as structured segment trees
if the public parser contract is now string-based.

## Epoch 4: Resolver And Package Resolution

### Slice 16
Status: completed

Move resolver import loading to consume the canonical string import target
instead of reconstructed segment text.

### Slice 17
Status: completed

Update `pkg` import normalization to interpret the quoted string as the package
target exactly.

### Slice 18
Status: completed

Update `loc` import normalization to interpret the quoted string as the relative
or direct local path exactly.

### Slice 19
Status: completed

Add resolver errors for malformed or unsupported quoted targets only if they are
still semantically invalid after parsing.

### Slice 20
Status: completed

Delete resolver tests that still embed old unquoted `pkg` targets.

### Slice 21
Status: completed

Delete resolver tests that still embed old unquoted `loc` targets.

### Slice 22
Status: completed

Harden bundled `standard` / alias-backed `pkg` resolution tests so they only use
quoted targets.

## Epoch 5: Lowering, Typecheck, And Backend Fixtures

### Slice 23
Status: completed

Update lowerer tests that embed source strings with old unquoted import targets.

### Slice 24
Status: completed

Update typecheck tests that embed source strings with old unquoted import
targets.

### Slice 25
Status: completed

Update backend emit/layout tests that embed source strings with old unquoted
import targets.

### Slice 26
Status: completed

Add one cross-layer regression proving a quoted `pkg` import survives:

- parse
- resolve
- lower
- typecheck
- backend emit

### Slice 27
Status: completed

Add one cross-layer regression proving a quoted `loc` import survives:

- parse
- resolve
- lower
- typecheck
- backend emit

## Epoch 6: Frontend, CLI, And Workspace Fixtures

### Slice 28
Status: completed

Update CLI compile fixtures using old unquoted `std` imports.

### Slice 29
Status: completed

Update CLI compile fixtures using old unquoted `pkg` imports.

### Slice 30
Status: completed

Update CLI/typecheck fixtures using old unquoted `loc` imports.

### Slice 31
Status: completed

Update routed/integration workspace fixtures to the quoted target form only.

### Slice 32
Status: completed

Add a CLI negative regression that explicitly proves:

- `use std: pkg = {std};`

fails with a parser error that points at quoted import targets.

## Epoch 7: Editor And Tree-sitter

### Slice 33
Status: completed

Audit editor parse/semantic fixtures that still use old unquoted targets and
convert them.

### Slice 34
Status: completed

Update LSP completion tests that include `use` declarations so they only use
quoted targets.

### Slice 35
Status: completed

Update LSP navigation/hover tests that include `use` declarations so they only
use quoted targets.

### Slice 36
Status: completed

Update tree-sitter example coverage and syntax snapshots for quoted import
targets only.

### Slice 37
Status: completed

Add one editor regression proving a missing quote in a `use` target surfaces as
syntax/parser failure in the editor path, not a later resolver failure.

## Epoch 8: Examples And App Fixtures

### Slice 38
Status: completed

Convert all checked-in `examples/` package imports to quoted targets.

### Slice 39
Status: completed

Convert all `test/apps/fixtures` imports to quoted targets.

### Slice 40
Status: completed

Convert formal package fixtures under `test/app/formal` to quoted targets where
they declare source imports.

### Slice 41
Status: completed

Add one positive example showing bundled std import with the final form:

```fol
use std: pkg = {"std"};
```

### Slice 42
Status: completed

Add one positive example showing nested external package import with the final
form:

```fol
use nested: pkg = {"other/package/nested"};
```

### Slice 43
Status: completed

Add one negative example showing old unquoted import target rejection.

## Epoch 9: Docs And Book

### Slice 44
Status: completed

Update the build/import docs to teach quoted import targets only.

### Slice 45
Status: completed

Update runtime/bundled-std docs to teach:

- `use std: pkg = {"std"};`

and remove old brace-with-bare-name examples.

### Slice 46
Status: completed

Update book chapters covering modules/imports/source origins so every shown
`use` target is quoted.

### Slice 47
Status: completed

Update contributor docs and `AGENTS.md` to pin the new import contract and say
explicitly that old unquoted forms must not be reintroduced.

## Epoch 10: Final Cleanup

### Slice 48
Status: completed

Repo-wide stale sweep for:

- `pkg = {std}`
- `pkg = {json}`
- `pkg = {logtiny}`
- `loc = {math}`
- `loc = {../shared}`

and all similar unquoted forms.

### Slice 49
Status: completed

Add one top-level sync/integration test that scans canonical examples/docs for
stale unquoted import targets.

### Slice 50
Status: completed

Run the full repo gate after the cutover:

- `make build`
- `make test`

and only then mark the plan complete.
