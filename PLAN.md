# FOL Frontend Full Implementation Plan

Last updated: 2026-03-16

This plan replaces the closed frontend milestone record.

`fol-frontend` is already a usable workflow tool for the current `V1`
compiler/runtime/backend stack. What is still missing is the rest of the
package-management and workspace story:

- remote dependency fetching
- git-backed package locators
- a real package-store materialization contract
- lockfiles
- version/update workflows
- stronger workspace dependency reporting and repair workflows

This plan is about finishing that work so the frontend is not just a local
workflow shell, but a complete package-aware tool.

## Current State

What already exists:

- `clap` derive-based workflow CLI
- project/workspace scaffolding
- workspace/package discovery
- local package preparation over `fol-package`
- `check`, `build`, `run`, `test`, `emit`, `clean`, `completion`
- first backend integration
- colored human output and plain/json output

What is explicitly not implemented yet:

- real git fetch support
- remote locator parsing beyond placeholders
- package checkout/materialization into the store
- dependency lockfiles
- frontend commands for updating/pinning/frozen fetches
- dependency tree/status reporting

## Desired End State

At closeout, the frontend should support this kind of workflow end to end:

```text
fol new app --bin
fol fetch
fol work deps
fol build
fol run -- --flag value
fol update
fol fetch --locked
```

And package dependencies should support:

- installed store paths
- git URLs
- git SSH locators
- pinned revisions
- optional branch or tag selectors
- lockfile materialization to exact fetched revisions

The frontend should remain the user-facing tool, while:

- `fol-package` owns package metadata, locators, and materialization/loading
- `fol-frontend` owns workflow commands, summaries, lock/update UX, and policy

## Boundary

`fol-frontend` should own:

- command UX and orchestration
- workspace-wide fetch/update policies
- package-store root selection and cleanup policy
- lockfile loading/writing
- user-facing dependency summaries
- frozen/locked/offline behavior
- fetch/update diagnostics

`fol-package` should own:

- dependency locator parsing
- remote source identity
- fetch/materialize session logic
- checkout/cache/store layout
- manifest/lockfile data model handoff if shared there

## Package Source Model

The source kinds we want are:

- `loc`
- installed `pkg`
- remote `git`

Remote git source fields should cover:

- canonical URL
- optional transport-normalized identity
- optional branch
- optional tag
- optional revision
- package subdir if the repo contains multiple packages later

The rule should be:

- manifests may contain flexible selectors
- lockfiles contain exact pinned revisions

## Store And Cache Model

We need three distinct roots:

- source cache root
- materialized package-store root
- build/cache roots

Suggested semantics:

- source cache keeps git clones/fetch mirrors
- package store keeps exact checked-out package roots used by the compiler
- build/cache roots remain frontend-owned output/cache state

The package-store layout should be deterministic and revision-aware.

Example shape:

```text
.fol/
  cache/
    git/
      github.com/
        follang/
          tinylog.git/
  pkg/
    git/
      github.com/
        follang/
          tinylog/
            rev-<sha>/
  build/
```

## Lockfile

Add a workspace/package lockfile, for example:

- `fol.lock`

It should record:

- dependency logical name
- source kind
- normalized locator
- selected revision
- package version if declared
- materialized store path identity

Rules:

- `fol fetch` updates the lock if not frozen
- `fol fetch --locked` requires the lockfile to match
- `fol build --locked` and `fol run --locked` should honor the lock
- `fol update` is the workflow that intentionally refreshes git revisions

## New Command Surface

The current command tree stays, but we add:

- `fol update`
- `fol work deps`
- `fol work status`

And extend:

- `fol fetch --locked`
- `fol fetch --offline`
- `fol fetch --refresh`
- `fol build --locked`
- `fol run --locked`
- `fol test --locked`

## Output Expectations

Human mode should show:

- which packages were fetched
- whether they came from store or git
- whether the lockfile changed
- where packages were materialized
- what revision was selected

Plain/json should expose the same facts structurally.

## Test Fixture Direction

To prove this work, keep a tiny git-fetchable package outside the tracked tree.

Use:

- `xtra/logtiny`

That package should stay tiny and stable and can be pushed to git later for real
fetch tests.

The frontend test stack should cover:

- local fake git repos in temp dirs
- pinned revision fetch
- branch refresh
- lockfile generation
- locked mode mismatch failures
- offline mode with warm cache
- clean behavior preserving or pruning the right roots

## Phases

### Phase 0: Freeze The Missing Boundary

- `0.1` replace the closed frontend plan with the full implementation plan
- `0.2` freeze the split between frontend UX and package-session logic
- `0.3` freeze the source-cache vs package-store distinction
- `0.4` freeze lockfile goals and command semantics

