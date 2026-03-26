# PLAN: Bootstrap Bundled `std`

This plan builds the first real bundled standard library under:

- [lang/library/std](/home/bresilla/data/code/bresilla/fol/lang/library/std)

The goal is not a giant standard library.

The goal is:

- ship one real bundled `std` package with a small useful surface
- write that public surface in FOL where practical
- keep low-level runtime/compiler substrate in Rust where required
- make `std` import/use feel normal in `fol_model = "std"`
- keep `core` and `mem` as non-importable capability modes

This plan does **not**:

- invent importable `core` or `mem` libraries
- rewrite `fol-build` in FOL
- rewrite `fol-runtime` in FOL
- promise a large or final std API
- add compatibility shims for old experimental std shapes

---

## Design Freeze

The resulting model must be:

- `core` and `mem` remain compiler/runtime capability modes only
- only `std` is a bundled importable library namespace
- bundled `std` ships with the toolchain under `lang/library/std`
- `use std: ...` works only for `fol_model = "std"`
- public `std` modules are authored in FOL by default
- low-level hosted/runtime primitives may remain Rust-backed

The initial bootstrap `std` should stay intentionally small.

Target initial module families:

- `std.fmt`
- `std.io`
- `std.os` only if a minimal truthful surface is ready

Non-goals for this bootstrap:

- collections framework
- allocator APIs
- async/concurrency
- filesystem/process management breadth
- giant string API
- full formatting language

---

## Epoch 1: Freeze The Bootstrap Scope

### Slice 1
Status: complete

Write the bootstrap std contract into active docs.

Completion criteria:

- docs say `std` is bundled and intentionally small
- docs say `core` and `mem` remain non-importable modes

### Slice 2
Status: complete

Define the first public bundled std module set.

Completion criteria:

- one doc lists the initial bootstrap modules
- anything outside that set is explicitly out of scope for this plan

### Slice 3
Status: complete

Audit the current bundled std tree and classify files as:

- keep
- replace
- expand

Completion criteria:

- execution notes identify which current bundled std files are only placeholders

### Slice 4
Status: complete

Add one top-level test that treats the bundled std package as a shipped product.

Completion criteria:

- one integration test proves the bundled std root is present and parseable

---

## Epoch 2: Make `std` A Real Formal Package

### Slice 5
Status: complete

Audit [lang/library/std/build.fol](/home/bresilla/data/code/bresilla/fol/lang/library/std/build.fol) and make it the canonical shipped std package manifest/build entry.

Completion criteria:

- bundled std package metadata is stable and explicit
- the build graph exposes the intended initial std modules

### Slice 6
Status: complete

Add or confirm a real root module source for bundled std.

Completion criteria:

- [lang/library/std/lib.fol](/home/bresilla/data/code/bresilla/fol/lang/library/std/lib.fol) is part of the shipped package contract

### Slice 7
Status: complete

Make bundled std source discovery deterministic and source-only.

Completion criteria:

- bundled std package does not rely on checked-in generated artifacts
- tests assert that the shipped tree is source-only

### Slice 8
Status: complete

Add one package-level parse/typecheck smoke test for bundled std itself.

Completion criteria:

- bundled std package can be loaded and typechecked as a normal formal package

---

## Epoch 3: Bootstrap `std.fmt`

### Slice 9
Status: complete

Define the first honest `std.fmt` surface.

Completion criteria:

- `std.fmt` exports at least one tiny useful routine
- docs/tests name the exact routines

### Slice 10
Status: complete

Implement the first `std.fmt` routines in FOL.

Completion criteria:

- public `std.fmt` routines are authored in FOL source
- no user-facing behavior for those routines lives only in Rust

### Slice 11
Status: complete

Add positive compile/run coverage for `std.fmt`.

Completion criteria:

- one standalone example imports and uses `std.fmt`
- integration tests assert it builds and runs

### Slice 12
Status: complete

Add editor/LSP coverage for `std.fmt`.

Completion criteria:

- hover/completion/semantic tests recognize the shipped `std.fmt` surface

---

## Epoch 4: Bootstrap `std.io`

### Slice 13
Status: complete

Define the first honest `std.io` surface.

Completion criteria:

- `std.io` has a minimal public API
- the API is clearly smaller than future ambitions

### Slice 14
Status: complete

Decide how `.echo(...)` relates to `std.io`.

Completion criteria:

- one explicit rule is documented:
  - either `.echo(...)` stays primitive and `std.io` wraps it
  - or `std.io` becomes the preferred public surface while the primitive remains substrate

### Slice 15
Status: complete

Implement the first `std.io` routines in FOL.

Completion criteria:

- the user-facing `std.io` surface is authored in FOL
- any Rust piece used is clearly substrate-only

### Slice 16
Status: complete

Add positive runtime coverage for `std.io`.

Completion criteria:

- one standalone example imports and uses `std.io`
- integration tests assert real stdout behavior

### Slice 17
Status: complete

Add negative capability coverage for `std.io`.

Completion criteria:

- `core` rejects `use std`
- `mem` rejects `use std`
- error messages remain explicit and stable

---

## Epoch 5: Optional Minimal `std.os`

### Slice 18
Status: complete

