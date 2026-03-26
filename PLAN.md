# PLAN: Bundled FOL Library Root At `lang/library/std`

This plan introduces a bundled FOL standard-library source tree under:

- `lang/library/std`

The intent is:

- `std` is part of the FOL distribution
- `std` is written mostly in FOL
- `std` is not an external dependency the user downloads separately
- `std` is not part of the compiler implementation crates
- `core` and `mem` remain non-importable runtime/compiler capability layers implemented in Rust
- `use std ...` resolves against the bundled shipped library root
- `use std ...` is only legal when `fol_model = "std"`

This is a packaging and toolchain-distribution change, not a rewrite of the compiler into FOL.

---

## Design Freeze

The resulting model must be:

- runtime/compiler capability substrate:
  - `core`
  - `mem`
  - `std`
- only `std` is an importable bundled library namespace
- `core` and `mem` are selected through `fol_model`, not imported from source code
- shipped FOL library root:
  - `lang/library/std`
- default toolchain behavior:
  - bundled `std` is mounted automatically
- override behavior:
  - explicit std-root override may remain only for development/testing
  - no user-facing requirement to configure std manually

Non-goals for this plan:

- do not rewrite runtime/compiler implementation into FOL
- do not create public bundled `core` or `mem` library trees
- do not make `core` or `mem` importable namespaces
- do not try to build the full giant std library in one pass
- do not add compatibility back for external ad-hoc std roots as the normal path

---

## Epoch 1: Contract And Filesystem Layout

### Slice 1
Status: complete

Add `lang/library/` to the repo layout and document its role.

Completion criteria:

- `lang/library/` exists in the tree
- docs state that bundled FOL libraries live there

### Slice 2
Status: complete

Add `lang/library/std/` as the canonical bundled standard-library root.

Completion criteria:

- `lang/library/std/` exists
- repository conventions document it as the canonical std package root

### Slice 3
Status: complete

Write a concise toolchain-facing design note for bundled std semantics.

Completion criteria:

- note explains shipped std vs runtime/compiler substrate
- note explains that users should not download std separately

### Slice 4
Status: complete

Audit existing docs and examples that still imply manual std-root configuration as the normal path.

Completion criteria:

- all such sites are listed before behavior changes start

---

## Epoch 2: Frontend Default Std Root Resolution

### Slice 5
Status: complete

Add one canonical helper that computes the bundled std root from the repo/toolchain layout.

Completion criteria:

- frontend/package/resolver use one shared helper for the default bundled std root

### Slice 6
Status: complete

Make frontend commands default to bundled `lang/library/std` when no override is given.

Completion criteria:

- compile/build/check/run/test paths pick bundled std automatically

### Slice 7
Status: complete

Keep explicit std-root override only as a development/testing escape hatch.

Completion criteria:

- override still works
- docs clearly mark it as override behavior, not normal setup

### Slice 8
Status: complete

Update workspace/config loading so missing explicit std-root no longer means “no std available” when bundled std exists.

Completion criteria:

- workspace configs without std-root still resolve bundled std by default

### Slice 9
Status: complete

Harden frontend diagnostics around bundled std discovery.

Completion criteria:

- diagnostics distinguish:
  - bundled std missing from toolchain layout
  - explicit override missing
  - `use std` forbidden by model

---

## Epoch 3: Package And Resolver Integration

### Slice 10
Status: complete

Route package-session configuration to the bundled std root by default.

Completion criteria:

- package session gets bundled std without explicit caller plumbing everywhere

### Slice 11
Status: complete

Route resolver-session configuration to the bundled std root by default.

Completion criteria:

- resolver tests no longer require explicit std-root in the normal positive path

### Slice 12
Status: complete

Preserve model gating: bundled std resolution must still fail under non-`std` models when `use std` appears.

Completion criteria:

- bundled std availability does not weaken `fol_model` legality rules

### Slice 13
Status: complete

Update std import resolution tests to prefer bundled std fixtures over ad-hoc explicit std-root setup where appropriate.

Completion criteria:

- positive std import tests use the new default path in at least one canonical suite

### Slice 14
Status: complete

Keep one focused suite for explicit std-root override behavior.

Completion criteria:

- override behavior remains tested without dominating normal-path tests

---

## Epoch 4: Seed Bundled Std Package

### Slice 15
Status: complete

Create the first minimal bundled std package structure under `lang/library/std`.

Completion criteria:

- package root is structurally valid
- build/package metadata is valid under the current build model

### Slice 16
Status: complete

Add one minimal module in bundled std that is safe and tiny.

Completion criteria:

- module is written in FOL
- module compiles as part of std import tests

### Slice 17
Status: complete

Add one honest hosted-oriented module namespace placeholder.

Completion criteria:

- `std` tree clearly leaves room for:
  - `io`
  - `os`
  - later growth

### Slice 18
Status: complete

Write one positive example that imports bundled std from the shipped tree rather than a temporary fixture root.

Completion criteria:

- integration test proves bundled std is actually being used

---

## Epoch 5: Development Workflow And Overrides

### Slice 19
Status: complete

Define the expected contributor workflow for editing bundled std locally.

Completion criteria:

- docs explain how to iterate on `lang/library/std`
- docs explain when to use override flags

### Slice 20
Status: complete

Ensure CLI/dev workflows can still point to an alternate std root for experimentation.

Completion criteria:

