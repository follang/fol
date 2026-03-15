# FOL Runtime Plan

Last updated: 2026-03-15

This file defines the next compiler milestone: create `fol-runtime`, the
runtime support crate that makes the current lowered `V1` IR executable through
the first backend.

This is the step before `fol-backend`.

The order is intentional:

1. define the runtime contract
2. implement the runtime crate
3. only then build the first backend against that contract

Without a runtime contract, backend work would hard-code data layout, intrinsic
behavior, and recoverable-error behavior directly into emitted code. That would
be fragile and hard to change once we later add a second backend such as LLVM.

`fol-runtime` is therefore the semantic support library for generated programs.
`fol-backend` will be the code generator that targets it.

## 0. What This Crate Is

`fol-runtime` is **not**:

- another front-end phase
- another semantic checker
- the standard library
- the future `core` library
- the package manager
- the backend itself

`fol-runtime` **is**:

- the support crate that generated code can depend on
- the runtime representation of current `V1` FOL values that are not trivial
  scalars
- the owner of `V1` recoverable-result representation
- the owner of backend-facing runtime hooks such as `.echo`
- the owner of basic container wrappers and helpers needed by emitted programs

## 1. Why This Must Exist Before The Backend

The lowered compiler already exposes enough structure for code generation:

- lowered packages / source units / entry candidates in
  [`fol-lower/src/model.rs`](./fol-lower/src/model.rs)
- lowered data types in
  [`fol-lower/src/types.rs`](./fol-lower/src/types.rs)
- lowered instructions and terminators in
  [`fol-lower/src/control.rs`](./fol-lower/src/control.rs)
- explicit recoverable ABI metadata in
  [`fol-lower/src/model.rs`](./fol-lower/src/model.rs)

But those lowered forms are still abstract.

The backend still needs concrete answers for questions like:

- what is `str` at runtime?
- what is `seq[T]` at runtime?
- what is the runtime representation of `T / E`?
- how does `.len(...)` work on arrays vs vectors vs sequences vs maps vs sets?
- how do we print values for `.echo(...)`?
- how are records and entries represented?
- how do `opt[T]` and `err[T]` shells behave at runtime?

If the first backend answers those ad hoc in emitted Rust without a runtime
crate, then:

- changing semantics later becomes much harder
- adding a second backend duplicates logic
- tests become backend-specific instead of contract-specific

So `fol-runtime` is the right first move.

## 2. Runtime Scope For Current `V1`

This runtime plan is only for the current `V1` language subset defined in
[`VERSIONS.md`](./VERSIONS.md).

That means the runtime must fully support what the lowered compiler already
supports in `V1`, and it must explicitly **not** pretend to support `V2` / `V3`
features yet.

### 2.1 Runtime Must Cover

- builtin scalars:
  - `int`
  - `flt`
  - `bol`
  - `chr`
  - `str`
  - `never`
- containers:
  - `arr[T, N]`
  - `vec[T]`
  - `seq[T]`
  - `set[...]`
  - `map[K, V]`
- shells:
  - `opt[T]`
  - `err[T]`
- user aggregates:
  - records
  - entries
- recoverable routine results:
  - `T / E`
- runtime-owned intrinsics:
  - `.echo(...)`
- backend helper semantics for:
  - `.len(...)`
  - `check(...)`
  - `panic(...)`

### 2.2 Runtime Must Not Cover Yet

- ownership and borrowing
- raw/shared/unique pointers
- channels / eventuals / coroutines / mutexes
- standards / blueprints / generics
- C ABI
- filesystem / networking / time / process / serialization
- package fetching / package store behavior
- full `core` / `std`

Those belong to later milestones.

## 3. The Core Runtime Design

The first backend will target Rust, so the initial runtime implementation will
be a Rust crate.

But the runtime contract should be written as **FOL semantic intent**, not as
“whatever Rust makes easiest”.

That means:

- runtime names should reflect FOL concepts
- runtime types should mirror lowered FOL categories
- backend codegen should depend on stable runtime APIs rather than inline random
  Rust snippets everywhere

The likely first crate layout is:

```text
fol-runtime/
  src/
    lib.rs
    abi.rs
    builtins.rs
    strings.rs
    containers/
      mod.rs
      array.rs
      vector.rs
      sequence.rs
      set.rs
      map.rs
    shell/
      mod.rs
      optional.rs
      error.rs
    entry.rs
    value.rs
    prelude.rs
```

The exact file split can change, but the responsibilities should stay similar.

## 4. The Runtime/Backend Boundary

