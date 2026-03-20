# `bic` -> `fol` Native Linking Plan

Last updated: 2026-03-20

## Purpose

This plan replaces the previous `fol` plan.

The immediate goal is to make `fol` capable of consuming `bic` output and
producing a final executable or library that links against native C code
through the existing Rust backend pipeline.

The intended execution model is:

```text
C headers + native artifacts
    -> bic
    -> binding / validation / link metadata
    -> fol lowering + native attachment planning
    -> generated Rust crate (+ build.rs if needed)
    -> cargo build
    -> rustc
    -> system linker
    -> final fol artifact
```

This plan is specifically about making that path real.

## Current State

After inspecting `fol`, the current picture is:

### What already exists

- `fol-build` already has a native model:
  - `NativeIncludePath`
  - `NativeLibraryPath`
  - `NativeLinkDirective`
  - `NativeLinkInput`
  - `NativeLinkMode`
- `BuildArtifactDefinition` already has `native_attachments`
- `fol` already has a build graph / artifact graph / backend emission pipeline
- the backend already emits a Rust crate and builds it through `cargo build`

### What is still missing

- no `bic` bridge exists inside `fol`
- no lowering path turns `bic::BindingPackage` / `bic::ResolvedLinkPlan` into
  `fol-build` native attachments
- the generated Cargo crate currently only depends on `fol-runtime`
- the backend does not emit `build.rs`
- the backend does not emit:
  - `cargo:rustc-link-search=...`
  - `cargo:rustc-link-lib=...`
- concrete native artifact copying / staging policy is not wired
- no end-to-end example proves that a `fol` program can consume `bic` metadata
  and then successfully link a native library

## Guiding Rules

- keep `fol` as the owner of final build and link policy
- keep `bic` as the owner of C-surface discovery, ABI evidence, validation,
  and link-intent discovery
- do not duplicate C parsing or link planning logic inside `fol`
- prefer a code-driven integration path over file-driven configuration
- make the first end-to-end path work on Linux/ELF first
- keep Windows out of scope for now
- keep the generic native-linking model primary; `zlib` / SocketCAN examples are
  proofs, not hardcoded product behavior

## Non-Goals

This plan does not attempt to:

- replace Cargo or `rustc`
- make `fol` call `ldd` as part of building
- implement a full runtime loader / `dlopen` deployment manager
- solve packaging of arbitrary third-party native dependencies on every system
- make OpenSSL / libcurl the first target

Those can come later.

## Final Architecture Target

At the end of this roadmap, `fol` should support this flow:

1. `fol` code invokes `bic` programmatically
2. `fol` receives:
   - `BindingPackage`
   - `ValidationReport`
   - `ResolvedLinkPlan`
3. `fol` lowers that into:
   - binding/codegen IR
   - `fol-build` native attachment IR
4. the Rust backend emits:
   - Rust source
   - `Cargo.toml`
   - `build.rs` when native linking is required
5. Cargo builds the generated crate
6. Cargo/rustc invoke the system linker with the required native inputs
7. `fol` returns a built artifact with clear diagnostics if validation or
   linking policy blocks the build

## Main Workstreams

## Workstream A: `bic` Integration Surface in `fol`

### Why this matters

`fol` cannot use `bic` until there is a stable bridge layer that owns the
translation from `bic` concepts into `fol` concepts.

### Deliverables

- one bridge module or crate for `bic` integration
- a code-driven request/response API
- one typed lowering path from `bic` output into `fol` build and backend inputs

### Proposed slices

- Slice A1: add `bic` as a dependency in the relevant `fol` crate(s)
- Slice A2: create a dedicated integration module for `bic` consumption
- Slice A3: define a `FolNativeInteropRequest` and `FolNativeInteropResult`
  model
- Slice A4: add a scan-only integration path that returns `BindingPackage`
  without yet attempting final linking
- Slice A5: add a validation-aware integration path that also returns
  `ValidationReport`
- Slice A6: add a link-plan-aware integration path that also returns
  `ResolvedLinkPlan`

### Acceptance criteria

- `fol` can call `bic` directly from Rust code
- `fol` has one obvious integration surface instead of ad hoc usage

## Workstream B: Lower `bic` Link Metadata Into `fol-build`

### Why this matters

The bridge is not enough by itself.
`fol` must translate `bic` link metadata into its own native attachment model.

### Deliverables

- a deterministic lowering from `bic::BindingLinkSurface`
  and/or `bic::ResolvedLinkPlan` into `BuildArtifactNativeAttachmentSet`
- explicit lowering rules for:
  - include paths
  - library paths
  - named libraries
  - concrete artifact files
  - static vs dynamic preference

### Proposed slices

- Slice B1: inventory exact field mapping from `bic` link structures to
  `fol-build::native`
- Slice B2: implement lowering for include paths and library paths
- Slice B3: implement lowering for named library link directives
- Slice B4: implement lowering for concrete static/shared artifact inputs
- Slice B5: implement lowering for static vs dynamic preference
- Slice B6: add unit tests for the lowering layer with fixture-like `bic` input

### Acceptance criteria

- `bic` link metadata can be represented fully enough in `fol-build`
- lowering is deterministic and tested

## Workstream C: Native Attachment APIs In `fol-build`

### Why this matters

Even if lowering exists, `fol-build` still needs a supported way to attach
native requirements to artifacts in the graph/evaluator/build API.

### Deliverables

- a stable way to associate native attachments with build artifacts
- evaluator / graph support for those attachments
- tests proving the graph preserves them

### Proposed slices