- explicit override path is covered by tests

### Slice 21
Status: complete

Audit editor/LSP workspace logic to ensure bundled std can be discovered without special manual config.

Completion criteria:

- editor can resolve bundled std in normal repo usage

### Slice 22
Status: complete

Add LSP/regression coverage for bundled std discovery.

Completion criteria:

- hover/navigation/diagnostics work against bundled std without explicit std-root wiring

---

## Epoch 6: Docs, Book, And User Story

### Slice 23
Status: complete

Update build/runtime-model docs to say std ships with FOL.

Completion criteria:

- docs stop implying users fetch or separately install std
- docs state that only `std` is imported from source code
- docs state that `core` and `mem` remain model/capability choices only

### Slice 24
Status: complete

Update book sections that discuss `use std` to describe bundled shipped std.

Completion criteria:

- book wording matches the new toolchain story
- no book page suggests `use core` or `use mem`

### Slice 25
Status: complete

Add a concise “what ships with FOL” section.

Completion criteria:

- docs distinguish:
  - compiler/runtime substrate
  - bundled std library source
  - optional external dependencies

### Slice 26
Status: complete

Update examples/docs that currently use temporary synthetic std fixtures as if they were the normal model.

Completion criteria:

- docs point at bundled std for the normal case

---

## Epoch 7: Test Fixture Cleanup

### Slice 27
Status: complete

Audit tests that create ad-hoc std fixture trees and classify them.

Completion criteria:

- tests are grouped into:
  - normal bundled-std tests
  - explicit override tests
  - special isolated resolver tests

### Slice 28
Status: complete

Migrate the normal-path frontend integration tests to bundled std.

Completion criteria:

- normal compile/build integration no longer needs ad-hoc std roots

### Slice 29
Status: complete

Migrate the normal-path resolver tests to bundled std where practical.

Completion criteria:

- at least one canonical resolver std-resolution suite uses bundled std

### Slice 30
Status: complete

Keep isolated synthetic std fixtures only where they are clearly needed for narrow resolver behavior.

Completion criteria:

- remaining synthetic std fixtures are justified and documented in tests

---

## Epoch 8: Shipped Std Package Hardening

### Slice 31
Status: pending

Add a minimal build/package manifest for bundled std that matches the current `.build()` model.

Completion criteria:

- bundled std has valid package/build metadata under current rules

### Slice 32
Status: pending

Add one tiny exported symbol/module and one importer test package.

Completion criteria:

- importing bundled std works through the normal package path

### Slice 33
Status: pending

Add one negative test proving `use std` still fails under `fol_model = "core"`.

Completion criteria:

- bundled std presence does not bypass model gating

### Slice 34
Status: pending

Add one negative test proving `use std` still fails under `fol_model = "mem"`.

Completion criteria:

- mem-mode rejection is explicit and stable

### Slice 35
Status: pending

Add one positive hosted example package that relies on bundled std and runs.

Completion criteria:

- bundled std is exercised in a runnable end-to-end example

---

## Epoch 9: Tooling And Distribution Hardening

### Slice 36
Status: pending

Audit initialization/scaffolding commands so new std-mode projects do not teach manual std dependency setup.

Completion criteria:

- scaffolded projects rely on bundled std implicitly

### Slice 37
Status: pending

Audit build summaries and info/report surfaces for std-root wording.

Completion criteria:

- summaries refer to bundled std or explicit override accurately

### Slice 38
Status: pending

Add one integration suite that asserts bundled std discovery across:

- compile
- build
- run
- editor/LSP

Completion criteria:

- one suite catches cross-layer bundled-std drift

### Slice 39
Status: pending

Audit backend/integration tests that currently inject temporary std roots for emitted Rust checks and convert the normal path to bundled std where possible.

Completion criteria:

- emitted-Rust tests use bundled std in at least one normal-path coverage suite

---

## Epoch 10: Final Cleanup And Follow-Up Readiness

### Slice 40
Status: pending

Create a short roadmap note for how std grows inside `lang/library/std`.

Completion criteria:

- note explains that std starts small and grows gradually

### Slice 41
Status: pending

Define the first intended std module families without implementing the whole library.

Completion criteria:

- roadmap names likely first families such as:
  - `std.io`
  - `std.os`
  - one small utility/text family if needed

### Slice 42
Status: pending

Audit naming consistency around “bundled std”, “standard library”, and “std root”.

Completion criteria:

- wording is consistent across docs, tests, and diagnostics

### Slice 43
Status: pending

Remove stale assumptions that std is external user-managed package content.

Completion criteria:

- no active docs or tests imply that users normally fetch std separately

### Slice 44
Status: pending

Perform a final repo-wide scan for:

- stale std-root assumptions
- stale docs
- tests that still treat synthetic std as the normal path
- missing `lang/library/std` references

Completion criteria:

- scan is clean before plan closure

---

## Success Criteria

This plan is complete when all of the following are true:

- `lang/library/std` exists and is the canonical bundled std source root
- frontend/package/resolver default to bundled std automatically
- explicit std-root override remains only as a development/testing override
- bundled std is exercised by real positive tests and examples
- `use std` still respects `fol_model = "std"` and remains illegal under `core` and `mem`
- docs and tests consistently treat `core` and `mem` as non-importable model choices
- docs explain that std ships with FOL and does not need separate download
- the repo no longer treats temporary custom std roots as the normal user path