This distinction must stay clear:

### `fol-runtime`

- owns runtime data structures
- owns runtime helper functions
- owns the semantic implementation of runtime hooks
- exposes stable APIs to generated code

### `fol-backend`

- consumes `LoweredWorkspace`
- emits Rust crate/module/code
- maps lowered instructions to runtime calls / native operations
- invokes Cargo / rustc in build mode

Rule of thumb:

- “What type/function exists at runtime?” -> `fol-runtime`
- “How do we emit Rust syntax for this lowered instruction?” -> `fol-backend`

## 5. The V1 Runtime Contract

The runtime should define one stable public contract for the first backend.

That contract should include:

- scalar aliases or wrappers where needed
- container types
- shell types
- recoverable-result type
- display/debug hooks
- intrinsic hook functions
- basic helpers for construction / lookup / length / tag inspection

## 6. Runtime Type Strategy

### 6.1 Scalars

These can likely map closely to Rust primitives in the first version:

- `int` -> `i64` first, unless the backend later needs target-width variation
- `flt` -> `f64`
- `bol` -> `bool`
- `chr` -> `char`
- `never` -> `!` where possible, otherwise backend-specific unreachable shape

But `str` should not be emitted as raw `String` everywhere without a runtime
decision.

### 6.2 `str`

`str` needs an explicit runtime choice.

For `V1`, the most pragmatic plan is:

- runtime-owned `FolStr`
- internally backed by Rust `String`
- cloneable
- equality/order comparisons exposed through standard Rust traits
- printable through runtime helpers
- cheap conversion from literals

Why not raw `String` directly?

- keeps backend-generated code consistent
- gives one place to change representation later
- avoids backend sprinkling string utility decisions everywhere

The runtime may then expose:

- `FolStr`
- `FolStr::from_literal(&str)`
- `FolStr::len_chars()` or equivalent if needed later
- `Display` / `Debug`

For `V1`, `.len(...)` on `str` is not currently part of the accepted intrinsic
surface, so string length APIs can stay internal or deferred.

### 6.3 Arrays

Lowered IR already distinguishes:

- `Array { element_type, size }`

For `V1`, fixed-size arrays can likely lower directly to Rust arrays where the
size is known.

But the runtime should still define helper constructors/access patterns so the
backend does not special-case every path.

Likely approach:

- no dedicated heap-owning runtime wrapper if Rust arrays work well enough
- small runtime helper functions for generic length / debug / cloning behavior
- backend emits `[T; N]` directly where possible

### 6.4 Vectors

`vec[T]` can likely map to:

- runtime-owned `FolVec<T>`
- internally backed by `Vec<T>`

This type should expose:

- construction from element vectors
- indexing
- length
- debug/display hooks if needed

### 6.5 Sequences

The book may describe richer linked-list semantics, but for current `V1`, the
runtime should prioritize executable coherence over perfect long-term purity.

The first runtime should likely represent `seq[T]` as:

- runtime-owned `FolSeq<T>`
- internally backed by `Vec<T>`

Why:

- much simpler first backend
- enough to validate current lowering and type semantics
- can later evolve behind the runtime API if needed

The important rule is:

- the runtime contract is authoritative
- backend code should not assume `seq` is literally `Vec<T>` forever

### 6.6 Sets

Current lowered type uses heterogeneous member type vectors in the type table,
but the currently supported `V1` set literals are still bounded enough that the
runtime should provide a stable wrapper.

Likely first version:

- runtime-owned `FolSet<T>` for homogeneous set use
- backend lowering may keep using typechecked element family guarantees to emit
  `FolSet<T>`

If heterogeneous set semantics remain part of the core language design, that
needs a separate later runtime-generalization plan. For current lowered `V1`,
the runtime only needs to satisfy what typecheck/lower already permit.

### 6.7 Maps

Maps should likely be:

- runtime-owned `FolMap<K, V>`
- internally backed by `BTreeMap<K, V>` first for stable ordering and easier
  deterministic tests

Expose:

- construction from entry pairs
- indexing / lookup
- length

### 6.8 Optional shells

`opt[T]` should map to a dedicated runtime-owned wrapper or directly to
`Option<T>`.

The practical first step is:

- runtime type alias or wrapper around `Option<T>`
- explicit helper APIs so backend code still uses runtime-owned names

That keeps `!` shell unwrapping and future optional helpers centralized.

### 6.9 Error shells

`err[T]` is distinct from recoverable routine results and must stay distinct.

