# Full V1 Typecheck Completion Plan

Status: pending

Last reopened: 2026-03-14

## 1. Why This Plan Exists

The previous `fol-typecheck` milestone was substantial, but it was not the full
`V1` typechecker yet.

That earlier plan proved:

- the crate exists
- local declaration and expression typing works across a large `V1` subset
- `V2` and `V3` surfaces fail explicitly
- CLI integration exists

What it did **not** prove was the full `V1` semantic contract across the real
package/import graph.

This plan reopens typechecking specifically to finish `V1`, not to start `V2`
or `V3`.

## 2. Confirmed Current Gaps

These are confirmed from the current repository head, active tests, and direct
CLI repros.

### 2.1 Imported Symbols Are Not Fully Typed

Using imported exported symbols from `loc`, `std`, or `pkg` still fails under
typechecking with:

- `TypecheckInvalidInput: resolved symbol X does not have a lowered type yet`

Confirmed current repro shape:

- imported exported value used in `return`
- imported exported routine called from the entry package

This means the current typechecker is still local-package-centric rather than
package-graph-complete.

### 2.2 Typechecker Only Walks Entry Syntax

`fol-typecheck` currently lowers declaration signatures and expression bodies by
iterating `resolved.syntax().source_units`.

That is only the entry package syntax surface.

Imported package declarations that were loaded by `fol-package` and resolver are
not lowered into typed declaration facts before mounted imported symbols are
used.

### 2.3 Mounted Imported Symbols Lose Semantic Ownership

Resolver currently mounts imported exported symbols into the entry program as
cloned `ResolvedSymbol`s.

That is enough for name resolution, but not enough for full typechecking,
because:

- the mounted symbol does not carry a typed declaration fact
- the mounted symbol is not tied back strongly enough to the original foreign
  symbol/type owner for semantic expansion

### 2.4 Imported Method Lookup Is Also Entry-Package-Centric

Method lookup in the typechecker still scans only the entry package syntax.

So even after imported value/routine typing is fixed, imported receiver methods
would still remain incomplete unless method lookup is moved onto package-graph
facts instead of entry syntax scanning.

### 2.5 Optional/Nil/Unwrap Semantics Are Still Incomplete For V1

The book and current `VERSIONS.md` put optionals in the core language story.
But current head still rejects:

- `nil` literals
- postfix unwrap

with explicit `not part of the V1` diagnostics.

That means the current `V1` boundary is still inconsistent with the current book
and version split.

### 2.6 Named Aggregate Expansion Still Needs Completion

The current typechecker handles many record and entry cases, but named-type
expansion is still not fully complete across all expected-type contexts.

The important cases that still need explicit locking are:

- binding initializers against named record/entry types
- imported named record/entry types
- alias-wrapped aggregate expected types
- mounted/imported aggregate method and field surfaces

### 2.7 Current Typecheck Coverage Misses Real Import Cases

Current typecheck-focused tests are overwhelmingly local and single-package.

There is little or no direct typecheck coverage for:

- imported `loc` values/routines/types
- imported `std` values/routines/types
- imported `pkg` values/routines/types
- transitive `pkg` imports through build-defined package graphs
- imported methods and imported aggregate surfaces

### 2.8 Some User-Triggerable Failures Still Look Internal

The current typechecker still exposes user-triggerable `InvalidInput` and
fallback-style failures in places where a full `V1` typechecker should either:

- succeed
- reject with an explicit `Unsupported`
- reject with a real incompatibility/input diagnostic tied to the user surface

The imported-symbol failure is the clearest example, but it should not be the
only one audited in this pass.

### 2.9 Exact Origins Need A Final Audit On Reopened Surfaces

The diagnostics infrastructure is strong now, but several reopened typecheck
surfaces still need exact source-origin confirmation:

- imported symbol usage failures
- nil/unwrap failures
- named aggregate field/type mismatch failures
- mounted/imported method/field/call failures

## 3. Scope Of This Reopened Plan

This plan is only for completing the `V1` typechecker.

It **does** include:

- full typechecking over the current import/package graph
- typechecking of imported exported values/routines/types
- imported method lookup
- imported named aggregate usage
- optional `nil` and postfix unwrap semantics if they remain part of `V1`
- exact diagnostics for all reopened surfaces

It does **not** include:

- generics
- standards / protocols / blueprints / contract conformance
- ownership / borrowing / pointers
- async / await / coroutines / channels
- C ABI
- backend / lowering / code generation

Those remain outside this plan and should continue to fail explicitly or stay
for later milestones according to [VERSIONS.md](./VERSIONS.md).

## 4. Full V1 Definition Of Done

This plan is complete only when all of the following are true:

- the typechecker works across local code and imported package graphs
- `loc`, `std`, and installed `pkg` imports can be used in typechecked programs,
  not merely resolved
- imported exported values, routines, aliases, records, and entries can
  participate in `V1` typing where their local equivalents already can
- imported receiver methods resolve and typecheck correctly
- named aggregate expected-type expansion works in bindings, returns, calls, and
  imported contexts
