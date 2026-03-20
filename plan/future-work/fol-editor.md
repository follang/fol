# FOL Editor Future Work

This file holds follow-on ideas that should not block the first `fol-editor`
 milestone.

The active implementation work belongs in [`PLAN.md`](../PLAN.md).

## Future Tree-sitter Work

- injection queries if embedded syntaxes ever matter
- fold queries
- indentation helpers
- richer locals/reference capture tuning
- editor-specific whitespace/comment polish beyond the first grammar contract

## Future LSP Work

- code actions
- signature help
- workspace symbols
- multi-file rename safety
- incremental semantic invalidation instead of full-document/package reanalysis

## Future Formatting Work

- a formatter integrated through `fol tool format`
- formatting range support
- formatting diff/code-action support

## Future Frontend Exposure

- `fol tool rename`
- `fol tool format`

These should wait until the underlying editor features are real.

## Future Editor Ecosystem Work

- packaged Tree-sitter releases
- packaged LSP binaries/releases