The runtime should therefore expose a separate shell type:

- `FolErr<T>` or equivalent

This should likely be a thin wrapper around an owned payload or no payload for
bare `err`.

This matters because the book and compiler already distinguish:

- `err[T]` shell values
- routine error results `T / E`

The runtime must keep that distinction.

### 6.10 Records

For current `V1`, records are plain data layouts.

The backend may emit native Rust structs for concrete record declarations, but
the runtime still needs to define the *contract*:

- records are value layouts
- record field order in layout is stable and deterministic
- debug/echo representation is predictable

So:

- no single universal boxed record runtime type for `V1`
- backend emits concrete Rust structs per lowered record
- runtime provides shared traits/helpers for debug and maybe construction if
  needed

### 6.11 Entries

Entries should map naturally to Rust enums.

Again:

- backend emits concrete Rust enums per lowered entry declaration
- runtime defines shared expectations for debug/echo and maybe helper traits

### 6.12 Recoverable routine results

This is the most important runtime type.

Lowering already defines the current ABI in
[`fol-lower/src/model.rs`](./fol-lower/src/model.rs):

- tagged result object
- tag = `ok` / `err`
- success slot = `value`
- error slot = `error`

The runtime should own the actual representation, likely as:

- `FolRecover<T, E>`

Possible first internal shape:

```rust
pub enum FolRecover<T, E> {
    Ok(T),
    Err(E),
}
```

or a struct-plus-tag version if exact lowered ABI mirroring is more useful.

For `V1`, the important thing is:

- success and failure are explicit
- propagation is easy to encode
- `check(...)`, `unwrap`, fallback, and debug are easy to implement

## 7. Runtime Semantics For Intrinsics

Not every intrinsic belongs in runtime.

Split them carefully:

### 7.1 Pure compiler intrinsics

These may lower directly to Rust operators and not need runtime helpers:

- `.eq`
- `.nq`
- `.lt`
- `.gt`
- `.ge`
- `.le`
- `.not`

### 7.2 Backend-helper intrinsics

These may use runtime helpers depending on target mapping:

- `.len`
- `check`

### 7.3 Runtime-hook intrinsics

These should live explicitly in the runtime:

- `.echo`
- maybe panic formatting helpers even if final panic syntax is backend-native

## 8. `.echo(...)`

The first runtime-owned hook that must exist is `.echo(...)`.

The runtime should define a single public function the backend can call, such
as:

- `fol_runtime::echo(value)`

It should work for current `V1` value families:

- integers
- floats
- bools
- chars
- strings
- arrays/vectors/sequences
- sets/maps
- records
- entries
- optional/error shells

To make this manageable, the runtime should define a shared formatting trait.

Possible direction:

- `FolDisplay`
- blanket impls for runtime containers
- backend-generated impls or derived formatting for emitted record/entry types

## 9. `.len(...)`

The current accepted query intrinsic is only `.len(...)`.

The runtime contract must define what length means for each accepted family.

For `V1`, that should include:

- arrays -> element count
- vectors -> element count
- sequences -> element count
- sets -> member count
- maps -> entry count

Current typecheck already gates which families may use `.len(...)`, so runtime
does not need to invent new semantics.

## 10. `check(...)` And Recoverable Results

The runtime should expose a stable helper for checking recoverable results.

Likely:

- `fol_runtime::recover::is_err(&FolRecover<T, E>) -> bool`

The backend may inline or pattern-match directly in generated Rust, but the
runtime should still own the semantic helper.

That keeps the backend simpler and the recoverable contract centralized.

## 11. `panic`

`panic` is keyword-owned semantically, but runtime should probably still own
formatting/helper glue.

The split should be:

- backend decides how to emit a panic site
- runtime owns helper formatting if needed

For the Rust backend, plain `panic!` is likely enough first, but the runtime
may later provide a `fol_panic(...)` wrapper for consistency.

## 12. Lowered Instruction Coverage

The runtime plan must account for every lowered `V1` instruction family that
depends on runtime representations.

From [`fol-lower/src/control.rs`](./fol-lower/src/control.rs), runtime-sensitive
coverage includes:

- `Call`
- `RuntimeHook`
- `LengthOf`
- `ConstructRecord`
- `ConstructEntry`
- `ConstructLinear`
- `ConstructSet`
- `ConstructMap`
- `ConstructOptional`
- `ConstructError`
- `FieldAccess`
- `IndexAccess`
- `CheckRecoverable`
- `UnwrapRecoverable`
- `ExtractRecoverableError`
- `UnwrapShell`
- `Report`
- `Panic`