- `nil` and postfix unwrap have a real `V1` semantic contract, not a placeholder
  unsupported boundary
- current user-triggerable `InvalidInput/Internal` fallback failures on valid
  parsed+resolved `V1` programs are removed
- CLI tests exercise real successful and failing typechecked imports, not just
  root-flag acceptance
- exact diagnostic locations survive to CLI JSON for the reopened surfaces
- `README.md`, `PROGRESS.md`, and this plan agree on the final `V1`
  typechecking boundary

## 5. Architectural Direction

The current `ResolvedProgram` value is not enough by itself to finish `V1`.

The clean direction is:

- keep `fol-package` as the source of prepared package roots
- keep resolver as the owner of package loading and mounted import resolution
- expose a resolver-owned package graph/workspace to later semantic stages
- let the typechecker consume that graph instead of pretending the entry package
  syntax is the whole semantic world

### 5.1 Recommended New Semantic Handoff

Introduce a resolver-owned graph/workspace model. The exact type name is open,
but the role should be something like:

- `ResolvedWorkspace`
- or `ResolvedPackageGraph`

It should contain:

- the entry package identity
- the entry resolved program
- every loaded package identity
- every loaded package prepared controls
- every loaded package resolved program

### 5.2 Recommended Mounted-Symbol Provenance

Mounted imported symbols need a stable semantic backlink to their original
foreign declaration owner.

That provenance should include enough to recover:

- which loaded package the symbol came from
- which original foreign symbol it came from

Without that, typechecker cannot reliably ask:

- what is the actual declared type of this imported symbol?
- what method set does this imported receiver type have?
- what record/entry structure does this imported declared type expand to?

### 5.3 Typechecker Target Shape

`fol-typecheck` should eventually consume the resolver workspace/package graph,
not only a single `ResolvedProgram`.

Compatibility shims may remain for local or unit-test use, but the real CLI
path should use the full graph-aware path.

## 6. Reopened V1 Policy Decisions

These decisions must be frozen early in this plan.

### 6.1 Promote Optionals To Real V1 Semantics

Given the current book and `VERSIONS.md`, the reopened plan should treat these
as part of `V1`:

- optional shell typing
- `nil`
- postfix unwrap

If any narrower contract is desired, it must be written explicitly and tested.

### 6.2 Keep Later Surfaces Out

These remain outside this plan unless deliberately reclassified:

- function type literals and higher-order function typing
- union / any / none semantics
- advanced matching / rolling / select semantics
- optional chaining beyond the currently parsed postfix unwrap surface
- `check(...)` and similar helper-surface semantics unless their ownership is
  explicitly frozen as part of the core language rather than std/builtin tooling

If they are not pulled into `V1`, they must stay explicitly unsupported.

## 7. Implementation Phases

Each feature/fix slice must land with its test in the same commit.

### Phase 0: Boundary Reset

Status: pending

#### 0.1

Status: done

- Rewrite this plan from the previous optimistic completion record into a real
  reopened `full V1` plan with the confirmed missing surfaces.

#### 0.2

Status: done

- Freeze the reopened `V1` boundary in code/docs terms:
- imported graph typing is required
- `nil` is required
- postfix unwrap is required
- later-version surfaces remain later-version surfaces

#### 0.3

Status: done

- Add focused failing tests that lock the current real blockers before fixing
  them:
- imported `loc` exported value in `return`
- imported `std` exported value in `return`
- imported `pkg` exported value in `return`
- imported exported routine call
- `nil` in an optional-typed binding
- postfix unwrap of an optional-typed value

### Phase 1: Resolver Handoff For Semantic Consumers

Status: pending

#### 1.1

Status: done

- Add a resolver-owned workspace/package-graph model that exposes the entry
  package plus loaded packages to later semantic phases.

#### 1.2

Status: done

- Preserve mounted imported symbol provenance so later stages can recover the
  foreign package identity and original foreign symbol id.

#### 1.3

Status: done

- Add tests that direct, repeated, and transitive package loads are all retained
  in the semantic handoff model without changing resolver name-resolution
  behavior.

#### 1.4

Status: done

- Keep the old single-package resolver API available only as a compatibility
  shim while the CLI and typechecker move to the graph-aware path.

### Phase 2: Graph-Aware Typecheck Foundation

Status: pending

#### 2.1

Status: done

- Add `TypedPackage` / `TypedWorkspace` result models so typechecking can track
  per-package semantic facts instead of only one entry package shell.

#### 2.2

Status: done

- Typecheck each loaded package once and cache typed package facts by package
  identity.

#### 2.3

Status: done

- Teach the typechecker to answer mounted imported symbol types by following
  mounted-symbol provenance back into typed foreign package facts.

#### 2.4

Status: done

- Add tests that the new graph-aware typechecker still preserves the current
  successful local-only behavior.

### Phase 3: Imported Declaration Facts

Status: pending

#### 3.1

Status: done

- Lower declaration signatures for loaded packages as well as the entry package.

#### 3.2

Status: done

- Make imported exported values expose real declared types during expression
  typing.

#### 3.3

Status: done