### Phase 1: Shared Package Source Model

- `1.1` add git locator kinds in `fol-package`
- `1.2` parse HTTPS git locators
- `1.3` parse SSH git locators
- `1.4` parse `git+` locators
- `1.5` parse optional branch selectors
- `1.6` parse optional tag selectors
- `1.7` parse optional revision selectors
- `1.8` add normalized remote identity rendering
- `1.9` add explicit diagnostics for malformed git locators
- `1.10` lock locator parsing with unit tests

### Phase 2: Manifest Dependency Expansion

- `2.1` extend package metadata to carry dependency declarations
- `2.2` support source-qualified dependency entries
- `2.3` support package aliases/logical names
- `2.4` reject mixed invalid dependency forms clearly
- `2.5` lock manifest parsing for local/pkg/git dependencies

### Phase 3: Source Cache And Store Layout

- `3.1` add frontend/package config fields for git cache roots
- `3.2` define deterministic cache paths for git remotes
- `3.3` define deterministic store paths for pinned revisions
- `3.4` separate materialized package roots from source mirrors
- `3.5` add tests for root/path determinism

### Phase 4: Git Materialization In `fol-package`

- `4.1` add a git source session shell
- `4.2` clone/fetch into the source cache
- `4.3` materialize pinned revisions into the package store
- `4.4` load materialized packages through existing package loading
- `4.5` surface exact selected revisions
- `4.6` wrap git failures as package errors
- `4.7` lock materialized git package loading with tests

### Phase 5: Lockfile Model

- `5.1` add a lockfile data model
- `5.2` add stable lockfile serialization
- `5.3` add stable lockfile parsing
- `5.4` add lockfile identity hashing/versioning
- `5.5` lock lockfile roundtrips with tests

### Phase 6: Fetch Command Completion

- `6.1` make `fol fetch` resolve manifest dependencies
- `6.2` make `fol fetch` materialize git dependencies
- `6.3` write `fol.lock` after successful fetch
- `6.4` surface fetched package summaries in human/plain/json
- `6.5` add `--locked`
- `6.6` add `--offline`
- `6.7` add `--refresh`
- `6.8` add fetch integration tests over temp git repos

### Phase 7: Locked Build/Run/Test

- `7.1` make `check` honor `--locked`
- `7.2` make `build` honor `--locked`
- `7.3` make `run` honor `--locked`
- `7.4` make `test` honor `--locked`
- `7.5` fail clearly when manifests and lockfile disagree
- `7.6` lock locked-mode integration behavior

### Phase 8: Update Workflow

- `8.1` add `fol update` command shell
- `8.2` update git dependencies to latest allowed refs
- `8.3` refresh lockfile after update
- `8.4` report revision changes clearly
- `8.5` add integration tests for update flows

### Phase 9: Workspace Dependency Reporting

- `9.1` add `fol work deps`
- `9.2` add `fol work status`
- `9.3` show member package dependency graphs
- `9.4` show lockfile and fetch status
- `9.5` add integration tests for workspace dependency reporting

### Phase 10: Store Hygiene And Repair

- `10.1` teach `clean` about git source cache roots
- `10.2` keep safe boundaries for external/shared stores
- `10.3` add stale materialization pruning helpers
- `10.4` add optional repair workflow for missing materialized revisions
- `10.5` add integration tests for cleanup/repair boundaries

### Phase 11: UX Hardening

- `11.1` add guidance notes for auth/network/git failures
- `11.2` add locked/offline guidance notes
- `11.3` add manifest-vs-lock mismatch guidance notes
- `11.4` add stable JSON error structures for fetch/update failures
- `11.5` lock human/plain/json diagnostics

### Phase 12: Real Fixture Coverage

- `12.1` turn `xtra/logtiny` into a real remote-fetch test package later
- `12.2` add temp-repo helper utilities in frontend integration tests
- `12.3` add single-package git fetch fixture coverage
- `12.4` add workspace multi-member git fetch coverage
- `12.5` add pinned revision regression coverage
- `12.6` add offline warm-cache coverage

### Phase 13: Docs Closeout

- `13.1` update the frontend book chapter
- `13.2` update README frontend/package workflow docs
- `13.3` update PROGRESS frontend status
- `13.4` close the plan only after git fetch, lockfile, and update are real

## Definition Of Done

Do not close this plan until all of this is true:

- git locators are accepted and validated
- `fol fetch` can clone/fetch/materialize real git dependencies
- `fol.lock` is written and read
- `--locked` works on fetch/build/run/test
- `fol update` works
- workspace dependency reporting is real
- cleanup respects cache/store safety
- the frontend docs describe the implemented workflow accurately

At that point, `fol-frontend` can be called fully implemented for the complete
first-generation package/workspace workflow.
