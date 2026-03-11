# FOL Project Progress

Last scan: 2026-03-11
Scan basis: repository code, active tests, fixture inventory, and a full integration test run
Authority rule for this file: code and active tests win over older aspirational docs

## 0. Purpose Of This File

- This file is meant to answer one question: what is actually implemented right now.
- This file is not a language manifesto.
- This file is not the book.
- This file is not a wishlist.
- This file is a repo-backed status ledger.
- Every section below was rebuilt from the current workspace contents.
- When this file says "implemented", that means code exists in the repo today.
- When this file says "covered", that means there is at least one active test or fixture for it.
- When this file says "partial", that means syntax may exist while semantics/runtime/backend work is still missing.
- When this file says "missing", that means I did not find real implementation support in the code scan.

## 1. Scan Method

- Scanned the workspace file inventory with `find` and the active workspace manifests.
- Scanned all active Rust source modules under `fol-types`, `fol-stream`, `fol-lexer`, `fol-parser`, `fol-diagnostics`, and `src`.
- Scanned all active tests under `test/`.
- Counted active parser fixtures under `test/parser/simple_*.fol`.
- Read the parser AST definition, parser entry point, and every parser part module.
- Read the lexer token inventory and every lexer stage implementation.
- Read the stream crate and diagnostics crate.
- Read the root CLI entry point.
- Ran the root validation commands: `make build` and `make test`.
- Observed current integration status directly from the test run output.
- Did not rely on the mdBook, `README.md`, or `FEATURES.md` as implementation truth.

## 2. Snapshot Metrics

- Workspace member crates: `5`
- Root binary crate: `1`
- Active Rust source lines scanned: `17360`
- Core compiler Rust lines scanned:
- `fol-types`: `228`
- `fol-stream`: `480`
- `fol-lexer`: `2115`
- `fol-parser`: `14011`
- `fol-diagnostics`: `267`
- Root CLI: `259`
- Active parser fixtures: `1057`
- Active lexer tests: `36`
- Active stream tests: `43`
- Active parser/integration tests in `integration`: `1018`
- Observed active integration failures during the run: `0`
- Observed active integration run shape: suite is green

## 3. Current Headline Status

- `fol-stream`: implemented, hardened, and actively used
- `fol-lexer`: implemented, hardened, and actively used
- `fol-parser`: large surface implemented, structurally hardened, and actively used
- `fol-diagnostics`: implemented and actively used
- Root CLI: implemented as parse-and-report driver
- Semantic analysis pass: missing
- Whole-program name resolution: missing
- Whole-program type checking: missing
- Ownership/borrowing semantics enforcement: missing
- Standard/protocol conformance checking: missing
- Interpreter/runtime: missing
- Code generation/backend: missing
- Package/module system beyond source enumeration and syntax: partial
- Documentation alignment with implementation: materially improved for the front-end contract, still mixed elsewhere

## 4. Current Known Failures

- No active test failures were observed in the current validation run.
- Current validation result:
- `make build` passed
- `make test` passed
- test totals observed:
- `1` unit test
- `1018` integration tests
- The main remaining front-end debt is no longer parser-boundary ambiguity.
- The remaining missing work is later-phase compiler work, not front-end hardening.

## 5. Current Big Picture

- The project has a real front-end pipeline now.
- The pipeline is:
- `fol-stream` -> `fol-lexer` stage0/stage1/stage2/stage3 -> `fol-parser` -> `fol-diagnostics`
- The parser is no longer "minimal".
- The parser surface is broad enough that the real bottleneck is no longer basic syntax support.
- The top-level parser root shape is now explicit and test-backed:
- `Program.declarations` no longer leaks routine body nodes for top-level `fun`, `log`, or `pro`.
- The main missing systems are later-phase systems:
- semantic analysis
- type checking
- conformance checking
- package/module resolution
- lowering
- runtime/backend work
- Front-end docs are now much closer to code truth:
- [FRONTEND_CONTRACT.md](/home/bresilla/data/code/bresilla/fol/FRONTEND_CONTRACT.md) records the active stream, lexer, and parser contracts
- `PLAN.md` records the completed hardening work
- older aspirational docs are still behind the code in places

## 6. Workspace Inventory

### 6.1 Root Workspace Files

- `Cargo.toml`: active workspace root and binary/test manifest
- `src/main.rs`: active CLI entry point
- `README.md`: stale relative to parser surface
- `FEATURES.md`: stale relative to parser surface
- `CHANGELOG.md`: present
- `PLANNING.md`: present
- `PLAN.md`: present
- `PROGRESS.md`: this file, rebuilt from code
- `test_old/`: archival material, not the active integration suite

### 6.2 Active Crates

- `fol-types`
- `fol-stream`
- `fol-lexer`
- `fol-parser`
- `fol-diagnostics`

### 6.3 Active Test Entry Point

- Root integration target:
- `[[test]] name = "integration" path = "test/run_tests.rs"`

## 7. fol-types Status

### 7.1 Files

- `fol-types/src/lib.rs` (`44` lines)
- `fol-types/src/mod.rs` (`76` lines)
- `fol-types/src/error.rs` (`108` lines)

### 7.2 What Exists

- Shared `Glitch` trait exists.
- Boxed error cloning exists via `clone_box`.
- Dynamic downcasting support exists via `as_any`.
- Basic boxed error type exists as `BasicError`.
- Type aliases exist:
- `Win<T>`
- `Con<T>`
- `Vod`
- `List<T>`
- `Errors`
- Global slider constant exists:
- `SLIDER = 9`
- Error enums exist:
- `Flaw`
- `Typo`
- `Slip`
- Display implementations exist for all three enums.
- `std::error::Error` implementations exist for all three enums.
- `Glitch` implementations exist for:
- `BasicError`
- `Flaw`
- `Typo`
- `Slip`
- Legacy helper macros exist:
- `catch!`
- `crash!`
- `halt!`
- `erriter!`
- `noditer!`
- `logit!`

### 7.3 What This Means

- The shared error foundation is present.
- The current system is enough for:
- lexer errors
- parser errors
- diagnostics conversion
- boxed trait-object error transport
- The crate is small, but real.

### 7.4 What Is Not There Yet

- Structured error domains beyond a few early enums
- Rich error metadata models
- Stable machine-readable compiler error taxonomy
- Dedicated semantic/type-system error families
- More precise backend/runtime error types
- Modernized macro cleanup

### 7.5 Status Call

- `fol-types`: implemented
- Depth: foundational but still early
- Risk: medium, because later compiler phases will likely need a larger error model

## 8. fol-stream Status

### 8.1 File

- `fol-stream/src/lib.rs` (`456` lines)

### 8.2 Core Structures

- `CharacterProvider` trait exists
- `StreamSource` trait exists
- `Location { row, col, file }` exists
- `Source { call, path, data, namespace, package }` exists
- `SourceType { File, Folder }` exists
- `FileStream` exists

### 8.3 Implemented Capabilities

- `Source::init` exists
- `Source::init_with_package` exists
- `Source::path(abs)` exists
- `Source::module()` exists
- `FileStream::from_file` exists
- `FileStream::from_folder` exists
- `FileStream::from_sources` exists
- `FileStream::current_source` exists
- `FileStream::sources` exists
- `FileStream` implements `CharacterProvider`
- Path validation exists
- Canonicalization exists
- Package-name detection from `Cargo.toml` exists
- Namespace derivation from directory structure exists
- Recursive folder traversal exists
- `.fol` filtering exists
- `.mod` directory skipping exists
- Per-file location reset exists when moving between sources
- Multi-file streaming exists
- Source contents are eagerly loaded

### 8.4 Real Behavior Confirmed In Code

- Directories ending with `.mod` are skipped during recursive folder traversal.
- Non-`.mod` directories are traversed recursively.
- Only `.fol` files are included by `from_dir`.
- When a file path is supplied while a folder source is expected, parent-directory fallback exists.
- Package detection walks upward until it finds `Cargo.toml`.
- If parsing `Cargo.toml` fails to find a package name, the crate returns an error.
- Namespace computation prepends the detected package name.
- Namespace components reject dots and leading digits.
- The file stream walks sources sequentially and resets row/column when switching files.

### 8.5 Test Coverage Present

- `test/stream/test_stream.rs`: `10` tests
- `test/stream/test_namespace.rs`: `8` tests
- `test/stream/test_mod_handling.rs`: `7` tests
- Integration harness also exercises stream-to-lexer handoff

### 8.6 What Remains

- Lazy loading rather than eager full-file reads
- Better module/package semantics for `.mod` trees
- Stronger package manifest parsing
- Stable ordering guarantees across recursive file discovery
- Better error typing than generic `BasicError`
- More explicit support for non-Cargo package layouts

### 8.7 Status Call

- `fol-stream`: implemented
- Coverage: good for current scope
- Limitation: this is still source discovery/streaming, not a true package resolver

## 9. fol-lexer Status

### 9.1 Files