The runtime does not need to “implement” field access or calls directly, but it
must make them meaningful through the target types it defines.

## 13. Recoverable ABI Contract

This is one of the most important parts of the plan.

The lowered workspace already exposes:

- success tag
- error tag
- success slot
- error slot

The runtime should freeze the first real `V1` recoverable contract.

That contract should answer:

- what Rust type carries `T / E`
- how propagation is encoded
- how `report` constructs failure
- how `check(...)` inspects failure
- how `expr || fallback` is expressed
- how backend-generated `main` handles top-level failures

For Rust, the likely first mapping is:

- use runtime `FolRecover<T, E>` backed by `Result<T, E>`
- backend emits `match` or `?`-like patterns explicitly
- top-level entrypoint turns `Err(...)` into a printed error + non-zero exit

But the plan should keep this contract runtime-owned, not backend-ad-hoc.

## 14. Records And Entries In The Runtime Contract

The runtime should not try to erase all user types into one universal dynamic
box in `V1`.

That would overcomplicate the first backend.

Instead:

- backend emits native Rust structs/enums for concrete lowered record/entry
  declarations
- runtime provides:
  - formatting traits
  - helper derivation contract
  - maybe construction/inspection helper traits if useful later

So the runtime must document:

- concrete generated types are allowed
- generated types still conform to runtime formatting/recoverable/container
  expectations

## 15. Namespace And Package Awareness

The runtime itself does not manage package loading.

But it must be designed so generated code from many packages/namespaces can all
link against it cleanly.

That means:

- no global mutable runtime state as a default assumption
- generic container/runtime APIs should be namespace-agnostic
- emitted package modules should all be able to `use fol_runtime::...`

## 16. What Counts As “Fully Handle V1”

For this milestone, “fully handle V1” means:

- every current lowered `V1` type family has a runtime representation or a
  documented direct-target mapping
- every current lowered `V1` intrinsic/runtime hook has an implementation path
- every current lowered `V1` recoverable-result flow has a stable contract
- one future backend can compile real `V1` programs through this runtime without
  inventing missing semantics on the fly

## 17. Testing Strategy

This crate needs heavy tests because later backend correctness will depend on
it.

### 17.1 Unit tests

Per runtime type/helper:

- strings
- arrays/helpers
- vectors
- sequences
- sets
- maps
- optional shells
- error shells
- recoverable results
- `.echo`
- `.len`
- recoverable inspection helpers

### 17.2 Contract tests

Runtime-specific invariants:

- deterministic formatting for debug/echo
- deterministic map/set iteration policy if exposed
- recoverable-result propagation semantics
- optional/error shell unwrap behavior
- empty vs non-empty container behavior

### 17.3 Integration tests with lowered fixtures later

Even before the backend exists, add tests that mirror representative lowered
`V1` families so the backend has a target contract to hit.

### 17.4 Future backend smoke tests

Once `fol-backend` starts, generated Rust integration tests should exercise:

- one-file programs
- multi-file same-package programs
- sub-namespace programs
- `loc` / `std` / `pkg` package graphs
- recoverable errors
- containers
- records / entries
- intrinsics

## 18. Non-goals For This Runtime Milestone

Do not expand scope into:

- implementing `core` or `std`
- inventing a GC
- inventing pointer ownership machinery
- implementing async runtime
- implementing C ABI
- supporting every future intrinsic
- introducing dynamic universal boxed values for everything

The point is to make current lowered `V1` executable, not to solve the whole
future language.

## 19. Proposed Crate API

The public API should be small and explicit.

Likely exports:

- `FolStr`
- `FolVec<T>`
- `FolSeq<T>`
- `FolSet<T>`
- `FolMap<K, V>`
- `FolOption<T>`
- `FolError<T>`
- `FolRecover<T, E>`
- `echo(...)`
- length helpers / traits
- recoverable helpers
- formatting traits / prelude

## 20. Phase Breakdown

The implementation should land in many small slices.

### Phase 0: Scope And Crate Foundation

#### 0.1 done
- add new workspace crate `fol-runtime`
- wire it into root workspace manifests
- add crate smoke test

#### 0.2 done
- add public crate shell and top-level module layout
- define the first prelude surface

#### 0.3 done
- add runtime error type(s) for invariant violations inside the runtime crate
- add test helpers

#### 0.4 done
- document the runtime/backend boundary in crate docs
- freeze current `V1` scope in tests/doc comments

### Phase 1: Scalar And ABI Foundation

