# FOL Intrinsics Plan

Last updated: 2026-03-15

This file defines the next compiler-design milestone: create
`fol-intrinsics`, the shared compiler-owned intrinsic registry that replaces the
current scattered “build-in” handling and becomes the main extension point for
language-owned operations such as:

- `.eq()`
- `.nq()`
- `.not()`
- `.lt()`
- `.gt()`
- `.ge()`
- `.le()`
- `.cast()`
- `.as()`
- `.len()`
- `.echo()`

and later many more.

The goal is not to add every intrinsic at once. The goal is to build the
infrastructure that makes adding intrinsics easy, centralized, testable, and
safe across:

- parser
- typecheck
- lowering
- future backends
- user-facing docs

## 0. Why This Exists

Right now, “builtins” are a mix of:

- book-only ideas
- parser-recognized dot-root calls
- keyword-style diagnostics like `panic`
- ad hoc typecheck/lowering special cases

That is not a good long-term shape.

Intrinsics are not standard-library functions. They are compiler-owned language
operations. They need one semantic owner.

Without that owner, every new intrinsic risks becoming:

- parser special casing
- typechecker branching
- lowering branching
- backend branching
- stale docs

all in different places.

`fol-intrinsics` should become the one place where a new intrinsic is declared,
categorized, version-gated, documented, and wired to semantic/lowering
behavior.

## 1. Current Repository Truth

This plan is based on a rescan of the current codebase and book.

### 1.1 Book Truth

Current builtin documentation lives mainly in:

- [`book/src/300_meta/100_buildin.md`](./book/src/300_meta/100_buildin.md)
- [`book/src/400_type/200_container.md`](./book/src/400_type/200_container.md)
- [`book/src/400_type/400_special.md`](./book/src/400_type/400_special.md)
- [`book/src/500_items/200_routines/_index.md`](./book/src/500_items/200_routines/_index.md)
- [`book/src/700_sugar/200_pipes.md`](./book/src/700_sugar/200_pipes.md)
- [`book/src/700_sugar/800_chaining.md`](./book/src/700_sugar/800_chaining.md)

The book currently lists many dot-prefixed operations such as:

- `.echo()`
- `.not()`
- `.cast()`
- `.as()`
- `.eq()`
- `.nq()`
- `.gt()`
- `.lt()`
- `.ge()`
- `.le()`
- `.de_alloc()`
- `.give_back()`
- `.size_of()`
- `.address_of()`
- `.pointer_value()`

and many older examples also use:

- `.len()`
- `.low()`
- `.high()`
- `.assert()`
- `.typeof()`
- `.range(...)`
- `.regex(...)`

### 1.2 Parser Truth

The parser already has explicit syntax support for dot-root builtin call forms:

- [`fol-parser/src/ast/parser_parts/expression_parsers.rs`](./fol-parser/src/ast/parser_parts/expression_parsers.rs)
- [`fol-parser/src/ast/parser_parts/statement_parsers.rs`](./fol-parser/src/ast/parser_parts/statement_parsers.rs)

So the language surface for:

```fol
.eq(a, b)
.len(items)
.echo(value)
```

already exists syntactically.

### 1.3 Typecheck Truth

Today, type checking has real semantic ownership only for a very small subset:

- `panic`
- `report`
- `check(...)`

Current evidence:

- [`fol-typecheck/src/exprs.rs`](./fol-typecheck/src/exprs.rs)

At the same time:

- explicit `as` casts are still rejected in `V1`
- explicit `cast` operators are still rejected in `V1`
- there is no general intrinsic registry for `.eq()`, `.not()`, `.lt()`, `.gt()`,
  `.len()`, `.echo()`, and friends

### 1.4 Lowering Truth

Lowering knows about recoverable-call surfaces and some shell/runtime surfaces,
but it does not have one general “intrinsic op registry”.

Current evidence:

- [`fol-lower/src/exprs.rs`](./fol-lower/src/exprs.rs)

So the compiler is still missing the abstraction layer that says:

“this source-level intrinsic is owned by the compiler, has this type contract,
and lowers in this exact way.”

### 1.5 External Reference Scan

This plan also takes cues from a few official language references:

- Zig 0.15.2 builtin functions:
  - very broad compiler-owned surface for casts, overflow helpers, layout/type
    introspection, memory helpers, and native interop
  - <https://ziglang.org/documentation/0.15.2/#Builtin-Functions>
- Go built-in functions:
  - a deliberately small but high-value builtin set such as `len`, `cap`,
    `make`, `append`, `copy`, `delete`, `clear`, `new`, `panic`, and `recover`
  - <https://go.dev/ref/spec#Built-in_functions>
