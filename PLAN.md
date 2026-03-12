# FOL Source Layout And Package Scope Alignment Plan

Last rebuilt: 2026-03-12
Supersedes: immediate front-end follow-up work around source layout, package scope, and parser output shape
Next phase plan: `PLAN_NEXT.md` keeps the postponed `fol-resolver` plan

## 0. Purpose

- This plan exists because the book’s source-layout model is stricter and more structured than the current parser output.
- The immediate goal is not type checking or name resolution yet.
- The immediate goal is to make the front end preserve the source structure that later semantic phases need:
- package scope
- namespace scope
- file/source-unit boundaries
- successful syntax origins
- declaration-only file roots

This is a stream/lexer/parser plan, not a resolver plan.

## 1. Book Contract We Need To Follow

The book’s current source-layout model says:

- each `.fol` file inside a folder belongs to one package
- files in the same package share the same package scope
- declarations are order-independent across those files
- subfolders define nested namespaces
- direct imports and `::` access work against package/namespace layout
- some visibility is package-wide while `hid` is file-only

The important consequence is:

- files are not isolated little programs
- but they are also not disposable text fragments inside one anonymous flattened parse root

They are distinct source units inside one package and possibly one namespace.

## 2. Why The Current Front End Is Not Enough

### 2.1 What Is Already Good

- `fol-stream` already knows package and namespace per source file.
- `fol-stream` already preserves canonical source identity.
- the lexer already emits hard file boundaries.
- the parser already prevents declarations from continuing across file boundaries.

### 2.2 What Is Missing Or Wrong For The Book Model

- successful parser output is still one flattened `AstNode::Program { declarations }`
- the root surface is intentionally mixed and script-like today
- successful AST nodes do not retain source origins
- parser output does not expose source units/files as first-class parsed artifacts
- parser output does not expose package/namespace grouping directly
- file-private visibility such as `hid` cannot be enforced later unless file identity survives parsing
- later resolver work would be forced to reconstruct source structure that the front end already knew earlier

## 3. Scope Of This Plan

This plan covers:

- `fol-stream`
- `fol-lexer`
- `fol-parser`
- root CLI integration if parser API changes require it
- front-end contract/docs sync
- test harness and fixtures

This plan does not cover:

- `fol-resolver` implementation
- import target resolution
- symbol tables
- type checking
- ownership or borrowing analysis
- runtime or backend work

Those remain in `PLAN_NEXT.md`.

## 4. Target End State

By the end of this plan, the front end should model source layout like this:

- one compilation entry can produce one parsed package
- one parsed package contains multiple parsed source units
- each parsed source unit corresponds to one physical `.fol` file
- each parsed source unit records:
- canonical file path
- package
- namespace
- ordered parsed top-level items for that file
- successful syntax nodes and important references have origin metadata
- file roots are declaration-oriented rather than script-like
- same-package files remain connected semantically later, but remain distinct syntactically now

## 5. Design Rules

### 5.1 Keep `AstNode` Syntax-Oriented

- `AstNode` should continue to represent syntax nodes, not semantic symbols.
- Package/file wrappers should live outside the core AST node enum.
- Do not cram package scope directly into every node variant.

### 5.2 Preserve Real Source Units

- A file must remain a first-class parsed unit after parsing succeeds.
- Flattening all files into one root list loses information we need later.

### 5.3 Package Scope Is Shared, File Scope Still Exists

- Same-package files share package visibility and later shared name resolution.
- But file-private visibility such as `hid` still needs one concrete file boundary.
- So the parser must preserve both:
- shared package membership
- distinct source-unit identity

### 5.4 The Lexer Does Not Become The Resolver

- The lexer should continue to provide source boundaries and locations.
- It should not grow semantic package resolution logic.
- If lexer changes happen, they should be only to expose source-boundary truth more cleanly.

### 5.5 Parse Success Must Be Locatable

- Parse errors already know file/line/column.
- Successful parse nodes must gain equivalent origin support or later diagnostics will be weak.

## 6. Proposed Parser Output Model

### 6.1 New Wrapper Types