- `fol-lexer/src/lib.rs` (`30` lines)
- `fol-lexer/src/point.rs` (`135` lines)
- `fol-lexer/src/token/mod.rs` (`173` lines)
- `fol-lexer/src/token/buildin/mod.rs` (`130` lines)
- `fol-lexer/src/token/literal/mod.rs` (`43` lines)
- `fol-lexer/src/token/operator/mod.rs` (`69` lines)
- `fol-lexer/src/token/symbol/mod.rs` (`95` lines)
- `fol-lexer/src/token/void/mod.rs` (`35` lines)
- `fol-lexer/src/token/help.rs` (`53` lines)
- `fol-lexer/src/lexer/stage0/elements.rs` (`129` lines)
- `fol-lexer/src/lexer/stage1/element.rs` (`321` lines)
- `fol-lexer/src/lexer/stage1/elements.rs` (`129` lines)
- `fol-lexer/src/lexer/stage2/element.rs` (`180` lines)
- `fol-lexer/src/lexer/stage2/elements.rs` (`170` lines)
- `fol-lexer/src/lexer/stage3/element.rs` (`146` lines)
- `fol-lexer/src/lexer/stage3/elements.rs` (`188` lines)

### 9.2 Pipeline Reality

- Stage 0 exists as raw character windowing over `FileStream`.
- Stage 1 exists as first token classification.
- Stage 2 exists as token normalization and multi-operator folding.
- Stage 3 exists as number normalization and parser-facing token stream.
- The active parser consumes stage 3.

### 9.3 Location Tracking

- `point::Source` exists
- `point::Location` exists
- Row tracking exists
- Column tracking exists
- Token length tracking exists
- Bracket depth tracking exists
- Stream-to-point conversion exists
- Source-path carry-through exists
- Visual source rendering support exists in `visualize`

### 9.4 Token Families Present

- `KEYWORD::Literal`
- `KEYWORD::Keyword`
- `KEYWORD::Symbol`
- `KEYWORD::Operator`
- `KEYWORD::Void`
- `KEYWORD::Identifier`
- `KEYWORD::Comment`
- `KEYWORD::Illegal`

### 9.5 Lexer Helper Predicates Present

- `is_assign`
- `is_ident`
- `is_literal`
- `is_buildin`
- `is_illegal`
- `is_comment`
- `is_open_bracket`
- `is_close_bracket`
- `is_bracket`
- `is_decimal`
- `is_number`
- `is_numberish`
- `is_symbol`
- `is_operator`
- `is_void`
- `is_eof`
- `is_space`
- `is_eol`
- `is_nonterm`
- `is_terminal`
- `is_dot`
- `is_comma`
- `is_continue`

### 9.6 Lexed Builtin Keyword Inventory

- `ANY`: placeholder enum member exists; not a user-visible surface by itself
- `use`: lexed
- `def`: lexed
- `seg`: lexed
- `var`: lexed
- `log`: lexed
- `con`: lexed
- `fun`: lexed
- `pro`: lexed
- `typ`: lexed
- `std`: lexed
- `ali`: lexed
- `imp`: lexed
- `lab`: lexed
- `not`: lexed
- `or`: lexed
- `xor`: lexed
- `nor`: lexed
- `and`: lexed
- `nand`: lexed
- `as`: lexed
- `cast`: lexed
- `if`: lexed
- `else`: lexed
- `when`: lexed
- `loop`: lexed
- `is`: lexed
- `at`: enum/display exists; parser-side language use is still limited
- `has`: lexed
- `in`: lexed
- `on`: lexed
- `case`: lexed
- `this`: lexed
- `self`: lexed as `Selfi`
- `break`: lexed
- `return`: lexed
- `yeild`: lexed with the misspelled spelling
- `panic`: lexed
- `report`: lexed
- `check`: lexed
- `assert`: lexed
- `where`: lexed
- `true`: lexed
- `false`: lexed
- `each`: lexed
- `for`: lexed
- `do`: lexed
- `go`: lexed
- `get`: lexed
- `of`: lexed
- `let`: lexed

### 9.7 Literal Families Present

- `Stringy`
- `Bool`
- `Float`
- `Decimal`
- `Hexadecimal`
- `Octal`
- `Binary`

### 9.8 Operator Inventory Present

- `...` -> `Dotdotdot`
- `..` -> `Dotdot`
- `::` -> `Path`
- `:=` -> `Assign`
- `=>` -> `Flow`
- `->` -> `Flow2`
- `+` -> `Add`
- `-` -> `Abstract`
- `*` -> `Multiply`
- `/` -> `Divide`
- `==` -> `Equal`
- `!=` -> `Noteq`
- `>=` -> `Greateq`
- `<=` -> `Lesseq`
- `+=` -> `Addeq`
- `-=` -> `Subeq`
- `*=` -> `Multeq`
- `/=` -> `Diveq`
- `<<` -> `Lesser`
- `>>` -> `Greater`

### 9.9 Symbol Inventory Present