- Rust primitive and `Result` APIs:
  - show that many “nice to have” operations are better on primitives/core
    types than as compiler intrinsics
  - <https://doc.rust-lang.org/std/primitive.bool.html>
  - <https://doc.rust-lang.org/std/primitive.str.html>
  - <https://doc.rust-lang.org/std/primitive.slice.html>
  - <https://doc.rust-lang.org/std/result/enum.Result.html>

Design takeaway:

- build a richer roadmap than the current tiny subset
- but do not promote every convenient helper into compiler magic

## 2. Core Design Decision

Create a new shared crate:

`fol-intrinsics`

It is **not** another pipeline stage. It is shared semantic infrastructure used
by multiple stages.

The dependency shape should be:

- parser may use it for surface recognition or reserved-name validation
- typecheck must use it for semantic ownership
- lower must use it for IR lowering decisions
- future backend must use it for implementation mapping

`fol-intrinsics` should become the compiler source of truth for:

- intrinsic names
- aliases
- categories
- version availability
- arity rules
- type rules
- purity/effect flags
- lowering strategy
- backend/runtime expectations
- documentation metadata

## 3. Accessibility Goal

This crate should be the easiest place in the compiler to extend.

When adding a new intrinsic, the desired workflow should eventually be:

1. add one registry entry
2. add one or two semantic callbacks or lowering descriptors
3. add tests
4. update docs

not:

1. patch parser
2. patch resolver
3. patch typecheck
4. patch lowering
5. patch CLI
6. hope all the names stay consistent

The registry should therefore be:

- declarative first
- searchable
- strongly typed
- version-aware
- testable in isolation

## 4. V1 Boundary For Intrinsics

This plan is for the current `V1` compiler boundary.

That means `fol-intrinsics` should distinguish three kinds of intrinsic
surfaces:

### 4.1 V1 Intrinsics To Support For Real

These should become real semantic intrinsics in the current compiler:

- `.eq()`
- `.nq()`
- `.lt()`
- `.gt()`
- `.ge()`
- `.le()`
- `.not()`
- `.len()`
- `.echo()`
- `check(...)`
- `panic(...)`

Possible `V1` cast surfaces if we deliberately accept them in this milestone:

- `.cast()`
- `.as()`

but only if we define a clear `V1` conversion contract first.

### 4.2 V1 Intrinsics To Keep As Explicitly Unsupported

These may stay parsed but must fail with explicit diagnostics until their real
semantic owner exists:

- `.de_alloc()`
- `.give_back()`
- `.address_of()`
- `.pointer_value()`
- `.borrow_from()`
- ownership-facing memory helpers
- pointer helpers
- C-ABI-facing helpers

These are later than current `V1` because they depend on:

- ownership/borrowing semantics
- pointer semantics
- backend/runtime ABI

### 4.3 Book-Visible But Not Yet Real

Some book surfaces may need to remain in one of these states for now:

- parser-visible
- typecheck-explicitly-unsupported
- lowering-explicitly-unsupported

That is acceptable, but only if diagnostics are direct and intentional.

### 4.4 Large Roadmap, Narrow Semantics

This plan should intentionally track a **large** candidate intrinsic set, but
the compiler should stay strict about what really becomes intrinsic.

Use this rule:

- make it an intrinsic if it is:
  - syntax-sensitive
  - type-system-sensitive
  - recoverable-effect-sensitive
  - layout/backend-sensitive
  - required across all backends
- prefer `core` or `std` library APIs if it is:
  - ordinary data manipulation
  - collection algorithms
  - string/text convenience
  - serialization
  - filesystem/network/process behavior

That lets `fol-intrinsics` stay rich without turning the whole language into ad
hoc compiler special cases.

## 5. Intrinsic Families

`fol-intrinsics` should group intrinsics into clear families.

### 5.1 Comparison Intrinsics

Examples:

- `.eq(a, b)`
- `.nq(a, b)`
- `.lt(a, b)`
- `.gt(a, b)`
- `.ge(a, b)`
- `.le(a, b)`

Questions to settle:

- what scalar types are allowed in `V1`
- whether string equality is allowed in `V1`
- whether shell equality is allowed in `V1`
- whether records/containers are rejected in `V1`
- whether these are just source-level spellings over already-supported typed
  operator families, or distinct lowering ops

Recommended `V1` direction:

- treat these as intrinsic spellings over the same semantic comparison families
  already accepted by operator typing
- keep their result type `bol`
- reject unsupported families explicitly

### 5.2 Boolean / Logical Intrinsics

Examples:

- `.not(value)`

Possible later additions:

- `.and(a, b)`
- `.or(a, b)`
- `.xor(a, b)`

Recommended `V1` direction:

- support `.not(bol)` only
- leave richer logical/fold surfaces later unless there is a strong need now

