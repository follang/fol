# Parser Completion Ledger

## 1. Current Baseline

Current verified state:
- `cargo test -q --test integration` passes with `1070` integration tests.
- `cargo clippy --workspace --all-targets --all-features -q` passes.
- The parser is no longer in backlog mode for broad grammar sectors.

Practical meaning:
- the core grammar is implemented
- the major book sugar surfaces are implemented
- the processor/concurrency syntax that was concrete enough to parse is implemented
- the remaining items are mostly spec-ambiguity or semantic-policy questions, not parser gaps

## 2. Completed From This Plan

The following sectors that were originally open are now implemented and covered.

### 2.1 Destructuring And Unpacking Bindings

Implemented:
- destructuring binding AST
- rest bindings
- nested destructuring
- grouped destructuring
- `var`
- `lab`
- widened binding-path support

Covered by:
- `test/parser/test_parser_parts/destructuring_bindings.rs`

### 2.2 Type Shorthand Sugar

Implemented:
- `?T`
- `!T`

Lowering:
- `?T` lowers to optional
- `!T` lowers to never

Covered by:
- `test/parser/test_parser_parts/special_type_references.rs`

### 2.3 Type Limits

Implemented:
- limited type AST
- `type[...][.limit(...)]` parsing
- preserved lowering through existing type-reference paths

Covered by:
- `test/parser/test_parser_parts/special_type_references.rs`

### 2.4 Matching Expression Sugar

Implemented:
- expression-shaped matching with `if(...) { ... }`
- expression-shaped matching with `when(...) { ... }`
- `is`
- `in`
- `has`
- wildcard/default branches
- arrow bodies with `->` and `=>`

Covered by:
- `test/parser/test_parser_parts/matching_expressions.rs`

### 2.5 Builtin Leading-Dot Calls

Implemented:
- root `.echo(...)`
- root builtin expression calls
- root builtin statement calls
- use in block, flow, inquiry, and pipe contexts

Covered by:
- `test/parser/test_parser_parts/leading_dot_builtin_calls.rs`

### 2.6 Templates And Postfix Template Sugar

Implemented:
- postfix template access like `file$`
- structural AST node for template postfix usage

Covered by:
- `test/parser/test_parser_parts/template_calls.rs`

### 2.7 Record Constructors / Typed Initializers

Implemented:
- structural record initializer AST
- named field initializers `{ field = value, ... }`
- nested record initializers

Current note:
- named initializers are implemented
- positional `{ value1, value2, ... }` remains represented as ordinary container literals because the book examples do not define a parser-only disambiguator that can separate positional record construction from ordinary braced value literals without semantic type context

Covered by:
- `test/parser/test_parser_parts/record_initializers.rs`

### 2.8 Access Filters With Capture Or Assignment

Implemented:
- `expr => Name` inside access patterns
- wildcard capture `* => Name`
- structural wildcard/capture pattern nodes
- support in ordinary pattern access
- support in availability access

Covered by:
- `test/parser/test_parser_parts/access_pattern_captures.rs`

### 2.9 Optional Chaining / Chaining Sugar

Implemented:
- explicit postfix unwrap `value!`

Status note:
- the chapter’s concrete parser surface was mainly the unwrap form shown in examples
- broader nil-safe chain operators are not specified with a concrete tokenized syntax in the current book text

Covered by:
- `test/parser/test_parser_parts/chaining_sugar.rs`

### 2.10 Async / Await

Implemented:
- lexer keywords for `async` and `await`
- structural `async` pipe stage
- structural `await` pipe stage

Covered by:
- `test/parser/test_parser_parts/pipe_expressions.rs`
- `test/parser/test_parser_parts/book_processor_examples.rs`

### 2.11 Coroutines / Channels / Select / Mutex Parameters

Implemented:
- channel types `chn[...]`
- structural `channel[tx]` / `channel[rx]`
- structural `select(channel as c) { ... }`
- coroutine spawn `[>]expr`
- mutex parameters `((name))`

Covered by:
- `test/parser/test_parser_parts/channel_access_expressions.rs`
- `test/parser/test_parser_parts/select_statements.rs`
- `test/parser/test_parser_parts/spawn_expressions.rs`
- `test/parser/test_parser_parts/mutex_parameters.rs`
- `test/parser/test_parser_parts/book_processor_examples.rs`

### 2.12 Book Example Alignment

Added or expanded direct book-shape coverage for:
- processor/eventual examples
- coroutine/select/channel examples
- template postfix examples
- named record initializer examples
- nested record initializer examples

## 3. What Is Still Open

These are the only remaining items that still qualify as open from the parser-plan perspective.

### 3.1 Positional Record Initialization

Status:
- parser-ambiguous

Reason:
- `{ a, b, c }` is already the container literal surface
- the current book text does not define a parser-only discriminator for “this brace literal is positional record construction” versus “this brace literal is a normal literal container”
- resolving that cleanly may require semantic type-context rather than syntax alone

Conclusion:
- not an unresolved parser bug
- unresolved language-design / parse-vs-typecheck boundary

### 3.2 Broader Optional-Chaining Operators

Status:
- spec-ambiguous

Reason:
- the current chapter explains optional chaining conceptually
- it does not clearly define a concrete extra token/operator family beyond the unwrap form already implemented

Conclusion:
- no concrete parser work should be forced until the book defines exact tokens and examples

### 3.3 Async As Routine Modifier Outside Pipe Form

Status:
- spec-ambiguous

Reason:
- the book examples show `| async` and `| await`
- they do not clearly define a stable declaration/header surface such as `fun[async]` or similar

Conclusion:
- the concrete parser surface from the examples is implemented
- anything beyond that should wait for clearer spec wording

### 3.4 Any Further `def` / `seg` / `imp` Widening

Status:
- only if the book adds or clarifies extra grammar

Reason:
- the parser already supports the concrete documented forms that were previously missing
- no further clear parser-only gaps remain there from the current book sweep

## 4. Honest Final Status

Current parser completion against the concrete book syntax:
- core grammar: done
- declaration grammar: done
- routine grammar: done
- expression grammar: done
- type sugar from this plan: done
- processor syntax from this plan: done
- book-shaped regression coverage for those sectors: done

Best summary:
- the parser backlog described by this plan is complete for all concrete, syntax-definable items
- the remaining items are not “missing parser work” so much as “book/spec wording is not concrete enough to justify new syntax”

## 5. Definition Of Done

For the purposes of this plan, the parser work is finished because:
- every concrete syntax family listed in the plan now parses
- direct regression coverage exists for the implemented sectors
- the integration suite is green
- `clippy` is green
- the only remaining entries are explicitly documented above as spec-ambiguous rather than parser omissions