Decide whether a truthful minimal `std.os` exists for bootstrap.

Completion criteria:

- either `std.os` is explicitly deferred
- or one tiny real hosted API is approved

### Slice 19
Status: complete

If approved, implement one minimal `std.os` routine in FOL.

Completion criteria:

- the module stays tiny and honest
- it does not pretend to be a broad OS layer

### Slice 20
Status: complete

Add coverage for the chosen `std.os` decision.

Completion criteria:

- tests prove either:
  - minimal `std.os` works
  - or the bootstrap intentionally ships without it

---

## Epoch 6: Package Resolution And Import UX

### Slice 21
Status: complete

Audit `use std: ...` resolution against the real bundled package tree.

Completion criteria:

- import resolution matches the shipped std module layout

### Slice 22
Status: complete

Add tests for missing bundled std modules.

Completion criteria:

- unresolved `std` imports fail cleanly with exact module paths

### Slice 23
Status: complete

Ensure bundled std package/module names stay stable in diagnostics.

Completion criteria:

- diagnostics mention the actual bundled std package/module layout

### Slice 24
Status: complete

Add one regression proving explicit `--std-root` override can swap the bundled std during tests without changing the normal path.

Completion criteria:

- normal bundled std remains default
- override still works for dev/test only

---

## Epoch 7: Examples And Real User Paths

### Slice 25
Status: complete

Add a canonical `std.fmt` example package.

Completion criteria:

- one new example clearly demonstrates `std.fmt`

### Slice 26
Status: complete

Add a canonical `std.io` example package.

Completion criteria:

- one new example clearly demonstrates `std.io`

### Slice 27
Status: complete

Update existing `std` examples to use the bundled modules where appropriate.

Completion criteria:

- old examples stop bypassing the bundled std surface when a bundled module now exists

### Slice 28
Status: complete

Add one negative example proving `use std` under non-`std` models fails through the bundled library path.

Completion criteria:

- one checked-in negative example exercises the exact bundled std boundary

---

## Epoch 8: Editor And Tree-Sitter Hardening

### Slice 29
Status: complete

Add LSP completion coverage for the shipped bundled std modules.

Completion criteria:

- completions reflect the real `std` package tree

### Slice 30
Status: complete

Add hover/definition coverage for bundled std imports.

Completion criteria:

- editor navigation works against real shipped std source

### Slice 31
Status: complete

Audit tree-sitter/highlight coverage for bundled std import examples.

Completion criteria:

- build/source examples that import std stay highlighted sanely

### Slice 32
Status: complete

Update editor-sync docs for bundled std module growth.

Completion criteria:

- editor docs explain what must be audited when std gains new public names

---

## Epoch 9: Docs And Book Alignment

### Slice 33
Status: pending

Document the bundled std bootstrap in the build/book docs.

Completion criteria:

- docs say std ships with FOL
- docs show normal usage without manual dependency setup

### Slice 34
Status: pending

Document the first public std module APIs.

Completion criteria:

- docs list the actual bootstrap `std.fmt` and `std.io` surfaces

### Slice 35
Status: pending

Update runtime-model docs to describe the split clearly:

- `core` and `mem` are modes
- `std` is the importable bundled library

Completion criteria:

- runtime-model docs stop sounding like `std` is only a mode and not a library

### Slice 36
Status: pending

Update contributor guidance for bundled std growth.

Completion criteria:

- docs explain when new std APIs require:
  - tests
  - editor sync
  - examples
  - book updates

---

## Epoch 10: Final Hardening And Closure

### Slice 37
Status: pending

Run a bundled std example/build/doc sync audit.

Completion criteria:

- docs point at real std examples
- examples use real shipped std modules

### Slice 38
Status: pending

Add one top-level integration matrix for:

- bundled std import
- `fol_model = std`
- editor readability
- runtime execution

Completion criteria:

- one integration suite pins the bootstrap contract across layers

### Slice 39
Status: pending

Audit for stale placeholder/std-seed wording.

Completion criteria:

- bootstrap std files/docs now describe the current real state, not the old seed state

### Slice 40
Status: pending

Do a repo-wide sweep for accidental public `std` claims beyond what bootstrap actually ships.

Completion criteria:

- docs/examples/tests do not imply a larger std than exists

### Slice 41
Status: pending

Run a final build/test gate for the bootstrap std plan.

Completion criteria:

- `make build` passes
- `make test` passes

### Slice 42
Status: pending

Verify the bundled std tree stays source-only and clean after the full test suite.

Completion criteria:

- no checked-in generated artifacts in bundled std

### Slice 43
Status: pending

Close the plan with a final shipped-surface summary.

Completion criteria:

- docs/tests make the actual bootstrap std surface obvious

### Slice 44
Status: pending

Mark the plan complete on a clean worktree.

Completion criteria:

- worktree is clean
- all slices are completed

---

## Success Criteria

This plan is complete when:

- `std` is a real bundled formal package under `lang/library/std`
- the first public bundled std modules are real and authored in FOL
- `use std: ...` works against the shipped package tree in `fol_model = "std"`
- `core` and `mem` remain non-importable modes
- docs/examples/editor coverage match the real shipped std surface
- the repo stays honest about bootstrap scope
