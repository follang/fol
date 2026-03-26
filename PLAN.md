# PLAN: Rename `alloc` Model To `mem`

This plan performs a full cutover from:

- `fol_model = "alloc"`

to:

- `fol_model = "mem"`

The intent is:

- keep the three capability modes:
  - `core`
  - `mem`
  - `std`
- keep `core` and `mem` as non-importable compiler/runtime capability layers
- keep only `std` as the importable library namespace
- remove `alloc` as the public model spelling everywhere

This is a naming and architecture-alignment change across compiler, runtime, tooling,
tests, docs, and examples. It is not a request to create a public `mem` library.

---

## Design Freeze

The resulting model must be:

- user-facing build choice:
  - `fol_model = "core"`
  - `fol_model = "mem"`
  - `fol_model = "std"`
- source-level imports:
  - only `std` is importable
  - `core` and `mem` are not importable
- runtime/compiler substrate:
  - `core`
  - `mem`
  - `std`
- bundled std rules remain unchanged:
  - `use std ...` is forbidden in `core`
  - `use std ...` is forbidden in `mem`
  - `use std ...` is allowed in `std`

Non-goals for this plan:

- do not keep `alloc` as compatibility spelling
- do not add dual parsing for `alloc | mem`
- do not add migration warnings
- do not invent a public `mem` import namespace
- do not change the current bundled std plan into `std.mem`

---

## Epoch 1: Contract Freeze And Scan Lock

### Slice 1
Status: completed

Write the top-level naming contract into active docs.

Completion criteria:

- docs explicitly say the three modes are `core`, `mem`, `std`
- docs explicitly say `alloc` is no longer a valid model name

### Slice 2
Status: completed

Add one compiler-owned naming matrix test.

Completion criteria:

- a test asserts the canonical public model spellings are exactly:
  - `core`
  - `mem`
  - `std`

### Slice 3
Status: completed

Audit the repo for all active `alloc` model spellings and group them by subsystem.

Completion criteria:

- plan execution notes identify:
  - compiler/runtime surfaces
  - frontend/tooling surfaces
  - examples/fixtures
  - docs/book

### Slice 4
Status: completed

Freeze the rule that only the runtime substrate may still use “allocation” wording in purely internal comments where it is not the public model name.

Completion criteria:

- docs/tests/public diagnostics are pinned to `mem`
- internal implementation wording is allowed only when it refers to heap allocation as a concept, not the model name

---

## Epoch 2: Public Model Parsing And Validation

### Slice 5
Status: completed

Change build-surface parsing/validation so only `mem` is accepted as the middle model.

Completion criteria:

- `fol_model = "mem"` parses and validates
- `fol_model = "alloc"` is rejected

### Slice 6
Status: completed

Remove `alloc` from public model enums/string conversions in package/build-facing types.

Completion criteria:

- package/build public types render `mem`
- no public package/build string conversion still emits `alloc`

### Slice 7
Status: completed

Add direct negative tests for `fol_model = "alloc"` in build files.

Completion criteria:

- negative tests fail explicitly on the old spelling

### Slice 8
Status: completed

Update build diagnostics to point only at `core`, `mem`, `std`.

Completion criteria:

- invalid-model diagnostics list only the new canonical set

### Slice 9
Status: completed

Update workspace/build metadata extraction so artifact models serialize as `mem`.

Completion criteria:

- extracted artifact metadata no longer emits `alloc`

---

## Epoch 3: Typecheck Capability Model Cutover

### Slice 10
Status: completed

Rename the public capability model variant from `Alloc` to `Mem` in typecheck-facing APIs.

Completion criteria:

- typecheck public/config-facing APIs use `Mem`

### Slice 11
Status: completed

Update typecheck diagnostics from `alloc` wording to `mem`.

Completion criteria:

- hosted-surface rejection messages say:
  - current artifact model is `mem`

### Slice 12
Status: completed

Keep heap-backed capability behavior unchanged while renaming the visible model.

Completion criteria:

- `str`, `seq`, `vec`, `set`, `map`, dynamic `.len(...)` remain legal in `mem`
- hosted-only surfaces remain illegal in `mem`

### Slice 13
Status: completed

Add capability tests that prove `mem` is behavior-preserving relative to the former middle tier.

Completion criteria:

- one matrix test proves:
  - `core` rejects heap
  - `mem` accepts heap
  - `mem` rejects hosted std
  - `std` accepts hosted std

---

## Epoch 4: Runtime And Backend Naming Cutover

### Slice 14
Status: completed

Rename backend public model enums/strings from `Alloc` to `Mem`.

Completion criteria:

- backend config surfaces render `mem`

### Slice 15
Status: completed

Rename runtime-tier selection strings from `alloc` to `mem` in emitted metadata and traces.

Completion criteria:

- emitted trace/report surfaces use `mem`

### Slice 16
Status: completed

Decide runtime module-path strategy and apply it consistently.

Completion criteria:

- either:
  - runtime module path is renamed from `fol_runtime::alloc` to `fol_runtime::mem`
- or:
  - runtime module path stays internal and docs/tests clearly distinguish internal path vs public model name

Note:

- this slice must choose one path and remove the other
- no dual public wording

### Slice 17
Status: completed

Update backend emission tests to the chosen `mem` contract.

Completion criteria:

- emitted-Rust/import tests reflect the final chosen naming strategy

### Slice 18
Status: completed

Update backend/build summaries so model lines show `mem`.

Completion criteria:

- build summaries no longer show `fol_model=alloc`

---