- Make imported exported routines expose real callable signatures during
  expression typing.

#### 3.4

Status: done

- Make imported exported aliases, records, and entries expose real declared
  semantic type facts, not opaque unresolved placeholders.

#### 3.5

Status: done

- Lock direct imported declaration typing tests for `loc`, `std`, and `pkg`.

#### 3.6

Status: done

- Lock transitive imported declaration typing tests for `pkg`.

### Phase 4: Imported Expression And Method Parity

Status: pending

#### 4.1

Status: done

- Make plain imported value references typecheck in bindings, returns, and call
  arguments.

#### 4.2

Status: done

- Make plain imported routine calls typecheck with argument and return checking.

#### 4.3

Status: done

- Make qualified imported value/routine references typecheck with the same
  parity as local and plain imported references.

#### 4.4

Status: done

- Move method lookup onto typed package facts instead of entry-package syntax
  scanning so imported receiver methods can work.

#### 4.5

Status: done

- Lock imported method-call tests across direct and qualified import roots.

### Phase 5: Declared-Type Expansion Completion

Status: pending

#### 5.1

Status: done

- Complete apparent-type expansion for named declared types so aggregate
  operations do not stop at opaque declared shells when the underlying type is
  structurally known and part of `V1`.

#### 5.2

Status: pending

- Ensure record initializers work against named expected types in all already-
  intended `V1` contexts:
- binding initializers
- final routine body expressions
- call arguments where expected type is known
- imported named record types

#### 5.3

Status: pending

- Ensure entry/variant value surfaces work against named expected types in the
  same set of contexts.

#### 5.4

Status: pending

- Lock local and imported aggregate tests:
- named record binding initialization
- imported record binding initialization
- imported record field access
- imported entry value surfaces

### Phase 6: Optional And Error Shell V1 Completion

Status: pending

#### 6.1

Status: pending

- Freeze the exact `nil` contract for `V1`, including where `nil` is accepted
  and what expected-type context is required.

#### 6.2

Status: pending

- Implement `nil` typing for the accepted `V1` optional/error shell contexts.

#### 6.3

Status: pending

- Implement postfix unwrap typing for the accepted `V1` shell contexts.

#### 6.4

Status: pending

- Lock tests for:
- `nil` in bindings
- `nil` in returns
- `nil` in call arguments where expected type is known
- postfix unwrap in bindings
- postfix unwrap in returns
- mismatched unwrap targets

#### 6.5

Status: pending

- Sync `README.md`, `PROGRESS.md`, and relevant book/type pages to the final
  `nil`/unwrap `V1` contract once frozen.

### Phase 7: Invalid/Internal Hardening

Status: pending

#### 7.1

Status: pending

- Eliminate the current user-triggerable imported-symbol fallback:
- `resolved symbol X does not have a lowered type yet`

#### 7.2

Status: pending

- Audit current `TypecheckInvalidInput` and `TypecheckInternal` fallback paths
  that can still be reached from valid parsed+resolved `V1` programs.

#### 7.3

Status: pending

- Replace audited fallback failures with either:
- success
- explicit `Unsupported`
- explicit incompatibility/input diagnostics tied to the real user surface

#### 7.4

Status: pending

- Add regression tests that a valid parsed+resolved `V1` program no longer
  trips internal-looking typecheck fallbacks in the reopened areas.

### Phase 8: Exact Diagnostics For Reopened Surfaces

Status: pending

#### 8.1

Status: pending

- Ensure imported-value, imported-call, imported-method, `nil`, and unwrap
  diagnostics all keep exact primary locations.

#### 8.2

Status: pending

- Ensure nested aggregate/type mismatch diagnostics keep the real field/value
  site instead of generic fallback origins.

#### 8.3

Status: pending

- Add CLI JSON integration tests for the reopened surfaces so the exact
  structured diagnostic shape remains locked end to end.

### Phase 9: CLI And End-To-End V1 Parity

Status: pending

#### 9.1

Status: pending

- Replace the current root-flag-only `loc/std/pkg` CLI acceptance fixtures with
  real typechecked imported-symbol success cases.

#### 9.2

Status: pending

- Add failing CLI integration tests for imported-symbol type errors and imported
  aggregate/optional-shell errors.

#### 9.3

Status: pending

- Keep direct-folder entry packages, `std`, and installed `pkg` paths all
  working through the full:
- `fol-package -> fol-resolver -> fol-typecheck`
  chain.

### Phase 10: Closeout

Status: pending

#### 10.1

Status: pending

- Update `README.md`, `PROGRESS.md`, and this plan to describe `fol-typecheck`
  as the full `V1` typechecker only after the reopened import/optional work is
  actually complete.

#### 10.2

Status: pending

- Rewrite this file into a completion record only once the full `V1` definition
  of done in section 4 is actually true.

## 8. Next Boundary After This Plan

When this plan is complete, the next work should still stay inside the `V1`
compiler chain:

- later semantic/lowering stages
- backend/runtime/codegen direction

Only after that should the project return to:

- `V2` language semantics
- `V3` systems and interop
