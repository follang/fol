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
- The current quoted-literal split is stage1-owned: later stages preserve that family
  distinction instead of rediscovering delimiter intent.
- No current lexer contract depends on hidden reclassification overlap between stages;
  the remaining stage interactions are explicit token-stream refinement boundaries.

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

- Package identity is defined by the explicit package override when one is
  provided; otherwise it is derived from the explicit entry root used for source
  discovery.
- Detached folders fall back to their own folder name as the package name.
- Detached files fall back to their parent folder name as the package name.
- Nested `Cargo.toml` files are ignored by the default front-end package
  algorithm.
- If the entry root has no usable final path segment, the deterministic fallback
  package name is `root`.
- Explicit package overrides intentionally change logical identity without changing
  the canonical source path.

### Namespace Derivation

- Single-file entry keeps the detached-file root namespace even when the file
  lives in deeper nested folders.
- Folder entry derives namespace segments from nested directories under the chosen root.
- Namespace validation is ASCII-only and preserves the original component spelling.
- Valid components may include mixed case, a standalone `_`, leading underscores,
  underscores inside the name, and non-leading digits.
- Invalid components include dots, hyphens, leading digits, repeated underscore
  runs, and non-ASCII path segments.
- Invalid namespace components are skipped instead of aborting source discovery.
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

### Identifier And Keyword Edges

- Keyword recognition is exact-case only.
- Case variants such as `Fun` and `LOG` stay ordinary identifiers instead of being
  normalized into keyword tokens.
- Repeated underscore runs inside identifiers lower to `Illegal`.
- A standalone `_` still lowers as an identifier token because the current parser uses
  that surface for silent binders and destructuring-rest forms.
- Leading single underscores remain ordinary identifiers under the current contract.

### Literal Categories

- The current lexer surfaces `Stringy`, `Quoted`, `Bool`, `Float`, `Decimal`, `Hexadecimal`,
  `Octal`, and `Binary`.
- Double-quoted content arrives at the lexer boundary as `Stringy`.
- Single-quoted content arrives at the lexer boundary as `Quoted`.
- The current front end does not expose separate raw-vs-cooked literal token kinds;
  backticks still stay outside the literal taxonomy entirely.
- Backticks stay `Operator::ANY` until the language gives them a narrower meaning.
- Imaginary-unit suffixes are out of scope and stay outside the supported numeric
  literal families.

### Comment Policy

- Ordinary comments are fully ignorable by the parser-facing lexer output.
- Doc-comment spellings follow the same path as ordinary comments and are explicitly
  deferred instead of surfacing as a separate token family.

### Malformed-Input Policy

- Unterminated single-quoted, double-quoted, and backtick-delimited content all become
  the same parser-visible `Illegal` token instead of a hard lexer error.
- Invalid-looking escape spellings are preserved verbatim inside quoted payloads.
- Physical newlines inside quoted content stay inside the same token payload; the lexer
  does not apply a separate line-continuation rule at this boundary.
- The parser-facing split is stable: malformed quoted spans become `Illegal`, while raw
  unsupported characters still stop lexing with an error.
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

### Name And Path Encoding

- Bare names and keyword-like names normalize to the same plain string form when the
  parser accepts them as labels.
- Quoted names also normalize to plain, unquoted strings in AST fields such as
  declaration names and binding names.
- Qualified value paths are currently encoded as `::`-joined strings inside the value
  nodes that carry names, such as `Identifier { name }` and `FunctionCall { name }`.
- Qualified type paths are currently encoded as `FolType::Named { name }` with the full
  normalized `::`-joined path string.
- `use` declarations keep their import path text in the dedicated `path` field instead of
  reusing the value-path or type-path encoding.

### Current Root Shape

- `AstNode::Program { declarations }` is the single parser root.
- `Program.declarations` contains real top-level declarations and top-level lowered
  statements or expressions that the parser accepts at file scope.
- Top-level `fun`, `log`, and `pro` declarations now stay as single root declaration
  nodes instead of leaking their body statements into the program root.
- Nested routine declarations, type members, standards, implementations, and other
  nested bodies keep their child nodes inside their own body fields instead of leaking
  to the program root.

### Routine Body Shape

- `FunDecl.body`, `LogDecl.body`, and `ProDecl.body` are the authoritative routine-body
  fields for both top-level and nested routines.
- Routine bodies contain the statement and lowered-expression nodes accepted by the
  body parsers, including local bindings, returns, control-flow, calls, and other
  body-level forms that the current grammar supports.

### Declaration Family Shapes

- `fun`, `log`, and `pro` declarations share the same high-level shape: options,
  generics, captures, parameters, optional return and error types, a `body`, and
  `inquiries`.
- `log` declarations now lower through a dedicated `AstNode::LogDecl` node instead of
  being collapsed into `FunDecl`, so routine kind survives AST lowering for named
  routines.
