# Parser Completion Plan

This file is the working parser-completion backlog for the FOL project as of 2026-03-09.

It is based on the current code, not on older docs:
- `cargo test -q --test integration` is green: `896 passed`
- `cargo clippy --workspace --all-targets --all-features -q` is green
- the parser is already broad and stable

That means the remaining work is not “make the parser work at all”.
The remaining work is:
- finish intentionally narrow grammar sectors
- widen partial declaration/body forms
- decide whether some currently-forbidden syntactic forms should become legal
- continue paying down parser structure debt where the files are still close to the line cap


## 1. Current Baseline

### 1.1 Implemented and Stable

The parser already has working coverage for:

- top-level bindings and declarations:
  - `var`
  - `let`
  - `con`
  - `lab`
  - `use`
  - `ali`
  - `typ`
  - `def`
  - `seg`
  - `imp`
  - `std`
  - `fun`
  - `pro`
  - `log`

- routine surfaces:
  - normal routine headers
  - alternative routine headers
  - generic headers
  - capture lists
  - inquiry clauses
  - flow bodies
  - anonymous `fun` / `pro` / `log`
  - shorthand anonymous functions
  - pipe lambdas

- statements and control flow:
  - nested blocks
  - `if`
  - `when`
  - `while`
  - `loop`
  - `for`
  - `each`
  - `return`
  - `break`
  - `yeild`
  - flow-bodied branches

- expression surfaces:
  - precedence ladder
  - unary operators
  - arithmetic / logical / comparison ops
  - `in`
  - `has`
  - `is`
  - `as`
  - `cast`
  - ranges
  - container literals
  - field access
  - index access
  - slice access
  - pattern access
  - availability access
  - calls
  - method calls
  - general invoke
  - qualified path expressions
  - rolling expressions
  - pipe expressions

- type parsing / lowering:
  - qualified type references
  - bracketed type references
  - container aliases
  - array / matrix aliases
  - source-kind aliases
  - scalar lowering
  - function type references
  - `mod`
  - `blk`
  - `tst`
  - `any`
  - `none`
  - record / entry type declarations

- parser-side semantic validation:
  - duplicate parameter / generic / member rejection
  - custom-error `report(...)` validation
  - forward return-type lookup for report-call resolution
  - explicit diagnostics for many malformed bracket / separator cases

### 1.2 Important Recently-Fixed Infrastructure

The lexer had a stage-3 bootstrap spin bug on short inputs.
That is fixed in:
- `fol-lexer/src/lexer/stage3/element.rs`

This matters because parser-finish work is now on real grammar issues again, not hidden behind a stalled runner.


## 2. What Is Still Not Finished

This section is the actual remaining parser scope.

The best signal is the code itself:

- `fol-parser/src/ast/parser_parts/declaration_parsers.rs`
  - definition declarations currently support only a small set of target kinds

- `fol-parser/src/ast/parser_parts/standard_declaration_parsers.rs`
  - standard bodies are intentionally partial by kind
  - kind option brackets are intentionally stubbed

- `fol-parser/src/ast/parser_parts/type_references_and_blocks.rs`
  - function-type defaults are intentionally rejected

- `fol-parser/src/ast/parser_parts/routine_headers_and_type_lowering.rs`
  - routine generic headers intentionally reject defaults and variadics


## 3. Remaining Work By Sector

### 3.1 Definition Declarations (`def`)

#### Done

- `def name: mod[...] = { ... }`
- `def name: blk[...] = { ... }`
- `def name: tst[...] = { ... }`
- quoted names
- single-quoted names
- empty `blk` marker forms
- trailing semicolon support
- nested defs
- definition visibility options

#### Current Limit

In `declaration_parsers.rs`, `def` is explicitly limited to:
- `mod[...]`
- `blk[...]`
- `tst[...]`

Current diagnostic:
- `"Definition declarations currently support only mod[...], blk[...], or tst[...] types"`

#### Remaining Decisions / Work

Need to decide whether `def` should widen to support:
- source-kind definitions directly:
  - `url[...]`
  - `loc[...]`
  - `std[...]`
- richer definition targets if the language book describes them
- additional typed definition bodies beyond block/module/test semantics

