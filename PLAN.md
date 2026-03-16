# FOL Real-Program Hardening Plan

Last updated: 2026-03-16

This file defines the next hardening milestone on branch `feature/hardening`.

The goal is no longer to add a new compiler stage. The goal is to prove that
the current `V1` compiler can survive real programs, real folder layouts, and
real package graphs repeatedly and deterministically.

That means:

- write real FOL applications
- place them in the test tree as durable fixtures
- run them through the full chain:
  - package loading
  - resolver
  - typecheck
  - lowering
  - backend emission
  - backend build
  - binary execution where meaningful
- use those fixtures to expose the remaining gaps in the claimed `V1` surface

This plan is intentionally application-first.
It is not another unit-only milestone.

## 0. Hardening Objective

The compiler already has:

- `fol-stream`
- `fol-lexer`
- `fol-parser`
- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-lower`
- `fol-runtime`
- `fol-backend`

What it still needs is confidence.

That confidence should come from a curated set of small but real FOL programs
that exercise the current `V1` language surface as a user would actually write
it:

- multiple files
- folder namespaces
- sibling package imports
- installed `pkg`
- `std`
- records
- entries
- aliases
- methods
- intrinsics
- recoverable errors
- containers
- loops
- `when`
- declarations spread across files and folders

The test tree should stop relying only on narrow one-surface snippets and start
keeping real programs alive as regression fixtures.

## 1. Hardening Rule

Every fixture in this plan must follow all of these rules:

- it lives under the repo test tree
- it is a real folder program, not just one expression
- it compiles through the real CLI/backend path
- it is named after the behavior it proves
- it is small enough to debug quickly
- it is stable enough to become a permanent regression anchor

When possible, the fixture should also:

- run the produced binary
- assert stdout/stderr
- assert exit code

## 2. Fixture Tree Direction

Add a dedicated tree for application-style backend fixtures. Recommended shape:

- `test/apps/`
- `test/apps/fixtures/`
- `test/apps/fixtures/<fixture-name>/...`
- `test/apps/test_apps.rs`

The fixture runner should keep one clear distinction:

- fixture source layout belongs under `test/apps/fixtures`
- Rust assertions and harness helpers belong in `test/apps/test_apps.rs`

Do not mix these into parser-only or resolver-only fixture trees.

## 3. Coverage Target

Create at least `20` fixture folders.

That is a floor, not a ceiling.

The first complete hardening batch should aim for `24` to `28` fixtures so the
coverage is not one-fragile-example-per-topic.

## 4. Required Fixture Families

The fixture set must cover all of these families.

### 4.1 Single-Package Basics

1. `scalar_entry`
- one-file scalar return
- prove the backend can still build the smallest program

2. `bindings_and_calls`
- several local bindings
- plain routine calls
- nested routine use

3. `control_when`
- statement and value-producing `when`
- branch convergence and branch returns

4. `control_loop_break`
- `loop`
- `break`
- loop condition typing and runtime execution

### 4.2 Multi-File Same-Package Layout

5. `same_folder_shared_scope`
- two files in one folder
- declarations visible across files without `use`
- default package visibility

6. `same_folder_hidden_visibility`
- one file defines `hid`
- sibling file fails to use it
- fixture should be a compile-failure app test, not just a resolver unit test

7. `subfolder_namespace`
- root plus subfolder namespace
- namespace-qualified routine and type access

8. `deep_namespace_chain`
- nested folders two or three levels deep
- qualified access through that namespace chain

### 4.3 Local Package Imports

9. `loc_plain_values`
- sibling folder imported with `use ...: loc`
- imported exported vars and routines

10. `loc_types_and_records`
- imported record type plus imported constructor/helper routine
- cross-package record typing

11. `loc_methods`
- imported type with receiver-qualified routine
- method syntax from another local package
- prove procedural-not-OOP receiver lowering still works

12. `loc_recoverable_calls`
- imported errorful routine
- propagation, `check(...)`, and `|| fallback`

### 4.4 Standard And Installed Packages

13. `std_basic_import`
- explicit `--std-root`
- small std package with exported value/routine

14. `std_namespace_import`
- std package with sub-namespace folder
- qualified access through std-mounted namespace

15. `pkg_basic_import`
- installed package root with `package.yaml` + `build.fol`
- exported root load through `pkg`

16. `pkg_transitive_import`
- installed package with dependency on another installed package
- prove transitive loading still works through the backend path

17. `mixed_loc_std_pkg`
- one program that uses all three:
  - `loc`
  - `std`
  - `pkg`
- this must remain one of the headline hardening fixtures

### 4.5 Data Modeling

18. `record_flow`
- record declaration
- nested record construction
- field access
- field transport through calls and returns

19. `entry_flow`
- entry declaration
- entry value construction
- branch on entry-compatible values where current `V1` allows it

20. `alias_flow`
- aliases for records, shells, and routine-facing types
- prove aliases survive package/type/backend flow

21. `method_flow`
- receiver-qualified routines on a `typ`
- method syntax and plain-function lowering equivalence

### 4.6 Containers And Queries

22. `container_linear`
- arrays, vectors, or sequences from the current supported subset
- indexing
- `.len(...)`

23. `container_map_set`
- map and set construction
- stable runtime behavior that the backend actually supports today

24. `container_cross_package`
- container values created in one package and consumed in another

### 4.7 Intrinsics

25. `intrinsics_comparison`
- `.eq`, `.nq`, `.lt`, `.gt`, `.ge`, `.le`

26. `intrinsics_not_len_echo`
- `.not`
- `.len`
- `.echo`

27. `intrinsics_panic_check`
- `check(...)`
- `panic(...)`
- where panic is used, assert failure-path behavior intentionally

### 4.8 Recoverable Errors

28. `recoverable_propagation`
- routine `Result / Error`
- plain propagation

29. `recoverable_check`
- `check(...)` over real errorful calls

30. `recoverable_fallback`
- `expr || fallback`
- include success and failure branches

31. `recoverable_package_boundary`
- errorful routine imported from another package
- handled by caller in a different package

### 4.9 Shells

32. `shell_optional`
- `opt[...]`
- `nil`
- postfix `!`

33. `shell_error`
- `err[...]`
- unwrap behavior distinct from routine recoverable calls

34. `shell_vs_recoverable_boundary`
- one fixture whose whole point is proving that:
  - shells use `!`
  - routine recoverable calls use propagation / `check(...)` / `||`

## 5. Required Negative Fixtures

Not every application fixture should succeed.

Keep a smaller failure set under the same tree for real whole-program failures.

Minimum required failing-app families:

1. `fail_hidden_cross_file`
- sibling file reaches `hid`

2. `fail_loc_targets_formal_pkg_root`
- `loc` pointed at a folder with `build.fol`

3. `fail_type_mismatch_real_app`
- ordinary app-level type mismatch

4. `fail_recoverable_plain_context`
- errorful call used without propagation/handling

5. `fail_shell_unwrap_boundary`
- `!` applied to a recoverable routine call instead of a shell

6. `fail_deferred_intrinsic`
- use a reserved/deferred intrinsic and assert explicit milestone guidance

These should still be folder-style app fixtures, not one-line unit snippets.

## 6. Harness Rules

The Rust test harness should provide:

- fixture root discovery
- per-fixture temporary build roots
- helper to run `fol` on a folder entry
- helper to pass `--std-root`
- helper to pass `--package-store-root`
- helper to run the produced binary when a build succeeds
- helper to assert:
  - compile success
  - compile failure
  - stdout
  - stderr
  - exit code

The harness should also support two fixture modes:

- `compile_only`
- `compile_and_run`

And two expectation modes:

- `expect_success`
- `expect_failure`

## 7. Program Style Rules

To keep the fixtures useful:

- prefer explicit names over tiny cryptic examples
- keep each app around `10` to `80` lines per file
- use multiple files when the feature is about layout or namespace
- avoid adding unsupported `V2`/`V3` surfaces just to be clever
- if a fixture hits a real compiler gap, keep the smallest reproducer and add it
  instead of quietly rewriting it away

This matters because the fixture tree should become the permanent regression
layer for the executable `V1` surface.

## 8. Execution Phases

### Phase 0: Harness Foundation

#### 0.1 done
- add `test/apps/` tree and app-fixture runner entrypoint

#### 0.2 done
- add helpers for:
  - compile-only folder fixtures
  - compile-and-run folder fixtures
  - compile-failure folder fixtures

#### 0.3 done
- add helpers for explicit `std` and `pkg` roots inside fixture sandboxes

#### 0.4 done
- add assertion helpers for:
  - stdout
  - stderr
  - exit code
  - emitted artifact existence

### Phase 1: Basic Runnable Apps

#### 1.1 done
- add `scalar_entry`

#### 1.2 done
- add `bindings_and_calls`

#### 1.3 done
- add `control_when`

#### 1.4 done
- add `control_loop_break`

### Phase 2: Package Layout Apps

#### 2.1 done
- add `same_folder_shared_scope`

#### 2.2 done
- add `same_folder_hidden_visibility`

#### 2.3 done
- add `subfolder_namespace`

#### 2.4 done
- add `deep_namespace_chain`

### Phase 3: Local Package Import Apps

#### 3.1 done
- add `loc_plain_values`

#### 3.2 done
- add `loc_types_and_records`

#### 3.3 done
- add `loc_methods`

#### 3.4 done
- add `loc_recoverable_calls`

### Phase 4: std/pkg Apps

#### 4.1 done
- add `std_basic_import`

#### 4.2 done
- add `std_namespace_import`

#### 4.3 done
- add `pkg_basic_import`

#### 4.4 done
- add `pkg_transitive_import`

#### 4.5 done
- add `mixed_loc_std_pkg`

### Phase 5: Data Modeling Apps

#### 5.1 done
- add `record_flow`

#### 5.2 done
- add `entry_flow`

#### 5.3 done
- add `alias_flow`

#### 5.4 done
- add `method_flow`

### Phase 6: Containers And Intrinsics

#### 6.1 done
- add `container_linear`

#### 6.2 done
- add `container_map_set`

#### 6.3 done
- add `container_cross_package`

#### 6.4 done
- add `intrinsics_comparison`

#### 6.5 done
- add `intrinsics_not_len_echo`

#### 6.6 done
- add `intrinsics_panic_check`

### Phase 7: Recoverable And Shell Apps

#### 7.1 done
- add `recoverable_propagation`

#### 7.2 done
- add `recoverable_check`

#### 7.3 done
- add `recoverable_fallback`

#### 7.4 done
- add `recoverable_package_boundary`

#### 7.5 done
- add `shell_optional`

#### 7.6 done
- add `shell_error`

#### 7.7 done
- add `shell_vs_recoverable_boundary`

### Phase 8: Negative Whole-App Fixtures

#### 8.1 done
- add `fail_hidden_cross_file`

#### 8.2 done
- add `fail_loc_targets_formal_pkg_root`

#### 8.3
- add `fail_type_mismatch_real_app`

#### 8.4
- add `fail_recoverable_plain_context`

#### 8.5
- add `fail_shell_unwrap_boundary`

#### 8.6
- add `fail_deferred_intrinsic`

### Phase 9: Gap Harvesting

#### 9.1
- after the first 20+ apps exist, run the full tree and record every real gap

#### 9.2
- convert any new failure into either:
  - a compiler fix
  - or a deliberate expected-failure fixture with explicit diagnostic checks

#### 9.3
- keep every found regression as a permanent fixture; do not throw away the
  discovered real-app repros

### Phase 10: Docs And Status Sync

#### 10.1
- update `PROGRESS.md` to acknowledge the real-app hardening fixture layer

#### 10.2
- update `README.md` if the fixture tree becomes a user-visible recommendation

#### 10.3
- rewrite `PLAN.md` into a hardening completion record once the fixture set is
  real, green, and stable

## 9. Definition Of Done

This hardening milestone is done only when:

- there is a dedicated real-app fixture tree under `test/`
- at least `20` folder fixtures exist
- those fixtures cover the main current `V1` language families listed above
- both success and failure app fixtures exist
- success fixtures compile through the real backend path
- run fixtures execute the produced binaries
- negative fixtures assert stable compiler diagnostics
- mixed `loc` / `std` / `pkg` graphs are represented
- methods, records, entries, aliases, containers, intrinsics, recoverable
  errors, and shells all appear in real app fixtures
- every discovered real-app regression is either fixed or permanently captured
- the hardening layer is treated as part of the normal compiler test suite

## 10. Why This Matters

The current compiler can already claim a runnable `V1` path.

This milestone is about making that claim hard to accidentally break.

Small unit tests are still necessary.
But from here on, real folder programs need to become first-class compiler
infrastructure too.