### 5.3 Conversion Intrinsics

Examples:

- `.cast(value, Type)`
- `.as(value, Type)`

or whatever final source shape the language keeps

This family is dangerous because it needs a coherent conversion policy.

Recommended direction:

- build the registry and diagnostics now
- do not mark casts as real `V1` intrinsics until the conversion rules are
  frozen
- make the unsupported boundary explicit if casts stay deferred

### 5.4 Diagnostic / Environment Intrinsics

Examples:

- `.echo(value)`
- `.assert(cond)`

Questions:

- is `.echo` lowered as a dedicated IR op or as a runtime/builtin call stub
- is `.assert` a real `V1` intrinsic or still book-only
- are these backend-required or debug-only

Recommended `V1` direction:

- make `.echo` real early if it helps backend bring-up and demo programs
- keep `.assert` pending unless its behavior is frozen

### 5.5 Introspection / Size / Shape Intrinsics

Examples:

- `.len(value)`
- `.low(value)`
- `.high(value)`
- `.size_of(value_or_type)`
- `.typeof(value)`

Recommended `V1` direction:

- prioritize `.len()` because container examples already depend on it
- keep `.size_of()` and `.typeof()` deferred until backend/runtime layout
  policy is ready

### 5.6 Memory / Ownership Intrinsics

Examples:

- `.de_alloc()`
- `.give_back()`
- `.address_of()`
- `.pointer_value()`

Recommended direction:

- registry entries now
- explicit `V3` diagnostics now
- no fake `V1` semantics

### 5.7 Arithmetic And Numeric Helper Intrinsics

Candidate intrinsic spellings worth tracking:

- `.add()`
- `.sub()`
- `.mul()`
- `.div()`
- `.rem()`
- `.neg()`
- `.abs()`
- `.min()`
- `.max()`
- `.clamp()`
- `.pow()`
- `.sqrt()`
- `.floor()`
- `.ceil()`
- `.round()`
- `.trunc()`

Important boundary:

- ordinary arithmetic operators should remain the primary user surface
- intrinsic spellings are still useful for templates/macros, uniform dispatch,
  and backend-directed behavior

Recommended priority:

- near-term candidates: `.abs()`, `.min()`, `.max()`, `.clamp()`
- later numeric-policy candidates: `.pow()`, `.sqrt()`, rounding family

### 5.8 Overflow, Wrapping, And Checked Numeric Intrinsics

Inspired strongly by Zig-style builtin control over numeric behavior, track:

- `.checked_add()`
- `.checked_sub()`
- `.checked_mul()`
- `.checked_div()`
- `.checked_shl()`
- `.checked_shr()`
- `.wrapping_add()`
- `.wrapping_sub()`
- `.wrapping_mul()`
- `.wrapping_shl()`
- `.wrapping_shr()`
- `.saturating_add()`
- `.saturating_sub()`
- `.saturating_mul()`
- `.overflowing_add()`
- `.overflowing_sub()`
- `.overflowing_mul()`

These are high-value because they are:

- explicit
- backend-relevant
- hard to model cleanly as ordinary library helpers

But they need a frozen result contract before implementation.

### 5.9 Bitwise And Bit-Introspection Intrinsics

Important systems-language candidates:

- `.bit_and()`
- `.bit_or()`
- `.bit_xor()`
- `.bit_not()`
- `.shl()`
- `.shr()`
- `.rotl()`
- `.rotr()`
- `.pop_count()`
- `.clz()`
- `.ctz()`
- `.byte_swap()`
- `.bit_reverse()`
- `.bit_width()`

These map naturally to backend instructions and should be part of the roadmap
even if they land after the first intrinsic batch.

### 5.10 Shape, Capacity, And Container Query Intrinsics

Candidate container/query surfaces:

- `.len()`
- `.cap()`
- `.is_empty()`
- `.low()`
- `.high()`
- `.contains()`

Recommended split:

- likely intrinsic: `.len()`, `.cap()`, maybe `.is_empty()`
- likely `core/std`: richer query and mutation helpers unless the language
  decides they are primitive

### 5.11 Container Mutation And Construction Candidates

Useful but likely **not** first-wave compiler intrinsics:

- `.append()`
- `.push()`
- `.pop()`
- `.insert()`
- `.remove()`
- `.delete()`
- `.clear()`
- `.copy()`
- `.clone()`

These are worth tracking, especially because Go treats some mutation-adjacent
operations as builtins, but many of them may fit better in `core` collection
APIs than in compiler intrinsics.

### 5.12 Optional, Error, And Recoverable Helper Intrinsics

Current and future candidates:

- `check(...)`
- `.is_nil()`
- `.is_err()`
- `.unwrap_or()`
- `.unwrap_or_else()`
- `.expect()`
- `.ok_or()`
- `.err_or()`
- `.map_ok()`
- `.map_err()`

Important boundary:

- `check(...)` is clearly intrinsic because it inspects recoverable routine call
  results
- many shell/result convenience helpers may be better as `core` APIs unless they
  need compiler-owned semantics

### 5.13 Introspection And Layout Intrinsics

Track these as strong intrinsic candidates:

- `.size_of()`
- `.align_of()`
- `.type_of()`
- `.type_name()`
- `.field_count()`
- `.tag_name()`
- `.discriminant()`
- `.has_field()`
- `.has_method()`

These depend on compiler type knowledge and should eventually be owned by the
intrinsic registry even if they are not `V1` day-one features.

### 5.14 Allocation, Construction, And Lifetime Candidates

Track explicitly:

- `.new()`
- `.make()`
- `.zeroed()`
- `.default()`
- `.de_alloc()`
- `.give_back()`
- `.move()`
- `.deep_copy()`

These overlap with runtime, ownership, and allocation policy, so many should
remain deferred or non-`V1`.

### 5.15 Pointer And Address Intrinsics

Track explicitly:

- `.address_of()`
- `.pointer_value()`
- `.borrow_from()`
- `.offset_of()`
- `.field_ptr()`
- `.ptr_cast()`
- `.int_from_ptr()`
- `.ptr_from_int()`

These belong later, but they need reserved names and clear diagnostics.

### 5.16 Text, Bytes, And Regex-Adjacent Candidates

Candidates users will naturally want:

- `.starts_with()`
- `.ends_with()`
- `.contains()`
- `.split()`
- `.trim()`
- `.to_lower()`
- `.to_upper()`
- `.regex()`

Most of these should probably stay in `core` or `std`, not in compiler
intrinsics. They are still worth tracking so the docs and future design do not
blur the boundary.

### 5.17 Diagnostics, Debug, And Compiler-Control Intrinsics

Track:

- `.echo()`
- `.assert()`
- `.debug()`
- `panic(...)`
- `.unreachable()`
- `.compile_error()`
- `.compile_log()`
- `.trace()`

Some are runtime-facing, some compile-time-facing. The registry should be able
to represent both even if only `.echo()` and `panic(...)` are implemented
initially.

### 5.18 FFI And Native Artifact Intrinsics

Reserve room for future C-ABI and package/native integration surfaces:

- `.c_import()`
- `.header()`
- `.extern_symbol()`
- `.link_name()`
- `.abi_cast()`
- `.call_conv()`

These are not immediate `V1` work, but the registry model should not block them.

## 6. Surface Model

The registry needs to model more than just a name string.

Each intrinsic entry should eventually declare at least:

- stable `IntrinsicId`
- canonical name
- optional aliases
- source family
  - dot-root call
  - keyword call
  - postfix surface
  - operator-like alias
- category
- minimum compiler version milestone
  - `V1`
  - `V2`
  - `V3`
- current implementation status
  - implemented
  - unsupported
  - reserved
- purity/effect class
- arity rule
- overload family
- accepted argument shapes
- result shape
- recoverable-effect behavior
- lowering strategy
- backend expectation
- documentation summary
- examples and non-examples
- whether the surface should remain compiler-owned or eventually migrate to
  `core`

## 7. Lowering Strategy Model

Not every intrinsic should lower the same way.

The registry should support at least these lowering modes:

- lower to an existing general IR op
- lower to a dedicated intrinsic IR op
- lower to a backend-runtime hook
- lower to compile-time rejection
- lower to “not yet supported”

Examples:

- `.eq()` may lower to an existing comparison op
- `.not()` may lower to an existing boolean op
- `.len()` may lower to a dedicated `LengthOf` IR op or a runtime field access
- `.echo()` may lower to a backend runtime hook
- `.de_alloc()` may lower to `Unsupported(V3Only)`

## 8. Parser Integration Direction

The parser already recognizes dot builtin syntax. The key parser work is not to
grow more ad hoc branches.

Recommended parser responsibilities:

- keep parsing dot intrinsic syntax
- preserve the intrinsic name exactly
- optionally validate reserved builtin-root spelling through
  `fol-intrinsics`
- do not own type rules or semantics

The parser should not become the owner of:

- intrinsic arity semantics
- intrinsic overload semantics
- intrinsic version gating

## 9. Resolver Integration Direction

Resolver should stay mostly out of intrinsic semantics.

Recommended responsibilities:

- keep user-defined names and compiler-owned intrinsic names distinct
- ensure intrinsic roots do not collide with normal package resolution
- avoid pretending that `.eq` or `.echo` are imported symbols

Resolver should not perform:

