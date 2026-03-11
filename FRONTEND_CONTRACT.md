# Front-End Contract

This file records the current stream, lexer, and parser contracts that the code and
tests actually enforce today.

## Lexer Stage Ownership

- `stage0`: consumes the unified character stream, preserves source locations, and
  injects only the minimum synthetic boundary and EOF markers needed for stable
  downstream tokenization.
- `stage1`: performs first-pass token classification from characters into the initial
  token families.
- `stage2`: folds and normalizes the classified token stream, including multi-character
  operators and separator cleanup.
- `stage3`: performs the final parser-facing disambiguation, especially around numeric
  literal forms and explicit EOF behavior.