## Epoch 5: Frontend Routing And Reporting

### Slice 19
Status: completed

Update routed build/run/test planning to use `mem` in model distribution and diagnostics.

Completion criteria:

- routed summaries use `mem`
- ambiguity diagnostics use `mem`

### Slice 20
Status: completed

Update `work info` / `work status` model-distribution reporting.

Completion criteria:

- distribution text becomes:
  - `core=...`
  - `mem=...`
  - `std=...`

### Slice 21
Status: pending

Update scaffolding templates so generated examples and projects use `mem`.

Completion criteria:

- no scaffold emits `fol_model = "alloc"`

### Slice 22
Status: pending

Add CLI integration tests for `mem` routing and old `alloc` rejection.

Completion criteria:

- routed CLI tests cover:
  - `mem` positive path
  - old `alloc` negative path

---

## Epoch 6: Editor, LSP, And Tree-Sitter

### Slice 23
Status: pending

Rename editor workspace model recovery from `Alloc` to `Mem`.

Completion criteria:

- editor active-model recovery uses `Mem`

### Slice 24
Status: pending

Update LSP diagnostics, hover, and completion tests to `mem`.

Completion criteria:

- model-aware LSP tests use `mem`
- no active LSP test still expects `alloc`

### Slice 25
Status: pending

Update build-file semantic-token and tree-sitter tests for `mem`.

Completion criteria:

- build-file token/highlight tests use `core/mem/std`

### Slice 26
Status: pending

Audit compiler-backed editor sync helpers for stale `alloc` wording.

Completion criteria:

- no active editor sync helper advertises `alloc` as a public model spelling

---

## Epoch 7: Example And Fixture Migration

### Slice 27
Status: pending

Rename or replace example packages whose names or expectations encode `alloc` as the public model.

Completion criteria:

- example package names and assertions align with `mem` where they refer to the model

### Slice 28
Status: pending

Migrate build fixtures from `model_alloc_*` naming to `model_mem_*` where the name is model-facing.

Completion criteria:

- checked-in model fixtures use `mem`

### Slice 29
Status: pending

Update positive example summaries/import assertions from `alloc` to `mem`.

Completion criteria:

- normal example integration coverage expects `mem`

### Slice 30
Status: pending

Update negative examples and their diagnostics from `alloc` to `mem`.

Completion criteria:

- negative hosted-boundary examples use `mem`

### Slice 31
Status: pending

Add at least one new canonical `mem` showcase example so the new name is visible in the shipped example set.

Completion criteria:

- example tree contains one obvious positive `mem` example

---

## Epoch 8: Transitive Boundary Hardening Under `mem`

### Slice 32
Status: pending

Update transitive boundary tests from `alloc` to `mem`.

Completion criteria:

- `core -> mem` failure path is pinned
- `mem -> mem` success path is pinned
- `mem -> std` hosted rejection is pinned

### Slice 33
Status: pending

Update runtime import cleanliness tests for the renamed middle model.

Completion criteria:

- cross-package emission audits use `mem`

### Slice 34
Status: pending

Update mixed-model workspace tests from `core/alloc/std` to `core/mem/std`.

Completion criteria:

- mixed-workspace coverage surfaces `mem`

### Slice 35
Status: pending

Add one regression test proving that a stale `alloc` model in a dependency fixture fails before planning/emission.

Completion criteria:

- the old spelling is rejected early even in transitive/mixed setups

---

## Epoch 9: Docs, Book, And Contributor Guidance

### Slice 36
Status: pending

Update runtime-model docs from `alloc` to `mem`.

Completion criteria:

- [docs/runtime-models.md](/home/bresilla/data/code/bresilla/fol/docs/runtime-models.md) uses only `core/mem/std`

### Slice 37
Status: pending

Update build book examples and wording from `alloc` to `mem`.

Completion criteria:

- build book examples use `fol_model = "mem"`

### Slice 38
Status: pending

Update user/tooling docs that mention the middle model.

Completion criteria:

- tooling docs use `mem`

### Slice 39
Status: pending

Update contributor guidance in `AGENTS.md` if it still references `alloc` as a model name.

Completion criteria:

- contributor guidance matches `core/mem/std`

### Slice 40
Status: pending

Update any roadmap/progress notes that still describe `alloc` as the public model.

Completion criteria:

- active project notes use `mem`

---

## Epoch 10: Final Repo Sweep And Closure

### Slice 41
Status: pending

Perform a repo-wide sweep for stale public `alloc` model strings.

Completion criteria:

- remaining `alloc` hits are only:
  - internal implementation comments about memory allocation
  - intentionally chosen internal runtime names if retained by design

### Slice 42
Status: pending

Audit diagnostics snapshots and human-readable integration expectations for stale `alloc` wording.

Completion criteria:

- no active user-facing diagnostic still says `alloc` as the model name

### Slice 43
Status: pending

Run a final example/build/doc sync audit.

Completion criteria:

- docs point at real `mem` examples
- integration suites match the renamed examples/fixtures

### Slice 44
Status: pending

Close the plan with one final green-state scan.

Completion criteria:

- `make build` passes
- `make test` passes
- worktree is clean
- plan is fully marked complete

---

## Success Criteria

The plan is complete when:

- the public middle model is `mem` everywhere
- `alloc` is no longer accepted as a public `fol_model` spelling
- user-facing docs, diagnostics, and examples consistently say:
  - `core`
  - `mem`
  - `std`
- `core` and `mem` remain non-importable capability modes
- only `std` remains the bundled importable standard library namespace