- overload selection
- signature typing
- runtime mapping

## 10. Typecheck Integration Direction

Typecheck is where most intrinsic meaning should begin.

For each intrinsic, typecheck should own:

- arity validation
- argument type validation
- overload family selection
- result type determination
- recoverable-effect interaction
- unsupported diagnostics

Examples:

- `.eq(int, int) -> bol`
- `.eq(str, str) -> bol` if allowed
- `.len(seq[T]) -> int`
- `.not(bol) -> bol`
- `.echo(T) -> never` or unit-like behavior depending on final policy

Typecheck must also give better diagnostics than today:

- exact intrinsic name
- expected arity
- accepted type families
- version-gating guidance

## 11. Lower Integration Direction

Lowering should not rediscover intrinsic meaning from strings.

It should consume typed intrinsic selections from typecheck.

That means typecheck should hand lowering something like:

- selected intrinsic ID
- selected overload form
- typed operands
- result type

Then lowering can emit:

- intrinsic IR instruction
- ordinary IR instruction
- runtime call stub

without re-deciding what `.eq` means.

## 12. Future Backend Integration Direction

The first backend should consume lowered intrinsic ops, not raw source spellings.

The backend should be able to ask:

- what is the selected intrinsic
- what is the runtime contract
- does this intrinsic require backend runtime support

Examples:

- `.eq()` may compile to direct target-language comparison
- `.len()` may compile to field access or helper call
- `.echo()` may compile to a runtime print hook

This is another reason to build `fol-intrinsics` before backend implementation
grows too large.

## 13. Docs Direction

The old “build-in” page should eventually be replaced or renamed to reflect:

- the crate name `fol-intrinsics`
- the real implemented subset
- the V1/V2/V3 boundary

Docs should distinguish clearly between:

- compiler intrinsics
- core library functions
- standard library functions

That distinction matters because intrinsics are not imported library symbols.

## 14. Hard Definition Of Done

This milestone is complete only when:

- `fol-intrinsics` exists as a workspace crate
- at least one real intrinsic family is registry-owned end to end
- parser, typecheck, and lowering consume the registry instead of hard-coded
  string branching for that family
- unsupported intrinsic families fail with explicit version-aware diagnostics
- docs no longer describe a giant undifferentiated “builtin” surface
- adding a new intrinsic requires touching a small, predictable number of files

## 15. Execution Strategy

Do this in strict order:

1. create the crate and registry model
2. migrate the easiest real intrinsic family first
3. add explicit unsupported entries for dangerous late-version intrinsics
4. migrate additional `V1` families
5. only then sync docs

Do **not** try to implement every book-listed intrinsic in one pass.

Also do **not** let the larger roadmap force every candidate family into early
compiler magic. This plan is intentionally broader than the first
implementation batch.

## 16. Recommended First Real Families

The first migration batch should be:

1. comparison intrinsics
2. `.not()`
3. `.len()`
4. `.echo()`

These are the most useful because they:

- are clearly compiler-owned
- have obvious user value
- avoid ownership/C-ABI complexity
- help backend bring-up later

`cast/as` should be its own later sub-milestone unless the conversion rules are
frozen first.

## 17. Recommended Near-Following Families

After the first batch, the next most valuable families are:

1. shape/query:
   - `.cap()`
   - `.is_empty()`
2. assertion/debug:
   - `.assert()`
3. numeric helpers:
   - `.abs()`
   - `.min()`
   - `.max()`
   - `.clamp()`
4. bit introspection:
   - `.pop_count()`
   - `.clz()`
   - `.ctz()`
   - `.byte_swap()`
5. explicit unsupported but registry-owned:
   - ownership helpers
   - pointer helpers
   - FFI/C-ABI helpers

## 18. Explicit Non-Goals For The First Batch

Track these in the registry, but do not implement them in the first batch:

- ownership/lifetime helpers
- pointer/address helpers
- C-ABI helpers
- generic text-processing helpers
- container mutation helpers unless a later decision makes them truly primitive
- advanced compile-time/meta intrinsics
- full cast/conversion family before conversion rules are frozen

## 19. Implementation Slices

### Phase 0. Contract Freeze

- `0.1` `done` Scan all current dot-builtin parser entry points and list the
  exact syntactic builtin surfaces the parser accepts today.
  Current state:
  - parser dot-root builtin detection lives in
    [`fol-parser/src/ast/parser_parts/expression_parsers.rs`](./fol-parser/src/ast/parser_parts/expression_parsers.rs)
  - dot-root builtin calls currently parse as ordinary `FunctionCall { name,
    args }` nodes rather than a dedicated intrinsic AST node
  - keyword-style builtin statements currently parse through
    [`fol-parser/src/ast/parser_parts/statement_parsers.rs`](./fol-parser/src/ast/parser_parts/statement_parsers.rs)
    for `panic`, `report`, `check`, and `assert`
  - parser ownership is intentionally syntactic only; arity and semantic meaning
    are not frozen there
