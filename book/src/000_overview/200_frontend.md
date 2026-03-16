# Frontend Workflow

FOL now has a dedicated frontend layer above the compiler pipeline.

That layer is the `fol` tool itself:

- `fol init`
- `fol new`
- `fol work info`
- `fol work list`
- `fol fetch`
- `fol check`
- `fol build`
- `fol run`
- `fol test`
- `fol emit rust`
- `fol emit lowered`
- `fol clean`
- `fol completion`

The frontend is implemented in the `fol-frontend` crate. It sits above:

- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-lower`
- `fol-backend`

## One Tool

The intended workflow is entrypoint-first:

```text
fol init --bin
fol fetch
fol check
fol build --release
fol run -- --flag value
```

The goal is to make `fol` feel like the canonical language tool, not just a
compiler executable with a growing list of flags.

## Root Discovery

For workspace commands, the frontend discovers roots upward from the current
directory or from an explicit path:

- package roots use `package.yaml`
- workspace roots use `fol.work.yaml`

If no root is found, frontend diagnostics explain how to bootstrap one with
`fol init --bin` or `fol init --workspace`.

## Output Modes

Frontend command summaries support:

- `human`
- `plain`
- `json`

`human` is for interactive usage. `plain` is for scripts. `json` is for
machine-readable tooling and wrappers.

## Build Artifacts

Frontend build commands report explicit artifact roots.

That includes:

- workspace and package roots
- build roots
- emitted Rust crate roots
- lowered snapshot roots
- final binary paths

This keeps the frontend closer to a build tool than to a thin compiler shell.

## Current Boundary

The current frontend milestone is about local workflows and the first backend.

It already covers:

- project and workspace scaffolding
- root discovery
- package preparation through `fol-package`
- full `V1` build/run/test orchestration
- emitted Rust and lowered IR output
- shell completions

Future work is still expected around:

- remote dependency fetching
- richer package-store management
- lockfile/version workflows
- additional backend targets
