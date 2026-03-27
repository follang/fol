# PLAN: Bundled Std Phase 2 And Internal `alloc` -> `memo`

This plan combines the next two approved tracks:

1. grow the bundled shipped `std` a little further
2. rename the internal runtime seam from `alloc` to `memo`

There is no compatibility path.

If a new name or contract is chosen, the old one is deleted.

## Goals

- keep bundled `std` small, honest, and actually useful
- keep `std` authored in FOL where practical
- keep low-level hosted/runtime substrate in Rust
- make internal runtime naming match the public `memo` model
- stop leaking the old internal `alloc` name through backend output, tests,
  traces, and docs

## Non-Goals

This plan does not:

- turn `core` or `memo` into importable libraries
- make `std` ambient again
- make `std` large
- introduce speculative `std.os` surface without a real substrate API
- keep `fol_runtime::alloc` around as a compatibility alias

## Current Repo Reality

The current bundled `std` tree is:

- `lang/library/std/lib.fol`
- `lang/library/std/fmt/root.fol`
- `lang/library/std/fmt/math/lib.fol`
- `lang/library/std/io/lib.fol`

Current shipped public routines are still tiny:

- `std::fmt::answer()`
- `std::fmt::double(int)`
- `std::fmt::math::answer()`
- `std::io::echo_int(int)`
- `std::io::echo_str(str)`

The current internal runtime seam still uses:

- `lang/execution/fol-runtime/src/alloc.rs`
- `pub mod alloc;` in `lang/execution/fol-runtime/src/lib.rs`
- backend emit paths like `use fol_runtime::alloc as rt;`
- many integration and backend tests pinned to `fol_runtime::alloc`

So the next real work is:

- add a few more honest bundled-std helpers
- rename the internal runtime seam completely

## Final Contract After This Plan

After this plan:

- bundled `std` remains explicit and dependency-backed
- bundled `std` ships a slightly larger but still honest FOL surface
- internal runtime naming uses `memo` consistently:
  - file/module paths
  - backend runtime module selection
  - emitted Rust imports
  - trace output
  - tests/docs/examples that mention the internal runtime seam

## Epoch 1: Freeze Scope And Honest Surface

### Slice 1
Status: complete

Write the phase-2 scope into active docs:

- bundled `std` is still intentionally small
- only real shipped surfaces may be documented
- internal runtime rename is implementation cleanup, not a public API change

### Slice 2
Status: complete

Write the same scope into contributor guidance:

- `AGENTS.md`
- `lang/library/std/README.md`

### Slice 3
Status: complete

Add one shipped-surface contract test that pins the currently documented
bundled-std modules and forbids undocumented extras.

## Epoch 2: Expand `std.fmt` Slightly

### Slice 4
Status: complete

Audit the current `std.fmt` tree and choose one small expansion that stays
honest and substrate-free.

Examples of acceptable additions:

- a tiny arithmetic helper family
- a tiny string-composition helper family
- a tiny boolean/name helper family

Do not invent large formatting machinery yet.

### Slice 5
Status: complete

Implement the chosen `std.fmt` additions in FOL under:

- `lang/library/std/fmt/root.fol`
- or a small nested namespace if needed

### Slice 6
Status: complete

Add bundled-std example coverage for the new `std.fmt` routines.

### Slice 7
Status: complete

Add CLI/integration coverage proving the new `std.fmt` routines build and run
through the shipped bundled std.

## Epoch 3: Expand `std.io` Slightly

### Slice 8
Status: complete

Audit the current hosted substrate and choose one or two additional honest
`std.io` wrappers.

Only wrap real substrate that already exists cleanly.

### Slice 9
Status: complete

Implement those wrappers in:

- `lang/library/std/io/lib.fol`

### Slice 10
Status: complete

Add or update one canonical bundled-std example so `std.io` is the preferred
public story instead of raw `.echo(...)` when equivalent wrappers exist.

### Slice 11
Status: complete

Keep exactly one tiny explicit raw-substrate example and ensure docs label it
as substrate-level, not preferred public style.

## Epoch 4: Optional Tiny `std.os` Decision

### Slice 12
Status: complete

Audit whether there is one honest hosted/OS wrapper worth shipping now.

If no honest surface exists, explicitly document that `std.os` remains absent.