- `AliasDecl` stays a leaf declaration with only the alias name and target type.
- `TypeDecl` is the single carrier for alias, entry, record, and other type-definition
  families through the `type_def` field.
- `StdDecl`, `ImpDecl`, `DefDecl`, and `SegDecl` keep nested declarations inside their
  dedicated `body` fields instead of flattening those members into wrapper-specific
  side channels.
- Grouped binding and grouped type forms expand into ordinary sibling declaration nodes
  rather than producing wrapper nodes that later phases would need to unwrap.
- The currently unsupported declaration-family mixes also fail explicitly instead of
  collapsing into incidental parse shape:
  multi-name type declarations reject generic headers, explicit contract headers, and
  mismatched definition counts with dedicated parse errors.

### Grouped Declaration Invariants

- Grouped binding forms expand into multiple sibling declaration nodes in source order.
- Shared binding options apply to every expanded declaration in the grouped result.
- Shared values and parallel values preserve the declared name order when the parser
  lowers grouped bindings.
- Grouped and multi-name type declarations expand into multiple sibling `TypeDecl`
  nodes instead of keeping a wrapper node in the AST.
- Shared object-style type definitions are cloned per declared type name.
- Multi-name type declarations currently reject generic headers, explicit contract
  headers, and mismatched definition counts with explicit parse errors instead of
  silently guessing a shape.

### Method Receiver Retention

- Named `fun`, `log`, and `pro` declarations retain their parsed `receiver_type`
  directly in the AST.
- Receiver types survive at both top level and inside nested type-member routine
  declarations.
- Qualified and bracketed receiver type references stay lowered through the same
  `FolType` surfaces already used by the rest of the parser.

### Parser-Owned Validations

- The parser rejects duplicate and conflicting declaration options where those checks are
  already encoded in parser-side option handling.
- The parser rejects duplicate names in surfaces such as `use` declarations when they are
  repeated within the same declaration.
- The parser rejects duplicate inquiry targets and duplicate type members in the grammar
  surfaces that already track those sets during parsing.
- Name resolution, whole-program type checking, ownership rules, and cross-file semantic
  validation are still outside the parser contract.

### Current Failure-Shape Consistency

- Parser-owned unknown-option diagnostics name the surface that rejected the option, such
  as `use`, `implementation`, `standard`, `type`, `routine`, `binding`, `definition`,
  and `segment`.
- Duplicate and conflicting parser-owned option diagnostics also stay surface-specific
  instead of falling back to a generic parse failure.
- The currently unsupported multi-name type combinations are rejected explicitly:
  generics and explicit contracts are limited to single-name type declarations, and
  mismatched definition counts report a dedicated error instead of a later shape failure.
- That means the currently known unsupported declaration-family combinations fail
  intentionally and early rather than being inferred later from malformed AST shape.
- Representative missing-close diagnostics consistently use `Expected closing ...`
  language, although the exact trailing context is still shape-specific.
- Representative `Expected X` diagnostics also name the missing syntactic shape directly,
  such as missing routine names or missing assignment-target fields, instead of falling
  back to an opaque generic parse failure.

### Parser Boundary

- Structural parsing work stays in the parser:
  - declaration and statement shape recognition
  - delimiter and separator matching
  - duplicate/conflicting parser-owned option rejection
  - AST lowering for literals, names, paths, and other grammar-owned forms
- Deferred work remains outside the parser boundary:
  - whole-program name resolution
  - whole-program type checking
  - ownership and borrowing analysis
  - cross-file semantic validation

### Parser Module Ownership

- `parser_parts` ownership is now explicit enough to maintain without broad structural
  refactoring:
  - program/root assembly lives in `program_and_bindings`
  - literal atom lowering lives in `expression_atoms_and_report_validation`
  - statement/expression boundary behavior is locked by focused parser tests instead of
    depending on cross-module guesswork
- The remaining overlap hotspots are documented rather than hidden:
  - routine surfaces still touch both header parsing and body parsing modules
- No additional parser-part reshuffle is required for this hardening pass because the
  current ownership boundaries are stable enough to support targeted maintenance.

### Hardening Boundary Freeze

- During this hardening pass, parser-side semantic-adjacent `report` validation and
  routine-signature pre-scanning were removed instead of expanded.
- That keeps the parser boundary narrower than before this pass: new hardening work has
  stayed on syntax shape, AST invariants, token/literal continuity, and diagnostic
  consistency instead of growing parser-side semantic reach.

### Next-Stage Handoff

- A next-stage consumer can now rely on the front-end without reverse-engineering
  implementation quirks:
  - stream ordering and source identity are explicit
  - lexer payloads and literal families are explicit
  - parser root shape, declaration-family shape, and supported literal lowering are
    explicit
  - representative parser failure modes are explicit and test-backed
