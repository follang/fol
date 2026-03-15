# Compiler Diagnostics

This chapter is about compiler reporting, not about language-level `panic` or
`report` semantics.

In other words:

- `panic` and `report` describe what *your program* does
- diagnostics describe what *the compiler* tells you when it cannot continue or
  when it wants to surface important information

## Why this chapter exists

FOL now has a real compiler pipeline:

- `fol-stream`
- `fol-lexer`
- `fol-parser`
- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-lower`
- `fol-diagnostics`

That means errors are no longer just loose strings printed from one place.
Compiler failures now move through a shared diagnostics layer with stable
structure.

This matters for three reasons:

- humans need readable compiler output
- tests need stable enough structure to assert against
- future tools need a machine-readable format that is not just a copy of the
  human renderer

## What a diagnostic contains

At the current compiler stage, a diagnostic can carry:

- severity
- main message
- one primary location
- zero or more related locations
- notes
- helps
- suggestions
- a stable producer-owned code in the structured model

The current compiler mostly emits `Error`, but the reporting layer also supports
`Warning` and `Info`.

## Primary location

The most important part of a diagnostic is its primary location.

That location is currently expressed as:

- file
- line
- column
- optional span length

This is what allows the compiler to point at the exact token or source span that
caused the failure.

Typical examples:

- a parser error at the token that made a declaration invalid
- a package-loading error at the control file or package root that failed
- a resolver error at the unresolved identifier or ambiguous reference
- a typecheck error at the expression or declaration whose types do not match
- a lowering error at the typed surface that has no current `V1` lowering rule

## Related locations

Some compiler failures are not well described by one location alone.

For example:

- duplicate declarations
- ambiguous references
- duplicate package metadata fields

In those cases the compiler keeps one primary site and can also attach related
sites as secondary labels.

That allows the compiler to say things like:

- this declaration conflicts with an earlier declaration
- this name could refer to either of these two candidates
- this metadata field was already defined elsewhere

## Notes, helps, and suggestions

FOL diagnostics separate extra guidance into different buckets instead of
forcing everything into one long message.

The current contract is:

- the main message says what went wrong
- notes add technical context
- helps add actionable guidance
- suggestions describe a possible replacement or next step when the producer can
  express one

This split matters because tooling and tests can preserve structure instead of
trying to parse intent back out of prose.

## Human-readable diagnostics

By default the CLI prints human-readable diagnostics.

The current renderer is designed around:

- a severity-prefixed message
- an arrow line with `file:line:column`
- a source snippet when the file and line can be loaded
- an underline for the primary span
- note-style summaries for related labels
- note/help lines after the main snippet

Illustrative shape:

```text
error: could not resolve name `answer`
  --> app/main.fol:3:12
    |
  3 |     return answer
    |            ^^^^^^ unresolved name
  note: no visible declaration with that name was found in the current scope chain
  help: check imports or declare the name before use
```

This example is intentionally illustrative.

The current contract is about the information that survives:

- exact primary location
- snippet-oriented rendering when source is available
- related labels, notes, and helps

The exact wording and final formatting details can still evolve.

## Source fallbacks

Sometimes the compiler knows the location but cannot render the source line
itself.

Examples:

- the file is no longer readable
- the file path is missing
- the requested line is outside the current file contents

In those cases the compiler still keeps the location and falls back cleanly
instead of crashing the renderer.

So the priority order is:

1. exact location
2. source snippet when available
3. explicit fallback note when the snippet cannot be shown

## JSON diagnostics

When the CLI is invoked with `--json`, diagnostics are emitted as structured
JSON instead of human-readable text.

This output is meant for scripts, tests, editor tooling, and future integration
layers.

Important rule:

- JSON is not a lossy summary of human output

Instead, both human and JSON outputs are generated from the same structured
diagnostic model.

That means JSON can preserve:

- severity
- code
- message
- primary location
- related labels
- notes
- helps
- suggestions

Illustrative shape:

```json
{
  "severity": "Error",
  "code": "R2001",
  "message": "could not resolve name `answer`",
  "location": {
    "file": "app/main.fol",
    "line": 3,
    "column": 12,
    "length": 6
  },
  "labels": [
    {
      "kind": "Primary",
      "message": "unresolved name",
      "location": {
        "file": "app/main.fol",
        "line": 3,
        "column": 12,
        "length": 6
      }
    }
  ],
  "notes": [
    "no visible declaration with that name was found in the current scope chain"
  ],
  "helps": [
    "check imports or declare the name before use"
  ],
  "suggestions": []
}
```

Again, the exact payload can evolve, but the important guarantee is that the
structured fields are first-class rather than reverse-engineered from text.

## Which compiler phases currently participate

At head, the main producers that lower into the shared diagnostics layer are:

- parser
- package loading
- resolver
- type checking
- lowering

That means diagnostics are already strong across:

- syntax errors
- package metadata and package-root errors
- import-loading failures
- unresolved names
- duplicate names
- ambiguous references
- type mismatches and unsupported semantic surfaces inside `V1`
- unsupported lowered `V1` surfaces that still stop before any backend exists

This is the important boundary for the current compiler stage:

- the compiler can now parse, resolve, type-check, and lower the supported `V1`
  subset
- diagnostics already cover failures from each of those stages
- the project still does not promise a finished backend, linker, or runtime in
  this chapter

## Stable codes

The current diagnostics model carries stable producer-owned codes.

The important idea is not the exact prefix letters.
The important idea is ownership and stability:

- parser owns parser-family codes
- package loading owns package-family codes
- resolver owns resolver-family codes
- future semantic phases will own their own families as well

This is better than deriving codes from free-form message text, because message
wording can change without breaking code identity.

## What diagnostics do not guarantee

Diagnostics are strong, but they are not a substitute for the later semantic
phases.

Current limits still matter:

- parser diagnostics do not imply type checking has happened
- `report` compatibility still belongs to later semantic work
- type mismatch, coercion, and conversion diagnostics are future type-checker work
- ownership and borrowing diagnostics are future semantic work
- C ABI diagnostics are future package/type/backend work

So the current guarantee is:

- if stream, lexer, parser, package loading, or resolver can identify the
  problem now, diagnostics should be structured and exact

But not:

- all language-semantic errors already exist today

## Practical rule of thumb

When reading compiler output, think in this order:

1. trust the primary location first
2. use related labels to understand competing or earlier sites
3. read notes for technical context
4. read helps for the most actionable next step

That mental model matches how the compiler currently structures reporting.
