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

- multi-package rename safety
- incremental semantic invalidation instead of full-document/package reanalysis

## Future Formatting Work

- formatting range support
- formatting diff/code-action support

## Future Editor Ecosystem Work

- packaged Tree-sitter releases
- packaged LSP binaries/releases