Introduce parser-owned wrapper types outside `AstNode`:

- `ParsedPackage`
- `ParsedSourceUnit`
- `SyntaxOrigin`
- `SyntaxIndex`
- `SyntaxNodeId`

Expected shape:

- `ParsedPackage`
- package name
- source units
- syntax index

- `ParsedSourceUnit`
- canonical source identity
- file path
- package
- namespace
- ordered top-level items for that file

### 6.2 What Stays In `AstNode`

`AstNode` should continue to contain:

- declarations
- expressions
- statements
- comments/comment wrappers
- type forms

`AstNode` should not become:

- package wrapper
- namespace wrapper
- file wrapper
- semantic symbol node

### 6.3 Top-Level File Surface

The book-facing file root should be declaration-oriented.

That means the parser should stop treating file scope as a free-form script surface.

At file root, accepted top-level items should be:

- `use`
- `def`
- `seg`
- `imp`
- `var`
- `lab` if the language really intends it at file scope
- `fun`
- `pro`
- `log`
- `typ`
- `ali`
- `std`
- retained standalone comments

At file root, the parser should reject:

- free executable calls
- free assignments
- top-level `when`
- top-level `loop`
- top-level `return`
- top-level `break`
- top-level `yield`
- free literal-expression roots

If a top-level form is intentionally meant to stay valid at file scope, that must be stated explicitly and backed by book text. Otherwise the book wins.

### 6.4 Order Preservation

The parser should preserve source order:

- within each file
- across source units as discovered by `fol-stream`

But it should preserve it as:

- ordered items per source unit
- ordered source units per parsed package

not as one anonymous flattened list.

## 7. Syntax-Origin Support

This is a hard requirement, not a nice-to-have.

### 7.1 Needed Metadata

For every resolver-relevant successful syntax surface, we need:

- file
- line
- column
- length

### 7.2 Minimum Coverage

At minimum, origin support should exist for:

- top-level declarations
- nested declarations
- plain identifier references
- qualified references
- `use` declarations
- type-name references
- inquiry targets
- comments if retained as first-class top-level items

### 7.3 API Direction

Keep `AstParser::parse()` only as a temporary compatibility path if needed.

Add a package-aware parser entry point such as:

- `parse_package(...)`
- or `parse_indexed(...)`

The important part is the result shape, not the exact function name.

The new parser result must expose:

- parsed package/source units
- syntax index/origins

## 8. Stream And Lexer Expectations

### 8.1 `fol-stream`

`fol-stream` is already the source of truth for:

- canonical path
- package
- namespace
- deterministic source order

This plan should not reinvent that logic in the parser.

What parser/package wrappers should consume from stream identity:

- canonical file path
- package
- namespace
- source order

### 8.2 `fol-lexer`

The lexer already provides hard boundary tokens and per-token locations.

Likely required changes:

- none or very little in behavior
- maybe cleaner read-only access to source identity if the parser cannot already consume it cleanly
- stronger tests proving boundary/source metadata is preserved through parser grouping

This is a parser-shape plan, not a lexer redesign plan.

## 9. Visibility Preparation

This plan does not enforce `exp`/`hid` yet, but it must preserve what later phases need.

### 9.1 `exp`

- export/public visibility remains attached to declaration nodes as today
- parsed package/source-unit structure must preserve enough context to know what is exported from which package/namespace/file

### 9.2 `hid`

- the book treats `hid` as file-only visibility
- later enforcement is resolver work
- but parser output must preserve file/source-unit identity so later enforcement is even possible

Without real source units in parser output, `hid` cannot be modeled correctly later.

## 10. What This Plan Intentionally Defers

These items are real but intentionally deferred:

- cross-file declaration order independence as a semantic rule
- actual import target lookup
- export filtering during imports
- unresolved-name diagnostics
- namespace member resolution
- file-private visibility enforcement

These are resolver responsibilities after the parser shape is fixed.

## 11. Book Drift Tracked But Not Primary In This Plan

The following are real discrepancies or unsettled areas, but not the main target of this plan:

- nested procedure access/capture semantics versus the book
- stale book examples around some import spellings
- slash-comment compatibility remaining in the front end

Those should not block package/file/source-unit alignment.

## 12. Test Strategy

This plan needs a large test matrix, not a couple of smoke tests.

### 12.1 New Parser Test Areas

Add parser tests specifically for source layout and package scope:

- `test/parser/test_parser_parts/package_source_units.rs`
- `test/parser/test_parser_parts/package_root_contract.rs`
- `test/parser/test_parser_parts/source_origins.rs`
- `test/parser/test_parser_parts/namespace_layout.rs`
- `test/parser/test_parser_parts/file_visibility_prep.rs`

Exact file names can change, but these categories must exist.

### 12.2 Fixture Strategy

Use both:

- permanent folder fixtures under `test/parser/...`
- temp folder fixtures created inside tests when many layout permutations are needed

Because package/namespace behavior is folder-sensitive, single flat `.fol` files are not enough.

### 12.3 Required Positive Tests

- single-file parse produces one parsed package with one source unit
- folder parse produces one parsed package with multiple source units
- same-folder files share the same package label
- nested subfolder files get the expected namespace labels
- source-unit order follows stream discovery order
- comments survive at file root where allowed
- top-level declarations remain attached to the correct source unit
- syntax origins point to the correct file/line/column for successful top-level declarations
- syntax origins point to the correct file/line/column for nested declarations/references in covered surfaces

### 12.4 Required Negative Tests

- top-level executable call rejected at file scope
- top-level assignment rejected at file scope
- top-level control-flow statement rejected at file scope
- root literal-expression file rejected if the book does not permit it
- cross-file declaration continuation still rejected
- source-unit grouping never merges nodes from different files
- malformed folder layouts still report precise file origins

### 12.5 Required Transition Tests

- legacy parse path either remains explicitly compatibility-only or is retired cleanly
- CLI still compiles a single file and a folder without losing source metadata
- diagnostics still print parse-error locations exactly
- book-aligned package/file layout survives stream -> lexer -> parser end to end

## 13. Slice Plan

Each slice should land with tests in the same commit and be gated by:

- `make build`
- `make test`

Progress snapshot as of 2026-03-12:

- completed: `0.1`, `0.2`, `1.1`, `1.2`, `1.3`, `2.1`, `2.2`, `2.3`, `2.4`, `3.1`, `3.2`, `3.3`, `3.4`
- in progress overall: declaration-only file roots are now book-aligned, with standalone comments preserved as ordinary source-unit items
- still open: compatibility migration, visibility prep, hardening, and docs handoff

### Phase 0: Planning And Contract Setup

#### Slice 0.1

- Preserve the current resolver plan as `PLAN_NEXT.md`
- Replace `PLAN.md` with this package-scope alignment plan
- No code changes
- Status: complete on 2026-03-12

#### Slice 0.2

- Update front-end docs to say this source-layout alignment is the immediate pre-resolver phase
- Do not yet change runtime behavior
- Status: complete on 2026-03-12

### Phase 1: Successful Parse Origins

#### Slice 1.1

- Add parser-owned origin structs:
- `SyntaxOrigin`
- `SyntaxNodeId`
- `SyntaxIndex`
- Add unit tests for origin containers and ID stability
- Status: complete on 2026-03-12

#### Slice 1.2

- Add parser support for storing origins of successful top-level nodes
- Keep existing parse behavior otherwise
- Add tests that successful declarations retain exact file/line/column origins
- Status: complete on 2026-03-12

#### Slice 1.3

- Extend origin coverage to nested declaration/reference surfaces needed later
- Add tests for nested routine declarations, `use`, and representative references
- Status: complete on 2026-03-12

### Phase 2: Source Units As First-Class Parser Output

#### Slice 2.1

- Add `ParsedSourceUnit`
- Group parsed output by physical source file instead of only one flattened list
- Keep old parse path temporarily if needed
- Add tests for multi-file folder grouping
- Status: complete on 2026-03-12

#### Slice 2.2