#### Recommended Slice Order

1. Decide the full legal `def` type matrix from the language spec
2. Add missing legal `def` kinds one family at a time
3. Add body-shape restrictions per kind if needed
4. Add targeted diagnostics for wrong-body / wrong-kind combinations


### 3.2 Segment Declarations (`seg`)

#### Done

- top-level `seg`
- nested `seg`
- quoted / keyword names
- empty marker form
- empty option-marker form
- visibility options
- module-type validation

#### Open Questions

Current parser behavior strongly treats `seg` as a module-like declaration.

Still unclear / likely unfinished:
- whether `seg` should support anything beyond module-typed bodies
- whether `seg` option brackets should grow beyond visibility-only handling
- whether `seg` should carry richer metadata than the current AST keeps

#### Recommended Slice Order

1. Verify full `seg` grammar from the book
2. Add any non-module segment forms if they exist
3. Add tests for segment-local declaration restrictions if the spec requires them


### 3.3 Implementation Declarations (`imp`)

#### Done

- top-level and nested `imp`
- quoted names
- generic headers
- empty marker forms
- empty option-marker forms
- declaration visibility options
- duplicate / malformed option diagnostics

#### Current Limits

The parser accepts `imp` bodies, but there is no evidence yet that the full implementation surface is complete.

Potential gaps:
- richer `imp` body members if the spec allows more than current routine/field-like usage
- implementation-specific options beyond declaration visibility
- constraints or inheritance-style forms if the language has them

#### Recommended Slice Order

1. Verify whether `imp` has more body forms than current coverage
2. Add implementation-specific options if the spec defines them
3. Add validation around illegal implementation members if needed


### 3.4 Standard Declarations (`std`)

This is the biggest intentionally-partial declaration sector left.

#### Done

- `std name: pro = { ... }`
- `std name: blu = { ... }`
- `std name: ext = { ... }`
- nested `std`
- declaration visibility options
- empty `std[]`
- empty `pro[]`, `blu[]`, `ext[]` kind brackets
- duplicate-member rejection

#### Current Limits In Code

In `standard_declaration_parsers.rs`:

- protocol standards currently support only routine signatures
- blueprint standards currently support only field declarations
- extended standards currently support only routine signatures and field declarations
- kind option brackets currently support only empty brackets

That means this sector is only partially complete by design.

#### Remaining Work

Potential remaining grammar depending on spec:
- protocol bodies:
  - associated types
  - constants
  - default implementations
  - inquiry-like or contract clauses

- blueprint bodies:
  - methods
  - constants
  - grouped declarations
  - metadata on fields beyond current retention

- extended bodies:
  - richer member mix
  - method bodies vs signatures
  - override / extension markers if any exist

- kind-specific options:
  - `pro[...]`
  - `blu[...]`
  - `ext[...]`
  beyond empty placeholders

#### Recommended Slice Order

1. Finish `std` kind option grammar
2. Finish `pro` standard body surface
3. Finish `blu` standard body surface
4. Finish `ext` standard body surface
5. Add diagnostics that explain illegal member kinds by standard kind


### 3.5 Type Declaration Bodies (`typ`)

#### Done

- aliases
- records
- entries
- record marker syntax
- entry marker syntax
- generic headers
- type options
- metadata retention for:
  - defaults
  - binding options
  - binding alternatives
  - `var`
  - `lab`
  - `con`

#### Likely Remaining Work

The parser keeps richer field/variant metadata now, but there may still be type-body forms not represented yet, depending on the book:
- method members inside types
- attached implementations / contracts inside types
- computed fields or inquiry-like members
- type-level constraints or `where(...)` surfaces

#### Recommended Slice Order

1. Compare book type-body grammar against current `Record` / `Entry` support
2. Add missing member categories if they exist
3. Decide whether current AST is enough to retain those forms without lossy lowering


### 3.6 Type References and Type Lowering

#### Done

- qualified names
- quoted and single-quoted type segments
- bracketed type args
- special/container/source-kind lowering
- scalar lowering
- function-type references
- array / matrix / test-type lowering
- missing-close diagnostics for many bracket shapes

#### Current Intentional Limits

