# Frontend Workflow

FOL now has a dedicated frontend layer above the compiler pipeline.

That layer is the `fol` tool itself.

The frontend is implemented in the `fol-frontend` crate. It sits above:

- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-lower`
- `fol-backend`

Its job is not to replace those crates. Its job is to orchestrate them into one
coherent user tool.

## What The Frontend Owns

The frontend owns:

- derive-based `clap` command parsing
- command aliases and grouped help
- frontend-owned root help and version output
- workspace and package discovery
- package/workspace scaffolding
- package preparation over `fol-package`
- build/run/test orchestration
- emit workflows
- clean workflows
- shell completions
- human/plain/json output
- user-facing summaries and diagnostics

So the frontend is the workflow shell.

It is not another compiler stage.

## Command Surface

The current command surface is:

- `fol work init`
- `fol work new`
- `fol work info`
- `fol work list`
- `fol work deps`
- `fol work status`
- `fol pack fetch`
- `fol pack update`
- `fol code check`
- `fol code build`
- `fol code run`
- `fol code test`
- `fol code emit rust`
- `fol code emit lowered`
- `fol tool lsp`
- `fol tool parse <PATH>`
- `fol tool highlight <PATH>`
- `fol tool symbols <PATH>`
- `fol tool clean`
- `fol tool completion`
- hidden `_complete`

The root help contract is:

- `fol`
- `fol --help`

Both render the same frontend-owned help surface.

Aliases are part of the tool contract too. Examples:

- `fol code build`, `fol code b`, `fol code make`
- `fol pack fetch`, `fol pack f`, `fol pack sync`
- `fol tool clean`, `fol tool cl`, `fol tool purge`
- `fol code check`, `fol code c`, `fol code verify`
- root groups use one-letter aliases only:
  - `fol work`, `fol w`
  - `fol pack`, `fol p`
  - `fol code`, `fol c`
  - `fol tool`, `fol t`

## One Tool

The intended workflow is entrypoint-first:

```text
fol work init --bin
fol pack fetch
fol code check
fol code build --release
fol code run -- --flag value
fol tool symbols src/main.fol
```

The goal is to make `fol` feel like the canonical language tool, not just a
compiler executable with a growing list of flags.

Direct compile is still supported too, but it is frontend-owned now:

```text
fol path/to/package
fol code build path/to/package
fol code check path/to/package
fol code run path/to/package -- --flag value
fol code emit rust path/to/package
fol code emit lowered path/to/package
```

## How Dispatch Works

The frontend flow is:

1. parse CLI arguments with `clap`
2. resolve output/profile policy
3. detect the target root
4. load the frontend workspace model
5. dispatch the selected command
6. call down into package/compiler/backend layers as needed
7. render a frontend-owned command summary or frontend-owned error

For example:

- `fol code check` loads the workspace and drives the compile pipeline through
  typecheck/lower without backend artifact production
- `fol code build` drives the full compiler and backend path
- `fol code run` builds first, then executes the produced binary
- `fol code emit rust` keeps the backend in source-emission mode
- `fol code emit lowered` writes lowered IR snapshots instead of invoking the backend
- `fol tool ...` dispatches editor parsing, query, and LSP flows into `fol-editor`

Compile-oriented flags belong to the commands that use them. For example:

- `--std-root` and `--package-store-root` belong to compile/fetch flows
- `--keep-build-dir` belongs to backend-producing flows such as `code build`,
  `code run`, and `code emit rust`
- `dump lowered` is represented as `fol code emit lowered`, not as a root flag in the
  public CLI

## Root Discovery

For workflow commands, the frontend discovers roots upward from the current
directory or from an explicit path:

- package roots use `package.yaml`
- workspace roots use `fol.work.yaml`

Package roots and workspace roots are different frontend concepts.

A package root is one runnable/buildable package.

A workspace root is a parent root that owns multiple member packages plus
frontend-owned roots such as build and cache directories.

If no root is found, frontend diagnostics explain how to bootstrap one with
`fol work init --bin` or `fol work init --workspace`.

## Configuration And Precedence

The frontend currently supports environment and flag control for:

- output mode
- profile
- std root
- package store root
- build root
- cache root
- keep-build-dir

The intended precedence is:

1. explicit CLI flags
2. workspace-owned config where applicable
3. frontend environment variables
4. frontend defaults

That precedence is test-backed.

## Output Modes

Frontend command summaries support:

- `human`
- `plain`
- `json`

`human` is for interactive usage.

`plain` is for scripts.

`json` is for machine-readable tooling and wrappers.

The frontend also owns human highlighting behavior. Actions and paths can be
highlighted in human mode, while plain mode stays stable and ANSI-free.

Human mode is always colorized. There is no public color-policy switch.

## Editor Tooling Entry

The frontend also owns the public entrypoint for editor tooling.

That command family is:

- `fol tool lsp`
- `fol tool parse <PATH>`
- `fol tool highlight <PATH>`
- `fol tool symbols <PATH>`

The frontend does not implement Tree-sitter parsing or LSP semantics itself. It
parses the command, applies output policy, dispatches into `fol-editor`, and
renders the result or error in human/plain/json.

This keeps editor workflows under the same `fol` binary instead of introducing
a second public tool.

## Build Artifacts

Frontend commands report explicit artifact roots.

That includes:

- workspace roots
- package roots
- build roots
- emitted Rust crate roots
- lowered snapshot roots
- final binary paths

This keeps the frontend closer to a build tool than to a thin compiler shell.

## Relationship To The Root Binary

The repo still has a root `src/main.rs`, but it is now just a thin entry shim
into `fol-frontend`.

The CLI behavior itself lives in `fol-frontend`.

That includes:

- workflow commands
- direct compile dispatch
- root help
- output rendering
- frontend diagnostics

So the root binary is no longer its own separate CLI implementation.

## Current Boundary

The current frontend milestone is about local workflows and the first backend.

It already covers:

- project and workspace scaffolding
- root discovery
- package preparation through `fol-package`
- git-backed dependency fetching and materialization
- `fol.lock` writing, locked fetches, offline warm-cache fetches, and update flows
- workspace dependency/status reporting
- full `V1` build/run/test orchestration
- emitted Rust and lowered IR output
- editor-tooling entrypoints for parse, highlight, symbols, and LSP startup
- shell completions
- safe cleanup of build/cache/git/package-store roots
- frontend-owned direct compile routing

Future work is still expected around:

- richer package-store policy beyond the first git/store workflow
- lockfile/version solving beyond the current pinned git contract
- additional backend targets
