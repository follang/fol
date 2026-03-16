# FOL Frontend Full Implementation Plan

Status: closed
Closed on: 2026-03-16

This milestone is complete.

`fol-frontend` now provides the full first-generation package/workspace workflow
above `fol-package`, the compiler pipeline, and the first backend.

## What Landed

- derive-based workflow CLI with aliases, grouped help, and frontend-owned root help
- workspace/package discovery and scaffolding
- `fetch`, `update`, `check`, `build`, `run`, `test`, `emit`, `clean`, and `completion`
- `work info`, `work list`, `work deps`, and `work status`
- git locator parsing through `fol-package`
- git source-cache and materialized package-store layout
- `fol.lock` writing, parsing, and locked-mode validation
- `--locked`, `--offline`, and `--refresh`
- update flows with revision-change reporting
- stale materialization pruning and missing pinned-revision repair
- safe cleanup boundaries for build/cache/git/package-store roots
- human/plain/json diagnostics with fetch/update guidance
- temp-repo git integration coverage plus an ignored public GitHub fixture for
  `https://github.com/bresilla/logtiny`

## Definition Of Done

The intended closeout conditions are now true:

- git locators are accepted and validated
- `fol fetch` can clone/fetch/materialize real git dependencies
- `fol.lock` is written and read
- `--locked` works on fetch/build/run/test
- `fol update` works
- workspace dependency reporting is real
- cleanup respects cache/store safety
- the frontend docs describe the implemented workflow accurately

## Next Work

Frontend/package-manager follow-on work, if reopened later, should be about:

- version solving beyond pinned git revisions
- richer shared/external package-store policy
- registry support
- future backend-aware workflow expansion