- function type defaults are still rejected
- some error messages are still shape-specific rather than fully normalized

#### Remaining Work

Potential remaining type-reference sectors:
- more source-kind shapes if the language book defines them
- richer function-type syntax
- generic constraint forms not yet modeled
- type unions / intersections / tagged forms if the language has them and they are not already represented elsewhere

#### Recommended Slice Order

1. Confirm full type-expression grammar from the book
2. Add any missing first-class type operators
3. Normalize diagnostics for all malformed bracket / nested-type cases


### 3.7 Routine Generic Headers

#### Done

- constrained generic names
- unconstrained generic names
- duplicate generic rejection
- separator flexibility

#### Current Intentional Limits

In `routine_headers_and_type_lowering.rs`:
- generic headers reject defaults
- generic headers reject variadics

#### Remaining Work

Need a language decision:
- Are defaults in routine generic headers meant to exist?
- Are variadic generic headers meant to exist?

If yes:
- widen parser
- add AST support where needed
- add diagnostics for ambiguous generic/param interactions

If no:
- the current behavior is already “finished enough”
- keep as documented parser restriction


### 3.8 Function-Type Parameters

#### Done

- typed params
- grouped params
- return types
- usage across declarations / bindings / aliases

#### Current Intentional Limit

- function-type parameters reject default values

This is explicit in `type_references_and_blocks.rs`.

#### Remaining Work

Need a language decision:
- should `{fun (...)}` type signatures allow defaulted parameters?

If yes:
- parser widening is still open

If no:
- this sector is already complete enough


### 3.9 Inquiries (`where(...)`)

#### Done

- routine inquiries
- multiple targets
- `self`
- `this`
- named targets
- quoted targets
- qualified targets
- block bodies
- flow bodies
- declaration/control/block/routine bodies in flow form
- inquiry retention on anonymous routines / pipe lambdas / shorthand lambdas

#### Likely Remaining Work

This sector is broad already.
What might still remain:
- tighter inquiry-target validation if only some target shapes are legal semantically
- richer inquiry-body restrictions if the book imposes them
- AST refinement if inquiry targets should be structural instead of stored as strings

#### Recommended Slice Order

1. Decide whether `Inquiry.target` must stop being a `String`
2. Add structural inquiry targets in AST if semantic work needs it
3. Add any semantic restrictions later, not in the parser first


### 3.10 Anonymous Routines and Pipe Lambdas

#### Done

- anonymous `fun` / `pro` / `log`
- shorthand anonymous functions
- captures
- inquiries
- flow bodies
- return/error types
- grouped/default/variadic params for pipe lambdas
- separator flexibility

#### Likely Remaining Work

Potential remaining surfaces:
- anonymous routines as members in more declaration contexts
- additional shorthand sugar from the book, if any is still undocumented in tests
- semantic restrictions for anonymous `log` or `pro` forms if needed later

#### Recommended Slice Order

1. Audit the book for any anonymous syntax variants not yet covered
2. Add missing variants only if they exist in the spec


### 3.11 Rolling Expressions

#### Done

- basic rolling
- multi-binding rolling
- `if` / `when` filters
- silent binders
- typed binders
- quoted binders
- keyword binders
- `var` binders
- comma/semicolon separator support

#### Likely Remaining Work

Potential gaps:
- richer guards
- nested destructuring binders
- availability/pattern integration inside rolling binders if the spec allows it

#### Recommended Slice Order

1. Verify whether rolling supports destructuring or more advanced binders
2. Add those only if they are real language syntax


### 3.12 Access / Invoke / Pattern / Availability Grammar

#### Done

- field access
- index access
- slices
- reverse slices
- pattern access
- availability access
- invoke expressions
- invoke statements
- availability invokes
- assignment targets for field/index/slice/pattern

#### Likely Remaining Work

Potential gaps:
- more postfix chaining combinations
- structural AST retention for availability syntax if later semantic stages need more than the current target-node model
- parser-side validation for illegal assignment targets beyond what already exists

#### Recommended Slice Order

1. Audit the book for any missing postfix/access sugar
2. Leave semantic validity checks for later passes unless syntax requires them


## 4. Structural / AST Debt Still Left

The parser works, but there are still some architecture choices that may need cleanup.