- Slice C1: audit whether `BuildArtifactDefinition.native_attachments` is only a
  passive model or already reachable from build evaluation
- Slice C2: add graph or API support to attach native inputs to an artifact
- Slice C3: thread native attachments through evaluation / projection code
- Slice C4: add tests that preserve include paths, library paths, and link
  directives through the graph
- Slice C5: add docs for the native-attachment model in the build book

### Acceptance criteria

- `fol-build` can carry native attachment data from evaluation to backend input
- the data is not just modeled; it is reachable and preserved

## Workstream D: Backend Emission of Real Native Link Instructions

### Why this matters

This is the core missing piece.
Today the backend emits a Rust crate and runs Cargo, but does not emit the
native-link instructions that Cargo/rustc need.

### Deliverables

- generated `build.rs` support when native linking is required
- generated Cargo metadata if needed
- build-script emission for:
  - `cargo:rustc-link-search`
  - `cargo:rustc-link-lib`
- artifact path handling for concrete native libraries

### Proposed slices

- Slice D1: design the generated-crate native-link emission contract
- Slice D2: extend backend crate emission to optionally generate `build.rs`
- Slice D3: emit `cargo:rustc-link-search` from lowered library paths
- Slice D4: emit `cargo:rustc-link-lib` from lowered named libraries
- Slice D5: emit concrete artifact path handling for declared static/shared
  artifacts
- Slice D6: add backend tests that inspect generated `Cargo.toml` and
  `build.rs`

### Acceptance criteria

- the emitted crate can express native linker requirements through Cargo
- there is a tested path from `fol-build` native attachments to generated
  Cargo/build-script output

## Workstream E: Policy Gates Before Generation / Linking

### Why this matters

`fol` should not blindly generate or link if the `bic` evidence is weak or
contradictory.

### Deliverables

- explicit generation/link gating rules
- typed failure states for:
  - blocking diagnostics
  - insufficient ABI evidence
  - unresolved link requirements
  - ambiguous providers

### Proposed slices

- Slice E1: define the minimum acceptable `bic` evidence for generation
- Slice E2: define the minimum acceptable validation result for linking
- Slice E3: implement policy checks for blocking diagnostics and missing
  required layouts
- Slice E4: implement policy checks for unresolved or ambiguous link-plan
  entries
- Slice E5: add focused tests for accepted vs rejected native interop builds

### Acceptance criteria

- `fol` fails early and clearly when the native evidence is not good enough
- `fol` does not silently generate/link from weak or contradictory metadata

## Workstream F: First End-to-End Target

### Why this matters

The architecture is not proven until one real target successfully flows from
`bic` analysis through `fol` generation into a linked artifact.

### First target recommendation

Start with `zlib`.

Why:

- simpler than OpenSSL / libcurl
- common and stable
- already used as a stress/baseline target in `bic`
- enough to prove real named-library linking

After that, optionally prove a Linux system-header target such as SocketCAN
where link complexity is low but header/macros/layout evidence matter.

### Deliverables

- one end-to-end `zlib` proof
- one optional Linux/system proof after that

### Proposed slices

- Slice F1: create a minimal `fol` native-interop example target for `zlib`
- Slice F2: wire `bic` scan + validation + link-plan into that example
- Slice F3: make the backend emit a generated Rust crate that links `zlib`
- Slice F4: add an integration test that proves the final artifact builds
- Slice F5: optionally add a second Linux/system target such as SocketCAN or
  libc-backed sockets

### Acceptance criteria

- one real native dependency works end-to-end through `fol`
- the proof covers both metadata consumption and final linking

## Workstream G: Documentation and User Model

### Why this matters

Once this works, users need to understand the toolchain split clearly:
`bic` discovers, `fol` generates, Cargo/rustc link.

### Deliverables

- one build/book explanation of the full native pipeline
- one example-driven guide for code-only native interop
- explicit statement that `ldd` is diagnostic, not a build mechanism

### Proposed slices

- Slice G1: document the `bic -> fol -> Cargo -> rustc -> linker` pipeline
- Slice G2: document the new native attachment/build-script emission model
- Slice G3: document the first real `zlib` integration path
- Slice G4: document what remains downstream deployment/runtime policy

### Acceptance criteria

- users can understand how the pieces fit without guessing
- docs reflect the real architecture rather than idealized future behavior

## Recommended Order

1. Workstream A
2. Workstream B
3. Workstream C
4. Workstream D
5. Workstream E
6. Workstream F
7. Workstream G

This keeps the work staged correctly:

- first create the bridge
- then map the metadata
- then make the build graph carry it
- then make the backend emit it
- then add safety gates
- then prove one real target
- then document the final user-facing model

## Risk Notes

### Low Risk

- adding a `bic` bridge module
- lowering `bic` link metadata to `fol-build` native types
- backend generation of `build.rs`

### Medium Risk

- deciding how concrete native artifacts should be staged for Cargo builds
- keeping build outputs deterministic across machines
- expressing platform-conditioned link behavior cleanly

### Intentionally Deferred

- plugin loader runtime policy
- deployment-time search path fixing
- `dlopen` success proofs
- cross-platform breadth beyond Linux/ELF-first delivery

## Definition of Done

This plan is complete when:

1. `fol` can call `bic` directly from code
2. `fol` can lower `bic` link metadata into its build graph
3. the backend emits native link instructions through Cargo-compatible output
4. one real dependency such as `zlib` links end-to-end
5. docs clearly explain the architecture and remaining boundaries

## Progress

This is a new plan.

- total slices: 36
- completed: 0
- progress: 0%