### Slice 13
Status: complete

Only if there is a real honest wrapper:

- add `lang/library/std/os/...`
- add one example
- add one integration test

Otherwise mark the “no `std.os` yet” contract more strongly in docs/tests.

## Epoch 5: Bundled Std Tooling And Editor Sync

### Slice 14
Status: complete

Update editor/LSP completion coverage for any new bundled-std symbols.

### Slice 15
Status: complete

Update hover/definition coverage so new bundled-std public names resolve
cleanly from real shipped examples.

### Slice 16
Status: complete

Update tree-sitter real-example highlight coverage for the new bundled-std
example sources.

### Slice 17
Status: complete

Add one top-level shipped-std scan test that keeps examples/docs/readme in sync
with the real bundled module tree.

## Epoch 6: Rename Runtime Module File And Export

### Slice 18
Status: complete

Rename:

- `lang/execution/fol-runtime/src/alloc.rs`

to:

- `lang/execution/fol-runtime/src/memo.rs`

### Slice 19
Status: complete

Update `lang/execution/fol-runtime/src/lib.rs` so the public internal module
export is:

- `pub mod memo;`

and `pub mod alloc;` is deleted.

### Slice 20
Status: complete

Update internal runtime docs/comments to refer to `memo` instead of `alloc`.

### Slice 21
Status: complete

Update internal runtime unit tests to assert:

- `fol_runtime::memo::tier_name()`
- `fol_runtime::std::base_memo_tier()` or equivalent renamed seam

with no stale `alloc` naming left.

## Epoch 7: Backend Runtime-Tier Cutover

### Slice 22
Status: complete

Update backend runtime-module selection so `BackendFolModel::Memo` maps to:

- `fol_runtime::memo`

instead of `fol_runtime::alloc`.

### Slice 23
Status: complete

Update backend emitted Rust snapshots and tests so generated imports use:

- `use fol_runtime::memo as rt;`
- `use fol_runtime::memo as rt_model;`

where appropriate.

### Slice 24
Status: complete

Update backend trace output so it reports:

- `runtime_module=fol_runtime::memo`

instead of `fol_runtime::alloc`.

### Slice 25
Status: complete

Update compile/build-route mapping tests that still pin emitted `alloc`
runtime imports.

## Epoch 8: Frontend And Integration Rename Sweep

### Slice 26
Status: complete

Update frontend compile/build-route tests that still assert:

- `use fol_runtime::alloc as rt;`

for memo artifacts.

### Slice 27
Status: complete

Update integration tests in:

- `test/integration_tests/integration_editor_and_build.rs`

so memo/runtime expectations use:

- `use fol_runtime::memo as rt;`

### Slice 28
Status: complete

Update any routed/CLI human-readable trace text that still mentions the old
internal runtime module.

### Slice 29
Status: complete

Add one integration regression that explicitly proves memo artifacts now emit
and run through the renamed internal `fol_runtime::memo` path.

## Epoch 9: Repo-Wide Stale Sweep

### Slice 30
Status: complete

Repo-wide stale sweep for:

- `fol_runtime::alloc`
- `use fol_runtime::alloc`
- `alloc as rt`
- `runtime_module=fol_runtime::alloc`

in active source, tests, docs, and examples.

### Slice 31
Status: complete

Add one top-level scan test that fails if stale `fol_runtime::alloc` references
reappear in the active repo surface.

### Slice 32
Status: complete

Update docs that explain runtime tiers so the internal implementation seam is
named `memo` consistently.

## Epoch 10: Bundled Std Phase-2 Closure

### Slice 33
Status: complete

Update:

- `docs/bundled-std.md`
- `lang/library/std/README.md`
- relevant book pages

to list exactly the new shipped bundled-std routines and examples.

### Slice 34
Status: complete

Update contributor guidance so future std additions must:

- be real shipped FOL source
- have example coverage
- have editor/tree-sitter audit
- avoid overstating unshipped surfaces

### Slice 35
Status: complete

Add one top-level “shipped std honesty” matrix test that pins:

- current bundled module tree
- current public routine list
- current canonical examples

### Slice 36
Status: complete

Run `make build`.

### Slice 37
Status: complete

Run `make test`.

### Slice 38
Status: complete

Only if both pass, mark the plan complete.