- `(` -> `RoundO`
- `)` -> `RoundC`
- `[` -> `SquarO`
- `]` -> `SquarC`
- `{` -> `CurlyO`
- `}` -> `CurlyC`
- `<` -> `AngleO`
- `>` -> `AngleC`
- `.` -> `Dot`
- `,` -> `Comma`
- `:` -> `Colon`
- `;` -> `Semi`
- `\` -> `Escape`
- `|` -> `Pipe`
- `=` -> `Equal`
- `+` -> `Plus`
- `-` -> `Minus`
- `_` -> `Under`
- `*` -> `Star`
- `~` -> `Home`
- `/` -> `Root`
- `%` -> `Percent`
- `^` -> `Carret`
- `?` -> `Query`
- `!` -> `Bang`
- `&` -> `And`
- `@` -> `At`
- `#` -> `Hash`
- `$` -> `Dollar`
- `°` -> `Degree`
- `§` -> `Sign`
- `` ` `` -> `Tik`

### 9.10 Void Inventory Present

- `Space`
- `EndLine`
- `EndFile`

### 9.11 Stage 0 Status

- Sliding-window character iteration exists
- Current/peek/seek window helpers exist
- Previous-window helpers exist
- EOF marker injection exists
- Stream location conversion exists
- Safety buffer for post-EOF bumping exists

### 9.12 Stage 1 Status

- Comment detection exists
- Line comment handling exists
- Block comment handling exists
- EOF handling exists
- Newline handling exists
- Space handling exists
- Decimal number detection exists
- Hex literal detection exists
- Octal literal detection exists
- Binary literal detection exists
- Encapsulated literal handling exists for:
- double quotes
- single quotes
- backticks
- Symbol classification exists
- Alpha/identifier scanning exists
- Keyword lowering from raw text exists
- Unknown characters raise `Flaw::ReadingBadContent`

### 9.13 Stage 1 Caveats

- Character literal support is not cleanly separate from string handling
- Backticks become operator-ish `ANY` behavior rather than a finished surface
- Comments are normalized into space-like output
- Some token naming is still legacy and misspelled

### 9.14 Stage 2 Status

- Adjacent EOL compression exists
- EOL-to-space handling around nonterminal contexts exists
- Space/EOL-to-EOF normalization exists
- Multi-character operator folding exists
- Identifier carry-through exists
- Windowing with optional space skipping exists
- Jump/eat/until-term helpers exist for parser use

### 9.15 Stage 3 Status

- Negative-number folding exists
- Float folding exists
- Number-vs-identifier normalization exists
- Detection of missing space around number/dot patterns exists
- `Typo::LexerSpaceAdd` emission exists
- Parser-facing stage window helpers exist
- Parser-side `set_key` override exists

### 9.16 Active Lexer Tests

- token smoke tests exist
- keyword tests exist
- identifier tests exist
- literal tests exist
- operator tests exist
- symbol tests exist
- comment tests exist
- mixed-token tests exist
- token location tests exist
- file error path tests exist

### 9.17 What Remains In The Lexer

- clean up misspellings:
- `yeild`
- `Hexadecimal`
- maybe other legacy enum spellings
- decide what backticks actually mean
- separate char literal handling cleanly from string literal handling
- decide whether keyword registry helper `get_keyword()` should be real or removed
- add more precise illegal-token tests
- add more direct negative tests that do not panic via `.expect(...)`
- rationalize legacy enum names like `Abstract` for minus

### 9.18 Status Call

- `fol-lexer`: implemented
- Current maturity: real and usable
- Main debt: naming cleanup, edge-case cleanup, and spec alignment

## 10. fol-parser Status

### 10.1 Files

- `fol-parser/src/lib.rs` (`6` lines)
- `fol-parser/src/ast/mod.rs` (`909` lines)
- `fol-parser/src/ast/parser.rs` (`136` lines)
- `fol-parser/src/ast/parser_parts/access_expression_parsers.rs` (`332` lines)
- `fol-parser/src/ast/parser_parts/binding_alternative_parsers.rs` (`76` lines)
- `fol-parser/src/ast/parser_parts/binding_option_parsers.rs` (`149` lines)
- `fol-parser/src/ast/parser_parts/binding_value_parsers.rs` (`82` lines)
- `fol-parser/src/ast/parser_parts/declaration_option_parsers.rs` (`133` lines)
- `fol-parser/src/ast/parser_parts/declaration_parsers.rs` (`752` lines)
- `fol-parser/src/ast/parser_parts/expression_atoms_and_report_validation.rs` (`766` lines)
- `fol-parser/src/ast/parser_parts/expression_parsers.rs` (`871` lines)
- `fol-parser/src/ast/parser_parts/grouped_binding_parsers.rs` (`79` lines)
- `fol-parser/src/ast/parser_parts/implementation_declaration_parsers.rs` (`76` lines)
- `fol-parser/src/ast/parser_parts/inquiry_clause_parsers.rs` (`354` lines)
- `fol-parser/src/ast/parser_parts/pipe_expression_parsers.rs` (`91` lines)
- `fol-parser/src/ast/parser_parts/pipe_lambda_parsers.rs` (`85` lines)
- `fol-parser/src/ast/parser_parts/postfix_expression_parsers.rs` (`90` lines)
- `fol-parser/src/ast/parser_parts/primary_expression_parsers.rs` (`413` lines)
- `fol-parser/src/ast/parser_parts/program_and_bindings.rs` (`1010` lines)
- `fol-parser/src/ast/parser_parts/rolling_expression_parsers.rs` (`182` lines)
- `fol-parser/src/ast/parser_parts/routine_capture_parsers.rs` (`83` lines)
- `fol-parser/src/ast/parser_parts/routine_headers_and_type_lowering.rs` (`955` lines)
- `fol-parser/src/ast/parser_parts/segment_declaration_parsers.rs` (`82` lines)
- `fol-parser/src/ast/parser_parts/source_kind_type_parsers.rs` (`95` lines)
- `fol-parser/src/ast/parser_parts/standard_declaration_parsers.rs` (`488` lines)
- `fol-parser/src/ast/parser_parts/statement_parsers.rs` (`885` lines)
- `fol-parser/src/ast/parser_parts/test_type_parsers.rs` (`79` lines)
- `fol-parser/src/ast/parser_parts/type_definition_parsers.rs` (`263` lines)
- `fol-parser/src/ast/parser_parts/type_references_and_blocks.rs` (`939` lines)
- `fol-parser/src/ast/parser_parts/use_declaration_parsers.rs` (`199` lines)
- `fol-parser/src/ast/parser_parts/use_option_parsers.rs` (`112` lines)

### 10.2 Parser Entry Point Reality

- `AstParser` exists
- `ParseError` exists
- Parser stores `routine_return_types` in a mutable registry
- Parser has a `new()` constructor
- Parser has `Default`
- Parser returns `Result<AstNode, Vec<Box<dyn Glitch>>>`
- Parser performs an initial same-file routine-signature scan before the main parse
- Parser uses the scan to support forward report-validation checks

### 10.3 AST Node Inventory: Declarations

- `VarDecl`
- `FunDecl`
- `ProDecl`
- `TypeDecl`
- `UseDecl`
- `AliasDecl`
- `DefDecl`
- `SegDecl`
- `ImpDecl`
- `StdDecl`

### 10.4 AST Node Inventory: Expressions

- `BinaryOp`
- `UnaryOp`
- `FunctionCall`
- `Invoke`
- `AnonymousFun`
- `AnonymousPro`
- `MethodCall`
- `IndexAccess`
- `SliceAccess`
- `PatternAccess`
- `AvailabilityAccess`
- `FieldAccess`
- `Identifier`
- `Literal`
- `ContainerLiteral`
- `Rolling`
- `Range`

### 10.5 AST Node Inventory: Statements

- `Assignment`
- `LabDecl`
- `When`
- `Loop`
- `Return`
- `Break`
- `Yield`
- `Block`
- `Inquiry`
- `Program`

### 10.6 FolType Inventory

- `Int`
- `Float`
- `Char`
- `Bool`
- `Array`
- `Vector`
- `Sequence`
- `Matrix`
- `Set`
- `Map`
- `Record`
- `Entry`
- `Optional`
- `Multiple`
- `Any`
- `Pointer`
- `Error`
- `None`
- `Function`
- `Generic`
- `Module`
- `Block`
- `Test`
- `Path`
- `Url`
- `Location`
- `Standard`
- `Named`

### 10.7 Additional AST Support

- `StandardKind`: implemented
- `DeclOption`: implemented
- `IntSize`: implemented
- `FloatSize`: implemented
- `CharEncoding`: implemented
- `ContainerType`: implemented
- `Parameter`: implemented
- `Generic`: implemented
- `RecordFieldMeta`: implemented
- `EntryVariantMeta`: implemented
- `TypeDefinition`: implemented
- `BinaryOperator`: implemented
- `UnaryOperator`: implemented
- `VarOption`: implemented
- `FunOption`: implemented
- `TypeOption`: implemented
- `UseOption`: implemented
- `WhenCase`: implemented
- `LoopCondition`: implemented
- `RollingBinding`: implemented
- AST `children()` traversal exists
- AST `get_type()` helper exists
- AST visitor trait exists

### 10.8 Top-Level Parse Surfaces Confirmed

- top-level `var`: implemented
- top-level `let`: implemented
- top-level `con`: implemented
- top-level `lab`: implemented
- top-level `use`: implemented
- top-level `seg`: implemented
- top-level `imp`: implemented
- top-level `std`: implemented
- top-level `def`: implemented
- top-level `ali`: implemented
- top-level `typ`: implemented
- top-level `fun`: implemented
- top-level `log`: implemented
- top-level `pro`: implemented
- top-level `return`: implemented
- top-level calls: implemented
- top-level general invoke: implemented
- top-level `break`: implemented
- top-level `yeild`: implemented
- top-level builtin diagnostics: implemented
- top-level `when`: implemented
- top-level `if`: implemented
- top-level `loop`/`for`/`each`: implemented
- top-level assignments: implemented

### 10.9 Parser Quirk Observed In Main Entry

- When top-level `fun`, `log`, or `pro` parses successfully, the parser currently pushes routine body statements into `Program.declarations` before pushing the declaration node itself.
- This is real current behavior.
- This is almost certainly temporary/quirky rather than a finished AST design.
- Remaining work:
- decide whether `Program` should only contain declarations/statements actually written at root
- remove this flattening once downstream phases exist

## 11. Parser Surface: Bindings

### 11.1 Plain Binding Families

- `var`: implemented
- `let`: implemented
- `con`: implemented
- `lab`: implemented

### 11.2 Binding Options

- `mut` / `mutable`: implemented
- `imu` / `immutable`: implemented
- `+` / `pub` / `exp` / `export`: implemented
- `-` / `hid` / `hidden`: implemented
- `nor` / `normal`: implemented
- `sta` / `static` / `!`: implemented
- `rac` / `reactive` / `?`: implemented
- `new`: implemented
- `bor` / `borrow` / `borrowing`: implemented

### 11.3 Binding Forms

- single binding names: implemented
- comma-separated multi-name bindings: implemented
- grouped binding blocks with `(...)`: implemented
- shared single initializer across multiple names: implemented
- one-value-per-name assignment: implemented
- explicit type hints: implemented
- implicit type omission: implemented
- optional semicolon handling: implemented

### 11.4 Binding Alternatives

- prefix `+` before `var`/`let`/`con`: implemented
- prefix `-` before `var`/`let`/`con`: implemented
- prefix `~` before `var`/`let`/`con`: implemented
- prefix `!` before `var`/`let`/`con`: implemented
- prefix `?` before `var`/`let`/`con`: implemented
- prefix `@` before `var`/`let`/`con`: implemented

### 11.5 Binding Validation Present

- option conflict detection: implemented
- duplicate/invalid option diagnostics: implemented
- grouped binding boundary diagnostics: implemented
- binding value count mismatch diagnostics: implemented
- lookahead to distinguish binding segments from value lists: implemented

### 11.6 Remaining Binding Work

- actual storage/visibility semantics beyond AST flags
- actual borrowing/new/static/reactive semantics
- semantic validation of grouped patterns beyond syntax
- name resolution and shadowing rules

## 12. Parser Surface: Imports And Source-Kind Types

### 12.1 `use` Declaration Support

- `use` keyword: implemented
- visibility options: implemented
- multiple names in one declaration: implemented
- duplicate use-name rejection: implemented
- optional colon before import type: implemented
- typed import targets: implemented
- brace-wrapped use paths: implemented
- direct bare paths: implemented
- direct quoted paths: implemented
- multiple paths: implemented
- shared single path across multiple imported names: implemented
- optional semicolon: implemented

### 12.2 Use Option Support

- export option: implemented
- hidden option: implemented
- normal option: implemented
- duplicate option diagnostics: implemented
- export/hidden conflict diagnostics: implemented

### 12.3 Source-Kind Type Support

- bare `url`: implemented
- bare `loc`: implemented
- bare `std`: implemented
- `url[...]`: implemented
- `loc[...]`: implemented
- `std[...]`: implemented
- arity checks for these suffix forms: implemented

### 12.4 Remaining Import Work

- real package/module resolution beyond syntax
- filesystem-to-`use` semantic binding
- import cycle handling
- import visibility semantics
- package-wide symbol table construction

## 13. Parser Surface: Definitions, Segments, Implementations, Standards

### 13.1 `def`

- `def` declaration parsing: implemented
- visibility options: implemented
- quoted and keyword-like names: implemented
- `mod[...]` def types: implemented
- `blk[...]` def types: implemented
- `tst[...]` def types: implemented
- bare block marker acceptance: implemented for block defs
- optional empty block body for `blk[...]`: implemented
- real body parsing with `{...}`: implemented
- semicolon after block defs: implemented
- invalid def-type rejection: implemented

### 13.2 `seg`

- `seg` declaration parsing: implemented
- visibility options: implemented
- name parsing including quoted names: implemented
- requires module type: implemented
- body parsing: implemented
- invalid non-module type rejection: implemented

### 13.3 `imp`

- `imp` declaration parsing: implemented
- visibility options: implemented
- generic header: implemented
- quoted names: implemented
- target type parsing: implemented
- body parsing: implemented

### 13.4 `std`

- standard declaration parsing: implemented
- visibility options: implemented
- kind parsing:
- `pro`
- `blu`
- `ext`
- empty kind brackets validation: implemented
- body parsing for each kind: implemented
- duplicate member detection: implemented

### 13.5 Protocol Standard Body Support

- routine signatures only: implemented
- `fun` signatures: implemented
- `log` signatures: implemented
- `pro` signatures: implemented
- bodyless signature form ending in `;`: implemented

### 13.6 Blueprint Standard Body Support

- `var` fields: implemented
- `lab` fields: implemented
- duplicate field detection: implemented

### 13.7 Extended Standard Body Support

- routine signatures: implemented
- `var` fields: implemented
- `lab` fields: implemented
- duplicate member detection: implemented

### 13.8 Remaining Work For `def` / `seg` / `imp` / `std`

- real module block semantics
- implementation attachment semantics
- standard conformance semantics
- protocol/blueprint/extended runtime meaning
- symbol resolution between declarations
- later lowering or code generation

## 14. Parser Surface: Aliases And Types

### 14.1 `ali`

- alias declarations: implemented
- quoted alias names: implemented
- qualified target types: implemented
- semicolon support: implemented

### 14.2 `typ`

- type declarations: implemented
- type options: implemented
- generic headers: implemented
- alias-like target form: implemented
- explicit entry marker form: implemented
- explicit record marker form: implemented
- direct record body form: implemented
- quoted type names: implemented

### 14.3 Type Options Present

- export: implemented
- set: implemented
- get: implemented
- nothing/non: implemented
- ext: implemented

### 14.4 Entry Type Definition Support

- requires `{...}` body: implemented
- variant labels via `var`: implemented
- variant labels via `lab`: implemented
- multiple names per variant declaration: implemented
- optional variant type: implemented
- optional default value: implemented
- metadata recording: implemented
- duplicate variant rejection: implemented

### 14.5 Record Type Definition Support

- requires `{...}` body: implemented
- optional leading `var`: implemented
- optional leading `lab`: implemented
- field names: implemented
- required field type: implemented
- optional default value: implemented
- metadata recording: implemented
- duplicate field rejection: implemented

### 14.6 Remaining Type-System Work

- actual semantic difference between many type options
- full named-generic constraint checking
- structural vs nominal typing decisions
- entry/record constructor semantics
- method attachment semantics
- real alias expansion policies

## 15. Parser Surface: Routine Headers

### 15.1 Routine Declaration Families

- `fun`: implemented
- `log`: implemented
- `pro`: implemented

### 15.2 Routine Options

- export: implemented
- mutable: implemented
- iterator: implemented

### 15.3 Method Receivers

- optional receiver syntax before routine name: implemented
- receiver type parsing: implemented
- receiver restrictions against builtin scalar types: implemented
- receiver qualification for return-type registry keys: implemented

### 15.4 Routine Names

- normal identifiers: implemented
- quoted names: implemented
- single-quoted names: implemented
- keyword-like names in supported surfaces: implemented

### 15.5 Generic Headers

- named generics: implemented
- duplicate generic rejection: implemented
- optional single constraint after `:`: implemented
- grouped generic header parsing via routine header list conversion: implemented

### 15.6 Parameter Lists

- normal typed parameters: implemented
- grouped names sharing a type: implemented
- duplicate parameter rejection: implemented
- typed defaults: implemented
- variadics using `...`: implemented
- variadic-last enforcement: implemented
- variadic-default rejection: implemented
- quoted parameter names: implemented
- keyword-like parameter names in supported surfaces: implemented

### 15.7 Borrowability Tracking

- ALL_CAPS parameter-name heuristic to mark `is_borrowable`: implemented
- This is syntactic metadata only at the moment.

### 15.8 Capture Lists

- optional capture list after params: implemented
- duplicate capture rejection: implemented
- named captures only: implemented

### 15.9 Return And Error Types

- optional return type after `:`: implemented
- optional error type after second `:`: implemented
- conflicting return-type registration detection: implemented
- overloading by arity with return registry: implemented

### 15.10 Inquiry Clauses

- `where(self) { ... }`: implemented
- `where(this) { ... }`: implemented
- duplicate inquiry target rejection: implemented
- inquiry body parsing as expression list with semicolon tolerance: implemented

### 15.11 Remaining Routine Work

- real distinction for `log` beyond current AST reuse
- semantic meaning of iterator option
- semantic meaning of borrowability
- enforcement of inquiry semantics
- body lowering into later IR

## 16. Parser Surface: Anonymous Routines And Lambdas

### 16.1 Anonymous Routines

- anonymous `fun`: implemented
- anonymous `pro`: implemented
- anonymous `log`: implemented
- options on anonymous routines: implemented
- parameters: implemented
- optional return type: implemented
- optional error type: implemented
- block body: implemented

### 16.2 Shorthand Anonymous Functions

- grouped-parameter shorthand anonymous function syntax: implemented
- body as `{...}` block: implemented

### 16.3 Pipe Lambdas

- `|x| expr`: implemented
- `|x, y| expr`: implemented
- typed lambda params: implemented
- block lambda body: implemented
- expression lambda body lowered to `Return`: implemented

### 16.4 Remaining Work

- capture syntax for pipe lambdas
- return-type inference beyond current AST defaults
- semantic closure capture analysis

## 17. Parser Surface: Type References

### 17.1 Function Types

- `{ fun (...) : T }`: implemented
- optional skipped inner function name: implemented
- default values forbidden inside function types: implemented

### 17.2 Scalar Type References

- bare `int`: implemented
- bare `float` / `flt`: implemented
- bare `bool` / `bol`: implemented
- bare `char` / `chr`: implemented
- bare fixed-size ints: implemented
- bare fixed-size uints: implemented
- bare fixed-size floats: implemented
- `int[...]`: implemented
- `flt[...]`: implemented
- `chr[...]`: implemented
- scalar option validation: implemented

### 17.3 Special Type References

- `opt[...]`: implemented
- `mul[...]`: implemented
- `ptr[...]`: implemented
- `err[...]`: implemented
- `vec[...]`: implemented
- `arr[...]`: implemented
- `mat[...]`: implemented
- `seq[...]`: implemented
- `set[...]`: implemented
- `map[...]`: implemented
- `mod[...]`: implemented
- `blk[...]`: implemented
- `tst[...]`: implemented

### 17.4 Bare Special Type Names

- bare `mod`: implemented
- bare `blk`: implemented
- bare `any`: implemented
- bare `non` / `none`: implemented
- bare source-kind types: implemented

### 17.5 Qualified Type Names

- path-style `A::B::C` names: implemented
- quoted segments in supported surfaces: implemented

### 17.6 Array And Matrix Type Argument Validation

- array requires element type + decimal size: implemented
- matrix requires element type + at least one decimal dimension: implemented
- arity errors exist: implemented

### 17.7 Test Type Arguments

- optional quoted label in `tst[...]`: implemented
- subsequent access markers: implemented
- quoted arg only allowed in label position: implemented

### 17.8 Remaining Type Reference Work

- richer generic constraint semantics
- standardized pretty-printing instead of mixed debug-like labels
- more deliberate distinction between special types and named types

## 18. Parser Surface: Expressions

### 18.1 Atom Support

- integer literals: implemented
- float literals: implemented
- string literals: implemented
- boolean literals: implemented
- character-like token path is only partial/legacy
- identifiers: implemented
- quoted identifiers in supported positions: implemented

### 18.2 Unary Support

- unary minus: implemented
- unary plus: implemented as passthrough
- unary `not`: implemented
- unary `&`: implemented
- unary `*`: implemented
- missing-operand diagnostics: implemented

### 18.3 Binary Support

- `+`: implemented
- `-`: implemented
- `*`: implemented
- `/`: implemented
- `%`: implemented
- `^`: implemented as power
- `==`: implemented
- `!=`: implemented
- `<`: implemented
- `<=`: implemented
- `>`: implemented
- `>=`: implemented
- `in`: implemented
- `has`: implemented
- `is`: implemented
- `as`: implemented
- `cast`: implemented
- `and`: implemented
- `or`: implemented
- `xor`: implemented
- `nand`: implemented via negated `and`
- `nor`: implemented via negated `or`

### 18.4 Precedence Structure

- pipe expressions at the top: implemented
- logical or / nor: implemented
- logical xor: implemented
- logical and / nand: implemented
- comparisons and keyword comparisons: implemented
- ranges: implemented
- add/sub: implemented
- mul/div/mod: implemented
- power recursion: implemented
- primary/postfix: implemented

### 18.5 Calls And Invocation

- normal function calls: implemented
- method calls: implemented
- general invocation of arbitrary callee expressions: implemented
- call-argument parsing with separator diagnostics: implemented
- qualified path call targets: implemented
- quoted callable names: implemented

### 18.6 Access Forms

- field access: implemented
- index access: implemented
- slice access with open start: implemented
- slice access with open end: implemented
- reverse slice handling via double-colon/path-like separator: implemented
- pattern access inside brackets with comma-separated patterns: implemented
- prefix availability access `target:[...]`: implemented
- suffix availability access on index/slice/pattern nodes: implemented

### 18.7 Container And Range Forms

- `{a, b, c}` container literal: implemented
- single range in braces lowered as range instead of array: implemented
- open-start ranges: implemented
- open-end ranges: implemented
- `..` and `...` distinction: implemented

### 18.8 Rolling Expressions

- `{ expr for x in iterable }`: implemented
- multiple rolling bindings: implemented
- parenthesized rolling binding lists: implemented
- typed rolling binders: implemented
- optional trailing `if` filter: implemented
- duplicate rolling binder rejection: implemented

### 18.9 Pipes

- `lhs | rhs`: implemented
- `lhs || rhs`: implemented
- pipe stage may be:
- normal expression
- `return ...`
- builtin diagnostic statement

### 18.10 Remaining Expression Work

- deeper semantic typing
- short-circuit/runtime semantics
- more deliberate char literal handling
- lowering of pipe/rolling forms

## 19. Parser Surface: Statements And Control Flow

### 19.1 Builtin Diagnostic Statements

- `panic`: implemented
- `report`: implemented
- `check`: implemented
- `assert`: implemented
- multiple arguments: implemented
- separator diagnostics: implemented

### 19.2 `when`

- `when(expr) { ... }`: implemented
- `case(...)`: implemented
- `of(...)`: implemented
- `is(...)`: implemented
- `in(...)`: implemented
- `has(...)`: implemented
- `on(...)`: implemented
- default via bare body: implemented
- default via `*`: implemented
- default via `$`: implemented
- branch bodies via block: implemented
- branch bodies via `=> expr`: implemented

### 19.3 `if`

- `if (...) { ... }`: implemented
- `else if`: implemented
- `else`: implemented
- lowering to `When`: implemented

### 19.4 Loops

- `loop (...) { ... }`: implemented
- `for (...) { ... }`: implemented
- `each (...) { ... }`: implemented
- plain condition loop: implemented
- iteration binder loops: implemented
- typed binder declaration with semicolon: implemented
- `_` silent binder: implemented
- trailing `when` filter in iteration condition: implemented

### 19.5 Assignment Statements

- simple `=` assignment: implemented
- `+=`: implemented
- `-=`: implemented
- `*=`: implemented
- `/=`: implemented
- `%=` shape via symbol+`=` detection: implemented
- `^=` shape via symbol+`=` detection: implemented
- field assignment targets: implemented
- index assignment targets: implemented
- slice assignment targets: implemented
- method-call assignment targets rejected: implemented
- function-call assignment targets rejected: implemented

### 19.6 Block Statements

- nested blocks: implemented
- recursive body parsing: implemented
- optional semicolon consumption in relevant places: implemented

### 19.7 Return / Break / Yield

- `return` with optional value: implemented
- `break`: implemented
- `yeild` with required value: implemented

### 19.8 Remaining Statement Work

- runtime control-flow meaning
- loop scoping semantics
- definite-return analysis
- iterator/yield semantics beyond syntax

## 20. Parser-Local Validation Already Present

- illegal-token parse errors: implemented
- unary missing-operand diagnostics: implemented
- mismatched paren/bracket/brace diagnostics: implemented in many paths
- duplicate binding/routine generic/parameter detection: implemented in many paths
- duplicate capture detection: implemented
- duplicate inquiry clause detection: implemented
- duplicate standard member detection: implemented
- duplicate use-name detection: implemented
- duplicate type member detection: implemented
- conflicting binding option detection: implemented
- conflicting declaration visibility detection: implemented
- method receiver type restrictions: implemented
- unsupported declaration-family combinations: implemented
- dedicated parser-owned expected-shape diagnostics: implemented

### 20.1 Important Caveat

- These checks are structural parser responsibilities, not parser-side semantic analysis.

## 21. What The Parser Still Does Not Have

- whole-program symbol table
- import resolution across files
- real package/module linking
- standard conformance checking
- type inference beyond small local helpers
- full type checker
- ownership checker
- borrow checker
- effect system
- coroutine/eventual semantics
- backend/lowering pipeline
- optimizer
- executable output

## 22. fol-diagnostics Status

### 22.1 File

- `fol-diagnostics/src/lib.rs` (`267` lines)

### 22.2 Implemented

- `OutputFormat::Human`
- `OutputFormat::Json`
- `DiagnosticLocation`
- `Severity`
- `Diagnostic`
- `DiagnosticReport`
- error counting
- warning counting
- JSON rendering
- human-readable rendering
- location rendering
- `from_glitch` construction
- parser-error location handoff support
- basic error code extraction from message text

### 22.3 Current Limitations

- code extraction is message-string based
- help text is mostly unused
- no later-phase diagnostics because later phases do not exist yet

### 22.4 Status Call

- `fol-diagnostics`: implemented
- Scope: enough for current front-end

## 23. Root CLI Status

### 23.1 File

- `src/main.rs` (`172` lines)

### 23.2 Implemented

- accepts file or folder input
- default input fallback exists
- `--json` flag exists
- chooses stream mode based on file-vs-directory
- builds `FileStream`
- builds stage3 lexer
- builds `AstParser`
- emits diagnostics
- exits nonzero on errors

### 23.3 What Success Currently Means

- "success" currently means:
- stream creation succeeded
- lexing/parsing completed
- diagnostics report has no errors

### 23.4 What CLI Does Not Yet Do

- no semantic pass
- no type-check pass
- no IR generation
- no backend selection
- no output artifact emission
- no package build orchestration

## 24. Legacy / Orphan / Transitional Code

- `src/syntax/token/data/mod.rs` exists and defines a `TYPE` enum
- I did not find active references to it in the current workspace scan
- Status call:
- likely legacy or orphaned
- Remaining work:
- either wire it into the real front-end
- or delete/archive it cleanly

## 25. Documentation Status Relative To Code

- `README.md`: behind implementation
- `FEATURES.md`: behind implementation
- mdBook: mixed aspirational and outdated syntax
- Some current parser features appear only in tests/code, not docs
- Some docs describe semantics that are not yet enforced

### 25.1 Recommended Documentation Rule

- parser + tests should be treated as source of truth until docs are updated

## 26. Active Test Suite Overview

### 26.1 Harness Shape

- `test/run_tests.rs` includes:
- stream tests
- lexer tests
- parser tests
- integration smoke tests

### 26.2 Stream Test Files

- `test/stream/test_stream.rs`: `10` tests
- `test/stream/test_namespace.rs`: `8` tests
- `test/stream/test_mod_handling.rs`: `7` tests

### 26.3 Lexer Test File

- `test/lexer/test_lexer.rs`: `13` tests

### 26.4 Parser Test Driver

- `test/parser/test_parser.rs`: includes dozens of parser-part modules

### 26.5 Parser Test Module Counts

- `anonymous_function_expressions.rs`: `8`
- `assignments_and_logical_expressions.rs`: `23`
- `availability_access_expressions.rs`: `3`
- `basic_declarations.rs`: `26`
- `binding_alternatives.rs`: `20`
- `binding_multi.rs`: `10`
- `binding_options.rs`: `7`
- `call_and_postfix_expressions.rs`: `16`
- `comparison_keyword_expressions.rs`: `5`
- `container_and_unary_expressions.rs`: `20`
- `custom_error_report_validation.rs`: `44`
- `definition_declarations.rs`: `20`
- `flow_bodies.rs`: `3`
- `implementation_declarations.rs`: `9`
- `inquiry_clauses.rs`: `6`
- `invoke_expressions.rs`: `4`
- `keyword_named_bindings.rs`: `1`
- `keyword_named_parameters.rs`: `2`
- `keyword_named_routines.rs`: `1`
- `keyword_named_type_members.rs`: `4`
- `keyword_named_types.rs`: `1`
- `lab_declarations.rs`: `2`
- `loops_use_bindings_and_ranges.rs`: `23`
- `method_receivers_and_branching.rs`: `29`
- `named_function_types.rs`: `2`
- `named_generics.rs`: `5`
- `pattern_access_expressions.rs`: `1`
- `pipe_expressions.rs`: `10`
- `pipe_lambda_expressions.rs`: `4`
- `qualified_path_expressions.rs`: `5`
- `qualified_quoted_type_references.rs`: `5`
- `quoted_alias_names.rs`: `1`
- `quoted_bindings.rs`: `5`
- `quoted_binding_type_hints.rs`: `2`
- `quoted_call_expressions.rs`: `1`
- `quoted_declaration_targets.rs`: `2`
- `quoted_function_type_refs.rs`: `2`
- `quoted_iteration_binders.rs`: `4`
- `quoted_member_access.rs`: `1`
- `quoted_parameters.rs`: `4`
- `quoted_receiver_types.rs`: `2`
- `quoted_report_call_resolution.rs`: `40`
- `quoted_root_statements.rs`: `2`
- `quoted_routine_names.rs`: `1`
- `quoted_type_members.rs`: `5`
- `quoted_type_names.rs`: `1`
- `quoted_type_references.rs`: `1`
- `quoted_use_names.rs`: `6`
- `range_expressions.rs`: `1`
- `reference_keywords.rs`: `1`
- `report_call_resolution.rs`: `40`
- `rolling_expressions.rs`: `6`
- `routine_closure_captures.rs`: `3`
- `routine_error_types.rs`: `6`
- `routine_headers_and_when_forms.rs`: `21`
- `segment_declarations.rs`: `8`
- `single_quoted_names.rs`: `1`
- `slice_access_expressions.rs`: `3`
- `slice_assignment_targets.rs`: `2`
- `source_kind_types.rs`: `4`
- `standard_declarations.rs`: `15`
- `test_block_declarations.rs`: `4`
- `top_level_control_flow_and_calls.rs`: `21`
- `type_definition_validation.rs`: `20`
- `typed_iteration_binders.rs`: `11`
- `type_forms_and_function_decls.rs`: `19`
- `type_member_metadata.rs`: `2`
- `unary_and_call_argument_errors.rs`: `26`
- `unmatched_paren_errors.rs`: `8`
- `use_options.rs`: `6`
- `use_paths.rs`: `4`
- `variadic_parameters.rs`: `4`

### 26.6 Integration Smoke Tests In `test/run_tests.rs`

- stream-to-lexer integration: exists
- lexer-to-parser integration: exists
- full pipeline integration: exists
- parser-error location diagnostics propagation: exists

## 27. Current Main Risks

- parser and docs are diverging
- parser is carrying semantic-ish checks that should move later
- test suite is not fully green
- top-level AST flattening quirk may break later phases
- keyword spelling drift (`yeild`) is already leaking into tests and diagnostics

## 28. Highest-Value Remaining Work

- fix the two observed integration failures
- split parser-local semantic checks into a semantic pass
- build whole-program symbol collection
- build actual type checking
- decide official syntax where parser/docs disagree
- clean up lexer spelling debt
- remove legacy/orphan modules or wire them intentionally
- update public docs after syntax decisions are made

## 29. Status Summary By Area

- stream ingestion: implemented
- source namespace derivation: implemented
- stage0 raw lexing window: implemented
- stage1 token classification: implemented
- stage2 token normalization: implemented
- stage3 parser-facing token stream: implemented
- AST model: implemented
- declarations parsing: implemented broadly
- type reference parsing: implemented broadly
- expression parsing: implemented broadly
- statement parsing: implemented broadly
- diagnostics rendering: implemented
- CLI driver: implemented
- package resolver: partial/missing
- semantic analysis: missing
- type checker: missing
- backend/runtime: missing
- docs sync: missing

## 30. Appendices

- Appendix A: active source file inventory
- Appendix B: active function inventory outside parser parts
- Appendix C: parser-part function inventory
- Appendix D: active parser fixture inventory

## Appendix A: Active Source File Inventory

### A.1 Root And Shared Crates

- `Cargo.toml`: active workspace and integration-test manifest
- `src/main.rs`: active CLI driver for stream -> lexer -> parser -> diagnostics
- `fol-types/src/lib.rs`: active shared error trait and `BasicError`
- `fol-types/src/mod.rs`: active shared aliases/constants/macros
- `fol-types/src/error.rs`: active early error enums
- `fol-stream/src/lib.rs`: active source discovery and character streaming crate
- `fol-diagnostics/src/lib.rs`: active diagnostics rendering crate

### A.2 Active Lexer Files

- `fol-lexer/src/lib.rs`: active lexer traits
- `fol-lexer/src/point.rs`: active token/source location model
- `fol-lexer/src/token/mod.rs`: active token umbrella enum and helpers
- `fol-lexer/src/token/buildin/mod.rs`: active builtin keyword token inventory
- `fol-lexer/src/token/literal/mod.rs`: active literal token inventory
- `fol-lexer/src/token/operator/mod.rs`: active operator token inventory
- `fol-lexer/src/token/symbol/mod.rs`: active symbol token inventory
- `fol-lexer/src/token/void/mod.rs`: active void token inventory
- `fol-lexer/src/token/help.rs`: active raw-char helper predicates
- `fol-lexer/src/lexer/mod.rs`: active stage module export
- `fol-lexer/src/lexer/stage0/mod.rs`: active stage0 export
- `fol-lexer/src/lexer/stage0/elements.rs`: active stage0 implementation
- `fol-lexer/src/lexer/stage1/mod.rs`: active stage1 export
- `fol-lexer/src/lexer/stage1/element.rs`: active stage1 token builder
- `fol-lexer/src/lexer/stage1/elements.rs`: active stage1 stream/window
- `fol-lexer/src/lexer/stage2/mod.rs`: active stage2 export
- `fol-lexer/src/lexer/stage2/element.rs`: active stage2 token normalizer
- `fol-lexer/src/lexer/stage2/elements.rs`: active stage2 stream/window
- `fol-lexer/src/lexer/stage3/mod.rs`: active stage3 export
- `fol-lexer/src/lexer/stage3/element.rs`: active stage3 number normalizer
- `fol-lexer/src/lexer/stage3/elements.rs`: active parser-facing token stream

### A.3 Active Parser Files

- `fol-parser/src/lib.rs`: active parser crate export
- `fol-parser/src/ast/mod.rs`: active AST definition
- `fol-parser/src/ast/parser.rs`: active parser shell and module wiring
- `fol-parser/src/ast/parser_parts/access_expression_parsers.rs`: active postfix access parsing
- `fol-parser/src/ast/parser_parts/binding_alternative_parsers.rs`: active binding shorthand parsing
- `fol-parser/src/ast/parser_parts/binding_option_parsers.rs`: active binding option parsing
- `fol-parser/src/ast/parser_parts/binding_value_parsers.rs`: active binding value parsing
- `fol-parser/src/ast/parser_parts/declaration_option_parsers.rs`: active decl visibility parsing
- `fol-parser/src/ast/parser_parts/declaration_parsers.rs`: active def/ali/typ/fun/log/pro parsing
- `fol-parser/src/ast/parser_parts/expression_atoms_and_report_validation.rs`: active atom parsing and literal helpers
- `fol-parser/src/ast/parser_parts/expression_parsers.rs`: active precedence parser
- `fol-parser/src/ast/parser_parts/grouped_binding_parsers.rs`: active grouped binding parsing
- `fol-parser/src/ast/parser_parts/implementation_declaration_parsers.rs`: active `imp` parsing
- `fol-parser/src/ast/parser_parts/inquiry_clause_parsers.rs`: active inquiry clause parsing
- `fol-parser/src/ast/parser_parts/pipe_expression_parsers.rs`: active pipe parsing
- `fol-parser/src/ast/parser_parts/pipe_lambda_parsers.rs`: active lambda parsing
- `fol-parser/src/ast/parser_parts/postfix_expression_parsers.rs`: active postfix chaining
- `fol-parser/src/ast/parser_parts/primary_expression_parsers.rs`: active primary expression parsing
- `fol-parser/src/ast/parser_parts/program_and_bindings.rs`: active parse entry and binding declarations
- `fol-parser/src/ast/parser_parts/rolling_expression_parsers.rs`: active rolling expression parsing
- `fol-parser/src/ast/parser_parts/routine_capture_parsers.rs`: active capture list parsing
- `fol-parser/src/ast/parser_parts/routine_headers_and_type_lowering.rs`: active header parsing and type lowering
- `fol-parser/src/ast/parser_parts/segment_declaration_parsers.rs`: active `seg` parsing
- `fol-parser/src/ast/parser_parts/source_kind_type_parsers.rs`: active `url/loc/std` type parsing
- `fol-parser/src/ast/parser_parts/standard_declaration_parsers.rs`: active `std` parsing
- `fol-parser/src/ast/parser_parts/statement_parsers.rs`: active statement/control-flow parsing
- `fol-parser/src/ast/parser_parts/test_type_parsers.rs`: active `tst[...]` argument parsing
- `fol-parser/src/ast/parser_parts/type_definition_parsers.rs`: active record/entry body parsing
- `fol-parser/src/ast/parser_parts/type_references_and_blocks.rs`: active type refs and block bodies
- `fol-parser/src/ast/parser_parts/use_declaration_parsers.rs`: active `use` parsing
- `fol-parser/src/ast/parser_parts/use_option_parsers.rs`: active `use` options parsing

### A.4 Active Test Files

- `test/run_tests.rs`: active integration test harness
- `test/lexer/test_lexer.rs`: active lexer suite
- `test/stream/test_stream.rs`: active stream suite
- `test/stream/test_namespace.rs`: active namespace suite
- `test/stream/test_mod_handling.rs`: active `.mod` traversal suite
- `test/parser/test_parser.rs`: active parser suite loader
- `test/parser/test_parser_parts/*.rs`: active parser behavior suites
- `test/parser/simple_*.fol`: active parser fixtures

### A.5 Legacy / Suspicious / Orphan

- `src/syntax/token/data/mod.rs`: present, not referenced in current scan
- `test_old/`: present, archival, not the active test target

## Appendix B: Active Function Inventory Outside Parser Parts

### B.1 Root CLI

- `main`: implemented, root CLI entry
- `compile_file`: implemented, file/folder parse driver
- `report_input_error`: implemented, input-path diagnostic bridge
- `parser_error_location`: implemented, parse-error location extraction
- `compile_missing_file_reports_error`: implemented, CLI regression test

### B.2 fol-diagnostics

- `DiagnosticReport::new`: implemented
- `DiagnosticReport::add_diagnostic`: implemented
- `DiagnosticReport::add_error`: implemented
- `DiagnosticReport::has_errors`: implemented
- `DiagnosticReport::output`: implemented
- `DiagnosticReport::to_json`: implemented
- `DiagnosticReport::to_human_readable`: implemented
- `Default for DiagnosticReport::default`: implemented
- `Diagnostic::from_glitch`: implemented
- `Diagnostic::to_human_readable`: implemented
- `extract_error_code`: implemented
- `DiagnosticLocation::from_point_location`: implemented
- `PointLocationLike::get_file_path`: trait method present
- `PointLocationLike::get_row`: trait method present
- `PointLocationLike::get_col`: trait method present
- `PointLocationLike::get_len`: trait method present
- `test_diagnostic_report_json`: test present
- `test_diagnostic_report_human`: test present

### B.3 fol-stream

- `CharacterProvider::next_char`: trait method present
- `StreamSource::into_provider`: trait method present
- `Source::init`: implemented
- `Source::init_with_package`: implemented
- `Source::path`: implemented
- `Source::rel_path`: implemented
- `Source::abs_path`: implemented
- `Source::module`: implemented
- `FileStream::from_file`: implemented
- `FileStream::from_folder`: implemented
- `FileStream::from_sources`: implemented
- `FileStream::current_source`: implemented
- `FileStream::sources`: implemented
- `FileStream as CharacterProvider::next_char`: implemented
- `source`: implemented
- `from_dir`: implemented
- `check_validity`: implemented
- `sources`: implemented
- `detect_package_name`: implemented
- `compute_namespace`: implemented
- `is_valid_namespace_component`: implemented

### B.4 fol-lexer Point And Token Helpers

- `Source::new`: implemented
- `Source::path`: implemented
- `Location::visualize`: implemented
- `Location::print`: implemented
- `Location::set_source`: implemented
- `Location::from_stream_location`: implemented
- `Location::source`: implemented
- `Location::row`: implemented
- `Location::col`: implemented
- `Location::len`: implemented
- `Location::is_empty`: implemented
- `Location::set_len`: implemented
- `Location::longer`: implemented
- `Location::deep`: implemented
- `Location::set_deep`: implemented
- `Location::new_char`: implemented
- `Location::new_line`: implemented
- `Location::new_word`: implemented
- `Location::adjust`: implemented
- `KEYWORD::is_assign`: implemented
- `KEYWORD::is_ident`: implemented
- `KEYWORD::is_literal`: implemented
- `KEYWORD::is_buildin`: implemented
- `KEYWORD::is_illegal`: implemented
- `KEYWORD::is_comment`: implemented
- `KEYWORD::is_open_bracket`: implemented
- `KEYWORD::is_close_bracket`: implemented
- `KEYWORD::is_bracket`: implemented
- `KEYWORD::is_decimal`: implemented
- `KEYWORD::is_number`: implemented
- `KEYWORD::is_numberish`: implemented
- `KEYWORD::is_symbol`: implemented
- `KEYWORD::is_operator`: implemented
- `KEYWORD::is_void`: implemented
- `KEYWORD::is_eof`: implemented
- `KEYWORD::is_space`: implemented
- `KEYWORD::is_eol`: implemented
- `KEYWORD::is_nonterm`: implemented
- `KEYWORD::is_terminal`: implemented
- `KEYWORD::is_dot`: implemented
- `KEYWORD::is_comma`: implemented
- `KEYWORD::is_continue`: implemented
- `get_keyword`: implemented, minimal helper only
- `is_eof`: implemented
- `is_eol`: implemented
- `is_space`: implemented
- `is_digit`: implemented
- `is_alpha`: implemented
- `is_bracket`: implemented
- `is_symbol`: implemented
- `is_oct_digit`: implemented
- `is_hex_digit`: implemented
- `is_alphanumeric`: implemented
- `is_void`: implemented
- `is_open_bracket`: implemented
- `is_close_bracket`: implemented

### B.5 fol-lexer Stage 0

- `Elements::curr`: implemented
- `Elements::next_vec`: implemented
- `Elements::peek`: implemented
- `Elements::prev_vec`: implemented
- `Elements::seek`: implemented
- `Elements::init`: implemented
- `Elements::bump`: implemented
- `Elements::debug`: implemented
- `Iterator for Elements::next`: implemented
- `Display for Elements::fmt`: implemented
- `gen`: implemented

### B.6 fol-lexer Stage 1

- `Element::init`: implemented
- `Element::key`: implemented
- `Element::set_key`: implemented
- `Element::loc`: implemented
- `Element::set_loc`: implemented
- `Element::con`: implemented
- `Element::set_con`: implemented
- `Element::append`: implemented
- `Element::analyze`: implemented
- `Element::comment`: implemented
- `Element::endfile`: implemented
- `Element::endline`: implemented
- `Element::space`: implemented
- `Element::digit`: implemented
- `Element::encap`: implemented
- `Element::symbol`: implemented
- `Element::alpha`: implemented
- `Element::push`: implemented
- `Element::bump`: implemented
- `Elements::init`: implemented
- `Elements::curr`: implemented
- `Elements::next_vec`: implemented
- `Elements::peek`: implemented
- `Elements::prev_vec`: implemented
- `Elements::seek`: implemented
- `Elements::bump`: implemented
- `Elements::debug`: implemented
- `Elements::echo`: implemented
- `Iterator for Elements::next`: implemented
- `elements`: implemented

### B.7 fol-lexer Stage 2

- `Element::init`: implemented
- `Element::key`: implemented
- `Element::set_key`: implemented
- `Element::loc`: implemented
- `Element::set_loc`: implemented
- `Element::con`: implemented
- `Element::set_con`: implemented
- `Element::append`: implemented
- `Element::analyze`: implemented
- `Element::make_multi_operator`: implemented
- `Element::make_comment`: implemented
- `Element::bump`: implemented
- `Elements::init`: implemented
- `Elements::curr`: implemented
- `Elements::next_vec`: implemented
- `Elements::peek`: implemented
- `Elements::prev_vec`: implemented
- `Elements::seek`: implemented
- `Elements::bump`: implemented
- `Elements::jump`: implemented
- `Elements::eat`: implemented
- `Elements::until_term`: implemented
- `Elements::debug`: implemented
- `Elements::window`: implemented
- `Iterator for Elements::next`: implemented
- `elements`: implemented

### B.8 fol-lexer Stage 3

- `Element::init`: implemented
- `Element::key`: implemented
- `Element::set_key`: implemented
- `Element::loc`: implemented
- `Element::set_loc`: implemented
- `Element::con`: implemented
- `Element::set_con`: implemented
- `Element::append`: implemented
- `Element::analyze`: implemented
- `Element::make_number`: implemented
- `Element::bump`: implemented
- `Elements::default`: implemented
- `Elements::init`: implemented
- `Elements::set_key`: implemented
- `Elements::curr`: implemented
- `Elements::next_vec`: implemented
- `Elements::peek`: implemented
- `Elements::prev_vec`: implemented
- `Elements::seek`: implemented
- `Elements::bump`: implemented
- `Elements::jump`: implemented
- `Elements::eat`: implemented
- `Elements::until_term`: implemented
- `Elements::debug`: implemented
- `Elements::window`: implemented
- `Iterator for Elements::next`: implemented
- `elements`: implemented

## Appendix C: Parser-Part Function Inventory

### C.1 program_and_bindings.rs

- `new`: implemented
- `parse`: implemented
- `parse_lexer_literal`: implemented
- `parse_var_decl`: implemented
- `parse_let_decl`: implemented
- `parse_con_decl`: implemented
- `parse_lab_decl`: implemented
- `parse_binding_decl`: implemented
- `parse_binding_names`: implemented
- `build_binding_nodes`: implemented

### C.2 binding_alternative_parsers.rs

- `lookahead_binding_alternative`: implemented
- `parse_binding_alternative_decl`: implemented
- `next_significant_token_from_window`: implemented

### C.3 binding_option_parsers.rs

- `parse_binding_options`: implemented
- `merge_binding_options`: implemented
- `binding_options_conflict`: implemented
- `binding_option_label`: implemented

### C.4 binding_value_parsers.rs

- `parse_binding_values`: implemented
- `lookahead_closes_binding_values`: implemented
- `lookahead_starts_binding_segment`: implemented

### C.5 grouped_binding_parsers.rs

- `parse_binding_group`: implemented

### C.6 declaration_option_parsers.rs

- `validate_decl_visibility_options`: implemented
- `parse_decl_visibility_options`: implemented

### C.7 use_option_parsers.rs

- `parse_use_options`: implemented

### C.8 use_declaration_parsers.rs

- `parse_use_decl`: implemented
- `parse_use_paths`: implemented
- `build_use_nodes`: implemented
- `parse_use_names`: implemented
- `parse_direct_use_path`: implemented

### C.9 source_kind_type_parsers.rs

- `try_parse_source_kind_type_suffix`: implemented
- `lower_bare_source_kind_type_name`: implemented

### C.10 segment_declaration_parsers.rs

- `parse_seg_decl`: implemented

### C.11 implementation_declaration_parsers.rs

- `parse_imp_decl`: implemented

### C.12 standard_declaration_parsers.rs

- `lookahead_is_std_decl`: implemented
- `parse_std_decl`: implemented
- `parse_standard_protocol_body`: implemented
- `parse_standard_blueprint_body`: implemented
- `parse_standard_extended_body`: implemented
- `parse_standard_routine_signature`: implemented
- `standard_member_key`: implemented
- `parse_empty_standard_kind_options`: implemented

### C.13 declaration_parsers.rs

- `parse_def_decl`: implemented
- `parse_alias_decl`: implemented
- `parse_type_decl`: implemented
- `parse_empty_type_marker_brackets`: implemented
- `parse_type_generic_header`: implemented
- `parse_type_options`: implemented
- `parse_use_path`: implemented
- `parse_fun_decl`: implemented
- `parse_log_decl`: implemented
- `parse_pro_decl`: implemented
- `parse_routine_generics_and_params`: implemented

### C.14 type_definition_parsers.rs

- `parse_entry_type_definition`: implemented
- `parse_record_type_definition`: implemented

### C.15 routine_headers_and_type_lowering.rs

- `parse_routine_header_list`: implemented
- `parameters_to_generics`: implemented
- `ensure_unique_parameter_names`: implemented
- `parse_generic_list`: implemented
- `parse_routine_options`: implemented
- `parse_parameter_list`: implemented
- `parse_routine_name_with_optional_receiver`: implemented
- `register_routine_return_type`: implemented
- `register_routine_return_type_key`: implemented
- `callable_key`: implemented
- `reported_callable_arity_mismatch_message`: implemented
- `parse_callable_key`: implemented
- `fol_type_label`: implemented
- `lower_bare_scalar_type_name`: implemented
- `lower_integer_option`: implemented
- `lower_float_option`: implemented
- `lower_char_option`: implemented
- `parse_type_reference_tokens`: implemented

### C.16 routine_capture_parsers.rs

- `ensure_unique_capture_names`: implemented
- `parse_optional_routine_capture_list`: implemented

### C.17 inquiry_clause_parsers.rs

- `parse_routine_body_with_inquiries`: implemented
- `parse_optional_inquiry_clause`: implemented
- `parse_inquiry_body`: implemented

### C.18 test_type_parsers.rs

- `parse_test_type_arguments`: implemented

### C.19 type_references_and_blocks.rs

- `parse_function_type_reference`: implemented
- `try_parse_special_type_suffix`: implemented
- `parse_integer_type_reference`: implemented
- `parse_float_type_reference`: implemented
- `parse_char_type_reference`: implemented
- `parse_scalar_type_options`: implemented
- `parse_type_argument_list`: implemented
- `parse_array_type_arguments`: implemented
- `parse_matrix_type_arguments`: implemented
- `parse_balanced_type_suffix`: implemented
- `parse_block_body`: implemented
- `parse_block_stmt`: implemented
- `parse_return_stmt`: implemented
- `parse_break_stmt`: implemented
- `parse_yield_stmt`: implemented

### C.20 expression_atoms_and_report_validation.rs

- `parse_primary`: implemented
- `skip_ignorable`: implemented
- `token_can_be_logical_name`: implemented
- `token_to_named_label`: implemented
- `unary_prefix_info`: implemented
- `ensure_unary_operand`: implemented
- `validate_report_usage`: implemented
- `report_literal_type_mismatch`: implemented
- `report_identifier_type_mismatch`: implemented
- `report_unknown_identifier_in_expression`: implemented
- `report_expression_type_mismatch`: implemented
- `literal_matches_named_type`: implemented
- `is_builtin_scalar_type_name`: implemented
- `parameter_type_map`: implemented
- `infer_named_type_from_node`: implemented
- `fol_type_to_named_family`: implemented
- `type_family_name`: implemented
- `named_types_compatible`: implemented
- `is_numeric_named_type`: implemented
- `non_scalar_report_expression_label`: implemented
- `consume_optional_semicolon`: implemented
- `parse_literal`: implemented

### C.21 expression_parsers.rs

- `parse_call_args`: implemented
- `lookahead_is_assignment`: implemented
- `lookahead_is_call`: implemented
- `lookahead_is_method_call`: implemented
- `lookahead_is_general_invoke`: implemented
- `can_start_assignment`: implemented
- `previous_significant_key`: implemented
- `bump_if_no_progress`: implemented
- `token_is_word`: implemented
- `compound_assignment_op`: implemented
- `compound_assignment_symbol_op`: implemented
- `parse_logical_expression`: implemented
- `parse_logical_or_expression`: implemented
- `parse_logical_xor_expression`: implemented
- `parse_logical_and_expression`: implemented
- `parse_comparison_expression`: implemented
- `parse_range_expression`: implemented
- `next_significant_key_from_window`: implemented
- `consume_significant_token`: implemented
- `parse_add_sub_expression`: implemented
- `parse_mul_div_expression`: implemented
- `parse_pow_expression`: implemented

### C.22 primary_expression_parsers.rs

- `lookahead_is_shorthand_anonymous_fun`: implemented
- `parse_primary_expression`: implemented
- `parse_container_expression`: implemented
- `parse_anonymous_fun_expr`: implemented
- `parse_anonymous_pro_expr`: implemented
- `parse_anonymous_log_expr`: implemented
- `parse_anonymous_routine_after_keyword`: implemented
- `parse_shorthand_anonymous_fun_expr`: implemented

### C.23 postfix_expression_parsers.rs

- `parse_postfix_expression`: implemented

### C.24 access_expression_parsers.rs

- `token_can_start_path_expression`: implemented
- `consume_slice_separator`: implemented
- `parse_optional_slice_end`: implemented
- `parse_named_path`: implemented
- `parse_index_or_slice_expression`: implemented
- `parse_index_or_slice_assignment_target`: implemented
- `parse_prefix_availability_expression`: implemented

### C.25 pipe_expression_parsers.rs

- `parse_pipe_stage_expression`: implemented
- `parse_pipe_expression`: implemented

### C.26 pipe_lambda_parsers.rs

- `parse_pipe_lambda_expr`: implemented

### C.27 rolling_expression_parsers.rs

- `parse_rolling_expression`: implemented
- `parse_rolling_bindings`: implemented
- `parse_rolling_binding`: implemented

### C.28 statement_parsers.rs

- `parse_builtin_call_stmt`: implemented
- `parse_when_stmt`: implemented
- `parse_if_stmt`: implemented
- `parse_branch_body`: implemented
- `parse_loop_stmt`: implemented
- `parse_loop_condition`: implemented
- `parse_assignment_stmt`: implemented
- `parse_assignment_target`: implemented
- `parse_call_stmt`: implemented
- `parse_invoke_stmt`: implemented
- `parse_call_expr`: implemented
- `parse_method_call_expr`: implemented
- `parse_open_paren_and_call_args`: implemented

## Appendix D: Active Parser Fixture Inventory

### D.1 Fixture Count By Prefix Family

- `ali`: `3` fixtures covering alias declarations and alias naming variants
- `at`: `2` fixtures covering `@` binding shorthand and borrowing/new-related shorthand surfaces
- `bang`: `1` fixture covering `!` binding shorthand
- `binding`: `7` fixtures covering binding options and keyword-name variants
- `call`: `18` fixtures covering top-level call syntax and call error recovery
- `con`: `1` fixture covering constant declarations
- `def`: `22` fixtures covering block/module/test definitions and naming/options
- `each`: `8` fixtures covering `each` loop binders and silent/typed variants
- `for`: `6` fixtures covering `for` loop binders and mismatch diagnostics
- `fun`: `315` fixtures covering the largest parser surface: functions, expressions, calls, methods, control flow, pipes, rolling, error types, quoting, and assignment targets
- `function`: `2` fixtures covering function-type declarations/bindings
- `imp`: `9` fixtures covering implementation declarations and options
- `keyword`: `2` fixtures covering keyword-named declarations
- `lab`: `1` fixture covering label declarations
- `let`: `1` fixture covering `let` declarations
- `log`: `3` fixtures covering logical declarations and anonymous logical forms
- `loop`: `6` fixtures covering loop binders and loop top-level cases
- `minus`: `2` fixtures covering `-` binding shorthand and constant shorthand combinations
- `named`: `2` fixtures covering named generic/capture surfaces
- `pipe`: `4` fixtures covering pipe lambdas
- `plus`: `5` fixtures covering `+` binding shorthand variants
- `pro`: `67` fixtures covering procedures, inquiry clauses, receivers, and custom error reporting
- `qualified`: `1` fixture covering qualified quoted type references
- `query`: `1` fixture covering query-var parsing surface
- `quoted`: `9` fixtures covering quoted declaration/parameter/type/use/reference surfaces
- `root`: `1` fixture covering root quoted type refs
- `routine`: `12` fixtures covering routine options, grouped params, generics, and defaults
- `seg`: `7` fixtures covering segment declarations and visibility/name/type cases
- `single`: `13` fixtures covering single-quoted names and type/member/reference surfaces
- `source`: `3` fixtures covering `url/loc/std` source-kind types
- `std`: `14` fixtures covering standard declarations, kinds, options, and duplicate members
- `tilde`: `2` fixtures covering `~` binding shorthand
- `top`: `2` fixtures covering top-level quoted/keyword call and assignment cases
- `typ`: `55` fixtures covering type declarations, entry/record forms, options, special/container types, and generic variants
- `use`: `24` fixtures covering import declarations, options, quoting, direct paths, and multi-import forms
- `var`: `14` fixtures covering variable declarations, grouped/multi bindings, quoted names, and inferred/qualified types

### D.2 What The Fixture Ledger Says

- The parser test surface is very broad.
- The implementation focus is heavily concentrated in `fun` and `pro` parsing, which matches the amount of code in the parser modules.
- The fixture families confirm real support for:
- declarations
- receiver methods
- generics
- defaults
- variadics
- quoted names
- keyword-named items
- control flow
- ranges
- pipes
- rolling expressions
- availability/index/slice access

### D.3 What The Fixture Ledger Does Not Prove By Itself

- full semantic correctness across files
- runtime behavior
- conformance/type-checking completeness
- documentation accuracy

### D.4 Remaining Follow-Up For Fixtures

- fix the two currently observed failing tests
- audit whether any stale fixture names still describe syntax that has changed
- add a generated fixture index later if you want this appendix to list every file individually