### 4.1 `Inquiry.target` Is Still a String

Current AST:
- `AstNode::Inquiry { target: String, body: Vec<AstNode> }`

This is enough for current parsing/tests, but it may become limiting if later passes need:
- qualified target structure
- quoted target provenance
- distinction between `self`, `this`, plain identifiers, and qualified targets

### 4.2 Some Declarations Still Lower Into Reused Shapes

The parser has already improved this with dedicated `SegDecl`, `ImpDecl`, `DefDecl`, etc.
Still worth watching:
- whether `std` members need richer AST nodes than reusing existing declaration nodes
- whether type/standard member metadata needs its own first-class node types

### 4.3 Type / Routine Parsing Modules Are Near the Line Cap

Current biggest parser files:
- `routine_headers_and_type_lowering.rs`
- `type_references_and_blocks.rs`

These are still under the cap, but they remain the pressure points for future work.

If new parser features land there, plan to split further instead of letting them creep back over the cap.


## 5. Practical Definition Of “Parser Finished”

The parser should be considered finished when all of the following are true:

1. Every grammar surface documented in the book has either:
   - parser support
   - or an explicit, intentional rejection with rationale

2. Intentionally narrow sectors have been resolved:
   - `def`
   - `std`
   - generic-header feature decisions
   - function-type default decision

3. All parser modules remain under the file-size cap

4. The integration suite stays green while widening the remaining sectors

5. AST retention is good enough that later semantic/compiler stages do not have to recover lost syntax from flattened strings


## 6. Recommended Work Order

This is the recommended order to actually finish the parser from here.

### Phase A: Finish Intentionally-Narrow Declaration Sectors

1. finish `std` kind options
2. finish `std` protocol body surface
3. finish `std` blueprint body surface
4. finish `std` extended body surface
5. widen `def` target kinds if the spec requires more than `mod` / `blk` / `tst`

### Phase B: Resolve Intentional Syntax Restrictions

6. decide whether routine generic headers allow defaults
7. decide whether routine generic headers allow variadics
8. decide whether function-type params allow defaults

### Phase C: Audit Remaining Book Grammar

9. audit type-body grammar against current parser
10. audit anonymous/rolling/access sugar against current parser
11. add any remaining documented syntax not already represented by fixtures

### Phase D: AST and Structure Cleanup

12. decide whether inquiry targets need structural AST
13. split any parser modules that approach the line cap again
14. tighten diagnostics where parser messages are still generic rather than shape-specific


## 7. Suggested Next 20 Slices

If work continues in slices, this is the most sensible next batch queue.

1. `std` kind option grammar
2. protocol standards: constant members
3. protocol standards: associated-type-like members if spec has them
4. protocol standards: default routine bodies if spec has them
5. blueprint standards: routine members if spec has them
6. blueprint standards: constant members
7. extended standards: richer mixed-member bodies
8. extended standards: option validation by kind
9. `def` target widening: source-kind definitions if allowed
10. `def` target widening: additional body validation per target kind
11. routine generic defaults, if allowed
12. routine generic variadics, if allowed
13. function-type defaults, if allowed
14. inquiry AST target structuring
15. type-body audit slice
16. remaining anonymous syntax audit slice
17. remaining rolling syntax audit slice
18. remaining postfix/access syntax audit slice
19. type/routine parser refactor split if needed
20. final parser coverage/documentation pass


## 8. Non-Goals For The Parser Layer

These should not be forced into the parser unless syntax requires it:

- deep semantic type-checking
- ownership/borrow semantics
- implementation compatibility checking
- standard conformance checking
- codegen-oriented normalization

Those belong later.
The parser’s job is to:
- accept the full grammar
- reject malformed syntax clearly
- preserve enough structure for later phases


## 9. Summary

The parser is already in the “mostly complete” stage.

What remains is concentrated:
- `std` is still intentionally partial
- `def` is still intentionally narrow
- some parameter/type feature decisions are still unresolved by design
- a few AST representation choices may need refinement before semantic work grows

So the path to “finished parser” is now clear:
- complete the intentionally-limited sectors
- decide the still-open syntax rules
- keep the AST lossless enough for later compiler phases
