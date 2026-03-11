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

## Stream Contract

### Source Ordering

- Folder traversal is deterministic.
- Directory entries are processed in lexicographic filename order.
- Regular directories are traversed recursively in that same sorted order.
- `.mod` directories are skipped before any source collection.
- The lexer now preserves that stream ordering across file boundaries instead of
  accidentally joining touching files into one token.

### Source Identity

- A stream source is identified by its canonical file path plus the package and
  namespace chosen for the current run.
- The original call site is preserved separately so file discovery mode can still
  be reported without changing logical identity.

### Package Detection

- Detached folders fall back to their own folder name as the package name.
- Detached files fall back to their parent folder name as the package name.
- Nested manifests use the nearest `Cargo.toml` package name, not the outermost one.
- Explicit package overrides intentionally change logical identity without changing
  the canonical source path.

### Namespace Derivation

- Single-file entry keeps the root namespace even when the file lives in nested folders.
- Folder entry derives namespace segments from nested directories under the chosen root.
- Invalid namespace components are ignored instead of aborting source discovery.
- Valid components may include underscores and non-leading digits.
- `.mod` directories do not contribute sources or namespace segments.

### Location Guarantees

- Rows and columns are one-based for real source characters.
- Carriage return advances the column; line feed advances the row and resets the column.
- Switching to a new source restarts location tracking at row `1`, column `1`.
- Synthetic lexer-only markers use explicit out-of-band coordinates instead of pretending
  to be real source characters.

## Lexer Contract

### Token Payload Meaning

- Keywords, identifiers, symbols, and folded operators keep their source spelling.
- Numeric literal payloads preserve the original spelling, including supported prefixes
  and underscores.
- Quoted literal payloads keep their delimiters.
- Ignorable separators normalize to a single-space payload.
- EOF keeps an explicit `\0` sentinel and may carry only normalized trailing separator
  payload ahead of that sentinel.

### Literal Categories

- The current lexer surfaces `Stringy`, `Bool`, `Float`, `Deciaml`, `Hexal`, `Octal`,
  and `Binary`.
- Single-quoted and double-quoted forms both arrive at the lexer boundary as `Stringy`.
- Backticks stay `Operator::ANY` until the language gives them a narrower meaning.
- Imaginary-unit suffixes are out of scope and stay outside the supported numeric
  literal families.

### Comment Policy

- Ordinary comments are fully ignorable by the parser-facing lexer output.
- Doc-comment spellings follow the same path as ordinary comments and are explicitly
  deferred instead of surfacing as a separate token family.

### Malformed-Input Policy

- Unterminated quoted content becomes an `Illegal` token instead of a hard lexer error.
- Invalid-looking escape spellings are preserved verbatim inside quoted payloads.
- Raw unrecognized characters still raise a lexer error instead of being silently
  converted into tokens.

## Parser Contract

### Literal Lowering Guarantees

- Parser-supported literal lowering currently covers strings, booleans, `nil`, decimal
  integers, floats, hex, octal, and binary integers.
- Double-quoted content always lowers to `Literal::String`.
- Single-quoted one-character content lowers to `Literal::Character`.
- Wider single-quoted content lowers to `Literal::String`.
- Supported prefixed and underscored numeric spellings lower to their exact integer or
  float values instead of staying as raw text in the AST.
- End-to-end tests now lock this behavior across the full `stream -> lexer -> parser`
  path, not just through direct parser helpers.