#### 1.1 done
- define scalar aliases/wrappers policy
- freeze `int`, `flt`, `bol`, `chr`, `never` strategy

#### 1.2 done
- add `FolStr`
- implement literal conversion and equality/order/display behavior

#### 1.3 done
- add recoverable runtime type `FolRecover<T, E>`
- freeze `Ok/Err` mapping and helpers

#### 1.4 done
- add helper functions for `check(...)`-style error inspection

#### 1.5 done
- add top-level recoverable ABI tests
- prove success/failure/inspection semantics

### Phase 2: Shell Types

#### 2.1 done
- add `FolOption<T>` or runtime alias/wrapper policy for `opt[T]`

#### 2.2 done
- add `FolError<T>` / bare error-shell representation for `err[T]`

#### 2.3 done
- add shell unwrap helpers for optional and error shells

#### 2.4 done
- add shell formatting coverage

#### 2.5 done
- add tests proving shell values stay distinct from recoverable routine results

### Phase 3: Container Foundation

#### 3.1 done
- freeze runtime strategy for arrays
- document/direct-test native array support where used

#### 3.2 done
- add `FolVec<T>`

#### 3.3 done
- add `FolSeq<T>`

#### 3.4 done
- add `FolSet<T>`

#### 3.5 done
- add `FolMap<K, V>`

#### 3.6 done
- add deterministic constructors from element vectors/pairs

#### 3.7 done
- add indexing helpers where needed

#### 3.8 done
- add length helpers for all supported current `V1` families

#### 3.9 done
- add formatting tests for all container families

### Phase 4: Runtime Hooks And Intrinsic Support

#### 4.1 done
- add runtime-facing formatting trait(s) for `.echo(...)`

#### 4.2 done
- implement `echo(...)` for builtin scalars and strings

#### 4.3 done
- implement `echo(...)` for containers

#### 4.4 done
- implement `echo(...)` for shell values

#### 4.5 done
- add hook tests for nested values

#### 4.6 done
- freeze `.len(...)` helper contract against current accepted families

### Phase 5: Aggregate Support Contract

#### 5.1 done
- define a runtime trait contract for generated record types

#### 5.2 done
- define a runtime trait contract for generated entry types

#### 5.3 done
- provide formatter/debug hooks backend-generated record/entry types can use

#### 5.4 done
- add crate-level examples proving how backend-generated structs/enums should
  integrate with runtime traits

### Phase 6: Entry And Top-level Execution Contract

#### 6.1 done
- define how generated Rust `main` should interpret `FolRecover<T, E>`

#### 6.2 done
- add helper(s) for converting recoverable top-level failures into printable
  process outcomes

#### 6.3 done
- define minimal exit-code contract for current `V1`

#### 6.4 done
- add tests for top-level success/failure formatting behavior

### Phase 7: Runtime Verification And Hardening

#### 7.1 done
- add invariant tests for empty containers

#### 7.2 done
- add invariant tests for nested container formatting

#### 7.3 done
- add invariant tests for recoverable + shell interactions

#### 7.4 done
- add deterministic behavior tests for map/set ordering policy

#### 7.5 done
- add runtime doctests/examples for backend authorship

### Phase 8: Backend-facing Documentation

#### 8.1 done
- document exactly how the backend should map lowered builtins to runtime or
  native Rust forms

#### 8.2 done
- document exactly which lowered instructions require runtime support

#### 8.3 done
- document name/import expectations for generated crates

#### 8.4 done
- add a backend-integration guide inside crate docs

### Phase 9: Repo Docs Sync

#### 9.1 done
- update `README.md`
- update `PROGRESS.md`
- record `fol-runtime` as the next implemented support layer

#### 9.2 done
- update the book where runtime/builtin/container/error wording must acknowledge
  the new runtime contract

#### 9.3
- rewrite `PLAN.md` into a completion record only after runtime tests are green

## 21. Definition Of Done

`fol-runtime` is done for current `V1` only when all of the following are true:

- the crate exists and is wired into the workspace
- all current lowered `V1` runtime-sensitive types have a stable representation
  or explicit direct-target mapping
- `FolRecover<T, E>` and shell types are stable and tested
- `.echo(...)` and `.len(...)` have stable runtime behavior for the current
  accepted `V1` families
- record/entry integration hooks are defined for generated code
- top-level failure behavior is defined
- unit tests cover all runtime families
- repo/docs are synced

At that point, `fol-backend` can start against a stable support library instead
of inventing runtime semantics during code generation.