- The remaining front-end debt is narrow and declared up front:
  - later semantic passes still need whole-program resolution and type analysis
- That means the next stage no longer needs to guess at front-end structure or work
  around parser-owned semantic checks.

### Statement And Expression Boundaries

- File scope currently accepts real root statements such as calls, invokes, assignments,
  loops, and `when` forms in addition to declarations.
- At both file scope and routine-body scope, a bare named callee lowers as
  `AstNode::FunctionCall`, while a grouped or otherwise computed callee lowers as
  `AstNode::Invoke`.
- Assignment parsing stays a separate statement path at both scopes; call-like targets
  are rejected instead of being reinterpreted as assignment shapes.
- A top-level `when` statement stays a root `AstNode::When` with nested body nodes instead
  of being rewritten into a declaration-like wrapper.
- `when` and matching forms used in expression position stay nested under their owner
  node, such as `VarDecl.value` or `Return.value`, instead of surfacing as sibling
  statements.

### Parser Part Ownership

- `program_and_bindings.rs`: program root assembly, top-level declaration dispatch, and
  file-scope fallback lowering for statements, calls, literals, and identifiers.
- `declaration_parsers.rs`, `use_declaration_parsers.rs`, `segment_declaration_parsers.rs`,
  `implementation_declaration_parsers.rs`, and `standard_declaration_parsers.rs`:
  declaration-family parsing for the corresponding top-level syntactic forms.
- `routine_declaration_parsers.rs`, `routine_headers_and_type_lowering.rs`,
  `routine_capture_parsers.rs`, and `routine_body_parsers.rs`: routine header parsing,
  capture parsing, and routine-local body parsing.
- `binding_alternative_parsers.rs`, `binding_option_parsers.rs`,
  `binding_value_parsers.rs`, and `grouped_binding_parsers.rs`: binding options,
  storage/visibility modifiers, grouped bindings, and binding value lowering.
- `type_definition_parsers.rs`, `grouped_type_parsers.rs`, `special_type_parsers.rs`,
  `source_kind_type_parsers.rs`, `test_type_parsers.rs`, and
  `type_references_and_blocks.rs`: type declarations, grouped type expansion, special
  type forms, source/test type forms, and general type-reference parsing.
- `expression_parsers.rs`, `expression_atoms_and_report_validation.rs`,
  `primary_expression_parsers.rs`, `postfix_expression_parsers.rs`,
  `access_expression_parsers.rs`, `pipe_expression_parsers.rs`,
  `pipe_lambda_parsers.rs`, and `rolling_expression_parsers.rs`: expression precedence,
  atoms, postfix chains, access forms, pipe forms, pipe lambdas, and rolling
  expressions.
- `statement_parsers.rs`, `flow_body_parsers.rs`, and `inquiry_clause_parsers.rs`:
  statement parsing, flow-body parsing, and inquiry-clause parsing for declarations
  and anonymous routines.

### Parser Part Overlap Risks

- `program_and_bindings.rs` is the main maintenance hotspot because it mixes root
  assembly with direct lowering of many statement and expression forms that are also
  handled inside routine bodies.
- Routine parsing is spread across header, signature, capture, declaration, and body
  parser parts; that split is workable, but it means routine-structure changes have to
  be audited across several files rather than in one place.
- Type parsing is also intentionally split across declaration, grouped-type, and
  general type-reference parsers, so declaration-shape fixes need to preserve those
  boundaries instead of collapsing everything into one parser part.

## Deferred Front-End Debt

- Later semantic phases remain out of scope for this front-end hardening pass.

## Hardening Execution Notes

- This hardening pass has not introduced new major language syntax; the work has stayed
  on contract definition, regression coverage, and front-end shape correction.
- The executed order has stayed stream first, then lexer, then parser, with later
  contract freeze work only after those boundaries were exercised by tests.
- The changes in this pass have been intentionally surgical: most slices either lock
  existing behavior with tests, document a contract explicitly, or make a contained
  front-end correction without broad grammar churn.
- Eager source loading remains accepted for this cycle, but there is now an explicit
  follow-up expectation to revisit it only after parser hardening is complete.

## Undefined Behavior Audit

- Stream behavior that was previously implicit is now explicit: ordering, source
  identity, package lookup, namespace derivation, and file-boundary locations are all
  written down and test-backed.
- Lexer behavior that was previously quirk-driven is now explicit at the front-end
  boundary: token payload meaning, EOF handling, malformed literal handling, and the
  currently supported numeric families are all documented and exercised by tests.
- Parser behavior that remains unusual is explicit rather than undefined:
  declaration-family shapes, statement/expression boundaries, and representative
  failure shapes are all recorded in this contract.
- Remaining front-end debt is therefore conscious deferred debt, not undocumented
  boundary behavior that later phases would need to rediscover from the implementation.