- `0.2` `done` Scan all current book-listed builtin names and classify them as:
  - `V1 implement now`
  - `V1 explicit unsupported`
  - `V2`
  - `V3`
  Current state:
  - `V1 implement now`: comparison family, `.not`, `.len`, `.echo`, `check`,
    `panic`
  - `V1 explicit unsupported`: `.cast`, `.as`, `.assert`, `.cap`, `.is_empty`
    until their contracts are frozen
  - `V3 explicit unsupported`: `.de_alloc`, `.give_back`, `.address_of`,
    `.pointer_value`, `.borrow_from`, pointer/lifetime helpers, C-ABI-facing
    helpers
  - likely `core/std`, not intrinsic`: mutation-heavy container helpers,
    text-processing helpers, regex-heavy helpers, serialization, filesystem,
    and network-like convenience APIs
- `0.3` `done` Freeze the naming decision:
  - crate name is `fol-intrinsics`
  - docs stop using “build-in” as the authoritative compiler term
  Current state:
  - the compiler architecture term is now `intrinsics`
  - the old book page is kept only as a migration target to be rewritten later
  - future code, docs, and diagnostics should prefer `intrinsic` terminology
- `0.4` `done` Freeze the first implemented families:
  - comparison
  - `.not()`
  - `.len()`
  - `.echo()`
  Current state:
  - the first real implementation batch is comparison + boolean negation +
    length + echo
  - `panic(...)` and `check(...)` are intentionally left for a later alignment
    slice so dot and keyword surfaces can share one registry contract cleanly
- `0.5` `done` Freeze the first explicit unsupported families:
  - ownership
  - pointers
  - deallocation
  - address/pointer helpers
  - C-ABI-adjacent surfaces
  Current state:
  - ownership/lifetime helpers, pointer helpers, and native/ABI helpers are
    registry-owned later surfaces, not current `V1` work
  - the first implementation round should give them stable names plus explicit
    unsupported diagnostics instead of accidental fallback behavior

### Phase 1. Crate Foundation

- `1.1` `done` Add new workspace crate `fol-intrinsics`.
  Current state:
  - `fol-intrinsics` now exists as a workspace member with a minimal public API
  - the root compiler crate depends on it so ordinary `make build` and
    `make test` compile the new crate immediately
  - root smoke coverage now proves the crate is wired into the active build
- `1.2` `done` Add public registry model:
  - `IntrinsicId`
  - `IntrinsicCategory`
  - `IntrinsicSurface`
  - `IntrinsicAvailability`
  - `IntrinsicStatus`
  Current state:
  - the crate now exposes typed identity, category, surface, availability, and
    status enums instead of raw strings
  - root smoke coverage and crate-local unit coverage both exercise the public
    model API
- `1.3` `done` Add registry entry type with:
  - name
  - aliases
  - arity rule
  - category
  - version
  - doc string
  - lowering mode
  Current state:
  - the crate now exposes a typed `IntrinsicEntry` plus arity and lowering-mode
    enums
  - root smoke coverage now proves entries can be constructed without stringly
    typed tuple glue
- `1.4` `done` Add one canonical static registry table.
  Current state:
  - the crate now exposes one canonical static registry with implemented,
    unsupported, and deferred entries from the frozen first-batch families
  - stable IDs now exist for comparison, boolean, query, diagnostic,
    recoverable, conversion, memory, and pointer entries
- `1.5` `done` Add lookup APIs:
  - by canonical name
  - by alias
  - by surface family
  Current state:
  - the registry now exposes canonical-name lookup, alias lookup, and
    surface-family filtering helpers
  - root smoke coverage now proves later compiler stages can ask the registry
    for stable entries without open-coding slice scans
- `1.6` `done` Add unit tests for registry lookup, duplicate-name rejection,
  and alias stability.
  Current state:
  - the crate now validates canonical-name uniqueness, alias uniqueness, and
    alias-vs-canonical collisions
  - root smoke coverage now proves the canonical registry passes validation and
    malformed registries fail with stable validation kinds

### Phase 2. Compiler Boundary Wiring

- `2.1` `done` Add parser-facing helper API for recognizing reserved
  intrinsic-root names without forcing parser semantic ownership.
  Current state:
  - `fol-intrinsics` now exposes `reserved_intrinsic_for_surface(...)` and
    `is_reserved_intrinsic_name_for_surface(...)` for parser-facing lookups
  - `fol-parser` now retains call-surface information on `FunctionCall` nodes:
    `Plain`, `DotIntrinsic`, and `KeywordIntrinsic`
  - leading-dot intrinsic calls no longer collapse indistinguishably into plain
    routine calls at the AST boundary
- `2.2` `done` Add typecheck-facing selection API so typecheck can resolve a
  parsed builtin spelling to one registry entry.
  Current state:
  - `fol-intrinsics` now exposes `select_intrinsic(...)`
  - the selection API distinguishes:
    - unknown intrinsic names
    - names that exist but are used on the wrong surface family
  - root smoke coverage and crate-local tests now lock the selection contract
- `2.3` `done` Add lowering-facing lowering-mode API.
  Current state:
  - `fol-intrinsics` now exposes lookup by `IntrinsicId`
  - lowering-facing helpers now expose canonical lowering-mode queries by
    intrinsic id and by lowering-mode family
  - root smoke coverage and crate-local tests now prove `echo` stays a runtime
    hook while comparison intrinsics stay in the general-IR family
- `2.4` `done` Add structured diagnostics helpers for:
  - unknown intrinsic
  - unsupported intrinsic
  - wrong arity
  - wrong type family
  - wrong version milestone
  Current state:
  - `fol-intrinsics` now exposes stable, intrinsic-aware message helpers for
    unknown names, wrong surface family, wrong arity, wrong type family, wrong
    version milestone, and unsupported-yet entries
  - the helper messages now render the intrinsic spelling itself
    (`.eq(...)`, `panic(...)`, etc.) instead of forcing each compiler stage to
    format its own variants
  - root smoke coverage and crate-local tests now lock those rendered messages
- `2.5` `done` Add tests proving parser/typecheck/lower all see the same
  canonical intrinsic identity.

### Phase 3. Comparison Family

- `3.1` `done` Add registry entries for:
  - `.eq`
  - `.nq`
  - `.lt`
  - `.gt`
  - `.ge`
  - `.le`
  Current state:
  - the canonical registry now locks the full comparison family with stable ids,
    arity, availability, surface, and lowering mode
  - root smoke coverage and crate-local tests now prove the comparison entries
    stay canonical and keep the `ne -> nq` alias stable
- `3.2` `done` Freeze accepted `V1` operand families for comparison
  intrinsics.
  Current state:
  - `fol-intrinsics` now exposes a stable comparison operand contract:
    - equality-style intrinsics accept two comparable scalar operands
    - ordered-style intrinsics accept two ordered scalar operands
  - the selection API now accepts parsed intrinsic names instead of requiring
    compile-time string literals, so the compiler can select registry entries
    directly from syntax
  - root smoke coverage and crate-local tests now lock both contracts
- `3.3` `done` Typecheck `.eq/.nq` through the registry instead of string
  branches.
  Current state:
  - resolver now skips name lookup for dot-root intrinsic calls so intrinsic
    semantics are no longer forced through ordinary routine resolution
  - typecheck now selects `.eq/.nq` from `fol-intrinsics`, records the selected
    intrinsic id on the typed syntax node, and applies the frozen comparable
    scalar contract
  - wrong-arity and wrong-family diagnostics now come from the intrinsic
    registry helpers instead of ad hoc string formatting
  - exact integration coverage now locks accepted equality pairs plus
    representative rejection cases
- `3.4` `done` Typecheck `.lt/.gt/.ge/.le` through the registry instead of
  string branches.
  Current state:
  - ordered comparison intrinsics now reuse the same registry-selected path as
    equality intrinsics
  - `.lt/.gt/.ge/.le` now enforce the frozen ordered-scalar contract in
    typecheck
  - exact integration coverage now locks successful `int`, `flt`, `chr`, and
    `str` comparisons plus representative rejected boolean and mixed-scalar
    pairs
- `3.5` `done` Lower comparison intrinsics through selected intrinsic forms.
- `3.6` `done` Add exact tests for scalar success and representative rejected
  families.
- `3.7` `done` Add CLI integration coverage for intrinsic comparison calls.

### Phase 4. Boolean Family

- `4.1` `done` Add `.not` registry entry.
- `4.2` `done` Typecheck `.not(bol)` through the registry.
- `4.3` `done` Reject non-boolean `.not(...)` with explicit intrinsic
  diagnostics.
- `4.4` `done` Lower `.not` through backend-neutral IR.
- `4.5` `done` Add parser/typecheck/lower/CLI coverage for `.not`.

### Phase 5. Length Family

- `5.1` `done` Add `.len` registry entry.
- `5.2` `done` Freeze `V1` accepted receiver families for `.len`:
  - strings
  - arrays
  - vectors
  - sequences
  - sets
  - maps
- `5.3` `done` Typecheck `.len(...)` with exact family diagnostics.
- `5.4` `done` Lower `.len(...)` to one explicit lowering form.
- `5.5` `done` Add exact tests for `.len` across supported and rejected
  families.
- `5.6` `done` Add CLI integration coverage for `.len`.

### Phase 6. Echo Family

- `6.1` `done` Add `.echo` registry entry.
- `6.2` `done` Freeze `V1` `.echo` semantics:
  - whether it is statement-only or expression-capable
  - whether it is effectful and yields `never` or another fixed shape
- `6.3` `pending` Typecheck `.echo(...)` through the registry.
- `6.4` `pending` Lower `.echo(...)` to one explicit backend-facing intrinsic
  form or runtime hook.
- `6.5` `pending` Add CLI coverage for `.echo(...)` in lowered output.

### Phase 7. Unsupported Intrinsic Inventory

- `7.1` `pending` Add explicit registry entries for deferred ownership/pointer
  intrinsics.
- `7.2` `pending` Add explicit version-aware diagnostics for:
  - `.de_alloc`
  - `.give_back`
  - `.address_of`
  - `.pointer_value`
- `7.3` `pending` Add exact tests proving those fail as intentional `V3`
  boundaries instead of falling through to unknown-name or generic unsupported
  errors.

### Phase 8. Shape And Query Expansion

- `8.1` `pending` Add placeholder and classification entries for:
  - `.cap`
  - `.is_empty`
  - `.low`
  - `.high`
  - `.min`
  - `.max`
  - `.clamp`
- `8.2` `pending` Decide which of those are real current-`V1` intrinsics versus
  “registry only for now”.
- `8.3` `pending` If any are accepted now, wire them through typecheck and lower
  with exact diagnostics and lowering forms.
- `8.4` `pending` Add tests proving accepted and deferred query intrinsics are
  both classified explicitly.

### Phase 9. Cast / Conversion Preparation

- `9.1` `pending` Add placeholder registry entries for `.cast` and `.as`.
- `9.2` `pending` Decide whether casts are:
  - a sub-milestone inside current `V1`, or
  - explicit unsupported until a later conversion plan
- `9.3` `pending` If still deferred, route all cast surfaces through the
  intrinsic registry and emit one stable diagnostic family.
- `9.4` `pending` Add tests proving cast diagnostics mention intrinsic intent,
  not generic operator failure.

### Phase 10. Existing Keyword Intrinsic Alignment

- `10.1` `pending` Decide whether `panic(...)` and `check(...)` are represented in
  the same registry despite different syntax families.
- `10.2` `pending` If yes, add them as keyword-surface intrinsic entries.
- `10.3` `pending` Move their typecheck/lower ownership onto the registry model.
- `10.4` `pending` Add regression tests proving keyword and dot intrinsics share
  one diagnostics/code path where appropriate.

### Phase 11. Arithmetic, Bitwise, And Overflow Roadmap Entries

- `11.1` `pending` Add registry entries and status classifications for:
  - arithmetic helper family
  - numeric helper family
  - bitwise family
  - checked/wrapping/saturating/overflowing families
- `11.2` `pending` Mark which of those are:
  - likely `V1.x`
  - `V2`
  - `V3`
  - `core/std`, not intrinsic
- `11.3` `pending` Add unit tests proving the registry exposes these categories
  consistently and without accidental duplicate names or aliases.

### Phase 12. Backend-Readiness Surface

- `12.1` `pending` Add backend-facing lowering metadata for each implemented
  intrinsic:
  - pure op
  - control effect
  - runtime hook
  - target helper
- `12.2` `pending` Add deterministic lowered rendering for intrinsic selections
  so backend bring-up can inspect them.
- `12.3` `pending` Add lowering verifier checks for impossible intrinsic/result
  combinations.

### Phase 13. Docs Closeout

- `13.1` `pending` Rewrite [`book/src/300_meta/100_buildin.md`](./book/src/300_meta/100_buildin.md)
  into an “intrinsics” page with the actual current subset.
- `13.2` `pending` Sync the relevant type/sugar/routine pages to the registry
  contract.
- `13.3` `pending` Update [`README.md`](./README.md) and [`PROGRESS.md`](./PROGRESS.md)
  to describe `fol-intrinsics` as shared compiler infrastructure.
- `13.4` `pending` Rewrite this file into a completion record only after at
  least one meaningful intrinsic family is real end to end.

## 20. Immediate Recommendation

Do **not** start with `.cast()` first.

Start with:

1. `fol-intrinsics` crate foundation
2. comparison family
3. `.not()`
4. `.len()`
5. `.echo()`

That gives the compiler:

- a real intrinsic registry
- real value for users
- better backend readiness
- a clean path for future additions without turning intrinsics into another
  compiler-wide patchwork