- Add `ParsedPackage`
- Carry package and namespace information from stream identity into parser output
- Add tests for package/namespace labeling
- Status: complete on 2026-03-12

#### Slice 2.3

- Ensure ordered source units follow deterministic stream order
- Add tests for folder traversal order surviving parser output
- Status: complete on 2026-03-12

#### Slice 2.4

- Ensure each top-level node stays attached to the correct source unit
- Add multi-file tests that prove no file mixing happens
- Status: complete on 2026-03-12

### Phase 3: Book-Aligned File Root Contract

#### Slice 3.1

- Add a dedicated parser path for declaration-only file roots
- Define the accepted top-level declaration families explicitly
- Add positive tests for all accepted file-root declaration families
- Status: complete on 2026-03-12

#### Slice 3.2

- Reject top-level executable calls and assignments
- Add negative tests for those forms
- Status: complete on 2026-03-12

#### Slice 3.3

- Reject top-level control-flow and free literal-expression roots where the book does not support them
- Add negative tests for `when`, `loop`, and representative literal roots
- Status: complete on 2026-03-12

#### Slice 3.4

- Decide how retained standalone comments live at file root:
- allowed as source-unit items
- or quarantined as file-root trivia
- Implement the chosen model consistently
- Add tests for root comments across multiple files
- Status: complete on 2026-03-12

### Phase 4: Compatibility Transition

#### Slice 4.1

- Introduce a new parser entry point such as `parse_package(...)`
- Make it the preferred front-end API
- Add tests for single-file and folder input through the new API

#### Slice 4.2

- Decide the future of `AstParser::parse()`
- temporary compatibility shim
- or full migration away from it
- Add tests that lock down whichever transition choice is made

#### Slice 4.3

- Update the CLI to consume the new parser result shape if needed
- Keep user-facing parse diagnostics unchanged or better
- Add integration tests for single-file and folder compiles

### Phase 5: Visibility Preparation

#### Slice 5.1

- Ensure declaration nodes with visibility options remain tied to source units
- Add tests that `exp` and `hid` declarations retain unit-aware placement

#### Slice 5.2

- Add parser-visible or parsed-package-visible metadata needed later to distinguish:
- package-wide declarations
- namespace-contained declarations
- file-private declarations
- Do not resolve them yet
- Add tests for same-package different-file cases

### Phase 6: Hardening

#### Slice 6.1

- Audit all parser-owned declaration families for correct source-unit assignment
- Add tests for any missed declaration form

#### Slice 6.2

- Audit all parser error paths so cross-file failures still point at the right file and boundary
- Add exact-location tests

#### Slice 6.3

- Ensure the lexer/parser boundary contract remains explicit and documented
- Add or tighten tests around boundary token handling

### Phase 7: Docs And Handoff

#### Slice 7.1

- Update:
- `FRONTEND_CONTRACT.md`
- `PROGRESS.md`
- `README.md`
- any affected book notes if implementation now truly follows the book better

#### Slice 7.2

- Rewrite `PLAN.md` as a completion record for this source-layout alignment phase
- Promote `PLAN_NEXT.md` as the active next-phase resolver plan

## 14. Definition Of Done

This plan is complete only when all of the following are true:

- parser success preserves source origins, not just parse failures
- folder parsing returns distinct source units, not only one flattened root
- package and namespace information survive parsing in a structured way
- file roots are declaration-oriented rather than mixed script-like surfaces
- retained comments at file root have an explicit, documented model
- later enforcement of `hid` and import/export semantics is now structurally possible
- tests broadly cover package/source-unit behavior, not just one or two happy paths
- the resolver plan in `PLAN_NEXT.md` can start from a stable source-layout-aware parser boundary

## 15. After This Plan

Only after this plan is complete should the workspace proceed to the preserved next-phase plan in `PLAN_NEXT.md`:

- `fol-resolver`
- package/shared-scope name resolution
- namespace resolution
- import target resolution
- `exp`/`hid` enforcement
- later type and semantic phases

That next phase should consume a parser result that already understands package layout instead of reconstructing it after the fact.
