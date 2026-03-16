# FOL Frontend Plan

Last updated: 2026-03-16

Phase 0 status:

- [x] 0.1 Create `fol-frontend` plan reset commit
- [x] 0.2 Freeze `fol-frontend` as a crate, not root-main sprawl

This plan defines the next major milestone after the completed `V1` compiler
pipeline:

- `fol-stream`
- `fol-lexer`
- `fol-parser`
- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-lower`
- `fol-runtime`
- `fol-backend`

The next missing piece is the user-facing shell over all of that:

- command-line UX
- workspace creation
- package fetching and install/materialization
- build/run/test entry flows
- emitted artifact handling
- consistent human output
- structured machine output

That shell should live in a new crate:

- `fol-frontend`

This plan intentionally treats `fol-frontend` as a first-class crate, not just
"make `src/main.rs` larger".

The current root binary remains temporary migration surface only. New frontend
logic must land in `fol-frontend`, with root `src/main.rs` reduced to a shim as
Phase 17 progresses.

## 0. Goal

`fol-frontend` should become the canonical user entrypoint for FOL developer
workflows.

That means:

- creating a project
- creating a workspace
- resolving and fetching dependencies
- preparing installed package roots through `fol-package`
- building binaries through the full compiler pipeline
- running binaries
- testing packages/workspaces
- printing useful diagnostics and progress

It should feel closer to `zig` than to a thin compiler driver.

## 1. Design Direction

The frontend should borrow the *workflow shape* of modern Zig:

- one obvious top-level binary
- clear init/build/run/test commands
- package/workspace preparation owned by the frontend layer
- reproducible build roots
- useful emitted artifacts
- tool-managed build/cache directories

But it should still stay idiomatic to this repo:

- Rust implementation
- `clap` with derive
- split command modules
- explicit output helpers
- strong integration tests

The local `../../tools/roc` CLI is the immediate reference for:

- command/alias density
- visible aliases
- custom help presentation
- human/plain/json output helpers
- completion plumbing
- command module organization

Unlike `roc`, this crate should use:

- `clap` derive macros for command/arg definitions

## 2. Boundary

`fol-frontend` owns:

- CLI argument parsing
- command dispatch
- output modes and color policy
- workspace discovery
- workspace root creation
- package fetch/install orchestration
- frontend config loading
- frontend-facing build roots and cache roots
- calls into `fol-package`
- calls into the full compile pipeline
- calls into `fol-backend`
- command-level progress and summaries
- shell completions

`fol-frontend` does **not** own:

- lexing/parsing
- resolution
- typechecking
- lowering
- backend code generation logic
- runtime representations
- actual package metadata parsing rules

Those stay where they already belong.

## 3. Relationship To Existing Crates

### 3.1 `fol-package`

`fol-frontend` talks to `fol-package` for:

- workspace/package discovery
- package manifest/build-file semantics
- dependency preparation
- installed package root materialization
- package-store root management
- future fetch/install/lock operations

The frontend should never re-implement package graph logic.

### 3.2 Compiler Pipeline

For compile/build/run/test flows, `fol-frontend` calls:

- `fol-package`
- `fol-resolver`
- `fol-typecheck`
- `fol-lower`
- `fol-backend`

It should do this through explicit orchestration APIs, not by embedding logic
inline into command handlers.

### 3.3 Binary Entry

The final top-level `fol` user binary should be frontend-owned.

That means the repo eventually wants:

- `fol-frontend` as the real app crate
- current root `src/main.rs` reduced to a thin shim or removed once migration is complete

## 4. UX Principles

### 4.1 Command Shape

The CLI should have:

- short primary nouns
- clear aliases
- obvious defaults
- one command per workflow, not per internal stage

Initial command families should be:

- `init`
- `new`
- `work`
- `fetch`
- `build`
- `run`
- `test`
- `check`
- `emit`
- `clean`
- `completion`
- internal `_complete`

### 4.2 Aliases

Like `roc`, command aliases should be dense and practical.

Examples:

- `build`
  - aliases: `b`, `make`
- `run`
  - aliases: `r`
- `test`
  - aliases: `t`
- `check`
  - aliases: `c`, `verify`
- `completion`
  - aliases: `completions`, `comp`
- `work`
  - aliases: `w`, `ws`, `workspace`

Flag aliases should also exist where they make sense, especially for:

- output roots
- package store roots
- std roots
- keep-build-dir
- release/debug mode
- JSON output

### 4.3 Output Modes

Frontend output should support:

- `human`
- `plain`
- `json`

`human`:

- colors
- symbols
- sectioned status output
- progress summaries

`plain`:

- stable scripting-friendly text
- no ANSI

`json`:

- structured command outputs
- structured failures
- stable machine-readable artifact reports

### 4.4 Color Policy

Use color intentionally, not everywhere.

Recommended:

- command headers
- success/failure summaries
- action labels
- path highlights
- warnings/notes

Respect:

- `--color auto|always|never`
- non-TTY output
- plain/json modes

`colored` or `anstyle`-based styling is acceptable.
The key requirement is a single frontend-owned style policy.

### 4.5 Help Design

Do not accept default clap help as the final UX.

The help should include:

- short program description
- grouped command lists
- visible aliases
- common examples
- environment/config notes where relevant

The `roc` CLI structure is the local UX reference here, even though we will use
derive instead of builder-style definitions.

## 5. Workspace Model

The frontend needs a concrete workspace model.

At minimum, `V1` should support:

- single-package project roots
- multi-package workspace roots
- local package membership
- shared build/cache directories
- explicit package-store root

Recommended user-visible files:

- `fol.work.yaml`
  - workspace root file
- per-package:
  - `package.yaml`
  - `build.fol`

The exact file name can change later, but the frontend must choose one and
standardize on it.

### 5.1 Workspace Discovery

Starting from cwd or explicit path:

- walk upward to find `fol.work.yaml`
- if none exists, treat a folder with `package.yaml` as a single-package root
- otherwise fail with a clear "not a FOL workspace/package root" diagnostic

### 5.2 Workspace Contents

The frontend workspace model should track:

- workspace root
- member package roots
- std root override
- package store root override
- build output root
- cache root
- target/profile

## 6. Init And New

The frontend should support:

### 6.1 `fol init`

Initialize the current directory as:

- package
- or workspace root

Responsibilities:

- create `package.yaml`
- create `build.fol`
- create starter source tree
- optionally create `fol.work.yaml`
- optionally create `.gitignore`

### 6.2 `fol new <name>`

Create a new directory and scaffold:

- package mode
- workspace mode

Options:

- `--workspace`
- `--bin`
- `--lib`
- `--name`
- `--path`

The generated templates should match current `V1`, not future language ideas.

## 7. Fetch / Install / Materialize

This is where `fol-frontend` and `fol-package` meet most directly.

### 7.1 `fol fetch`

Responsibilities:

- read workspace/package metadata
- resolve dependency locators
- fetch git packages
- materialize installed package roots into the package store
- print what changed

### 7.2 Package Store

The frontend should own the *user-facing* package store policy:

- default store location
- override via flag
- override via env/config
- reporting current store root

`fol-package` should still own the *semantic* preparation of packages.

### 7.3 Locking

This plan should prepare for:

- lockfile generation
- lockfile update
- reproducible fetch

Even if the first implementation stops short of a full solver.

## 8. Build / Check / Run / Test

### 8.1 `fol check`

Fast validation without producing a final binary.

Should run:

- `fol-package`
- resolver
- typecheck
- lowering
- maybe backend emission verification only if needed

### 8.2 `fol build`

Produces artifacts.

Responsibilities:

- select target package
- choose entry routine
- choose profile/debug/release
- run the full pipeline
- produce binary
- report binary path

### 8.3 `fol run`

Build then execute.

Responsibilities:

- support `--` passthrough args to the built binary
- optionally reuse an already-built artifact if inputs are unchanged
- print build vs run phases clearly

### 8.4 `fol test`

For `V1`, this should support:

- package/workspace compile tests
- executing backend-built test binaries where applicable
- later integration with test blocks if/when frontend test enumeration is ready

### 8.5 `fol emit`

Backend-facing developer command.

Examples:

- `fol emit rust`
- `fol emit lowered`
- `fol emit diagnostics`

This should sit above current `--emit-rust` / `--dump-lowered` style flags and
eventually replace the ad hoc flag surface.

## 9. Command Tree

Initial recommended tree:

- `fol init`
- `fol new`
- `fol work info`
- `fol work list`
- `fol work graph`
- `fol fetch`
- `fol check`
- `fol build`
- `fol run`
- `fol test`
- `fol emit rust`
- `fol emit lowered`
- `fol clean`
- `fol completion`
- `fol _complete`

Potential later additions:

- `fol add`
- `fol remove`
- `fol update`
- `fol fmt`
- `fol doc`

Those are not part of this first milestone unless they become necessary for
workspace usability.

## 10. Clap Derive Requirements

Use:

- `#[derive(Parser)]`
- `#[derive(Subcommand)]`
- `#[derive(Args)]`
- `#[derive(ValueEnum)]`

Rules:

- one derive type per command family
- subcommands live in dedicated modules
- shared arg groups live in reusable `Args` structs
- aliases and visible aliases are declared in derive metadata
- help text stays local to the command type, not scattered

Recommended module split:

- `fol-frontend/src/lib.rs`
- `fol-frontend/src/main.rs` or binary shim later
- `fol-frontend/src/cli/mod.rs`
- `fol-frontend/src/cli/root.rs`
- `fol-frontend/src/cli/commands/*.rs`
- `fol-frontend/src/output/*.rs`
- `fol-frontend/src/workspace/*.rs`
- `fol-frontend/src/dispatch/*.rs`

## 11. Roc CLI Ideas To Reuse

Reuse conceptually from `../../tools/roc`:

- grouped command organization
- command aliases
- visible aliases
- custom output helper modules
- plain/json split
- completion command plus internal `_complete`
- UI-specific helper layer instead of printing directly in every command

Do **not** copy blindly:

- the builder-style clap setup
- the exact command names
- ROS-specific output conventions

Translate the useful patterns into FOL’s command model.

## 12. Zig-Inspired Frontend Behavior

Important Zig-like ideas to borrow:

- a canonical top-level tool
- obvious init/build/run/test flows
- explicit fetch/materialization commands
- generated workspace/project scaffolding
- clean distinction between source roots and build/cache roots
- emitted artifact visibility
- support for inspectable generated intermediate output

Important Zig ideas **not** to cargo-cult immediately:

- fully programmable build graph in the frontend layer
- overloading `build.fol` execution semantics before needed
- backend-specific option explosion too early

`build.fol` stays package/build metadata owned by `fol-package`.
`fol-frontend` orchestrates; it does not become a second build-language engine.

## 13. Config And Environment

The frontend should support config from:

- CLI flags
- workspace file
- env vars
- sensible defaults

Initial config surfaces:

- `--std-root`
- `--package-store-root`
- `--build-root`
- `--cache-root`
- `--profile debug|release`
- `--color auto|always|never`
- `--output human|plain|json`

Potential env vars:

- `FOL_STD_ROOT`
- `FOL_PACKAGE_STORE_ROOT`
- `FOL_BUILD_ROOT`
- `FOL_CACHE_ROOT`
- `FOL_COLOR`

## 14. Artifact Model

Frontend command outputs should be explicit about produced artifacts.

For build/run/test flows, report:

- workspace root
- selected package
- selected entry
- generated crate root
- compiled binary path
- kept build dir path if applicable
- exit code

JSON output should make these first-class fields, not human-parsed strings.

## 15. Diagnostics Integration

The frontend must unify command failures into structured diagnostics.

That includes:

- package errors
- compiler diagnostics
- backend build failures
- frontend config/discovery errors
- fetch/install failures

Frontend-only failures should still use the diagnostics stack where possible.

## 16. Completions

The frontend should own:

- `fol completion bash`
- `fol completion zsh`
- `fol completion fish`
- internal dynamic completion command:
  - `fol _complete`

This is where the `roc` pattern is directly reusable:

- shell generation command
- dynamic completion backend for workspace-aware values

Later dynamic completion candidates:

- workspace packages
- commands/subcommands
- output profiles
- known roots
- member names

## 17. Real Test Coverage

The frontend needs more than parser-style tests.

It should have:

- help output tests
- alias tests
- JSON output tests
- completion tests
- workspace creation tests
- fetch/materialization tests
- build/run tests
- failure-path tests

Recommended test files:

- `tests/help_output_integration.rs`
- `tests/frontend_execution_integration.rs`
- `tests/workspace_init_integration.rs`
- `tests/workspace_fetch_integration.rs`
- `tests/json_output_integration.rs`
- `tests/completion_integration.rs`

## 18. Migration Strategy

Migration should happen in stages:

1. Create `fol-frontend` crate beside the current root CLI.
2. Move current root CLI argument handling into frontend-owned orchestration.
3. Keep root `fol` binary as a thin shim calling `fol_frontend::run()`.
4. Replace old ad hoc flags with command/subcommand structure.
5. Gradually deprecate raw compiler-only flags in favor of frontend commands.

Do not break existing power-user flows abruptly if a compatibility shim is easy.

## 19. Initial Non-Goals

Not part of this first frontend milestone:

- `V2`/`V3` language work
- LLVM backend selection
- C ABI frontend workflows
- a full package solver
- docs generation frontend
- formatter frontend
- IDE/LSP work

Those can come later once the basic frontend is stable.

## 20. Phases

### Phase 0. Direction Freeze

0.1 Create `fol-frontend` plan reset commit
0.2 Freeze `fol-frontend` as a crate, not root-main sprawl
0.3 Freeze `clap` derive requirement
0.4 Freeze `roc` UX patterns to reuse conceptually
0.5 Freeze Zig-style workflow targets for `init/build/run/test/fetch`

### Phase 1. Crate Foundation

1.1 Add `fol-frontend` workspace crate
1.2 Add public `run()` shell
1.3 Add structured frontend error model
1.4 Add output mode enum and color policy shell
1.5 Add frontend config model
1.6 Add command result/artifact summary model

### Phase 2. CLI Shape

2.1 Add derive-based root parser
2.2 Add root command families
2.3 Add aliases and visible aliases for root commands
2.4 Add grouped help presentation
2.5 Add `--output`
2.6 Add `--color`
2.7 Add profile/debug/release selection
2.8 Add CLI smoke and help snapshot tests

### Phase 3. Output/UI Helpers

3.1 Add frontend output helper module
3.2 Add human output helpers
3.3 Add plain output helpers
3.4 Add JSON output helpers
3.5 Add command summary rendering
3.6 Add color auto-detection
3.7 Add output compatibility tests

### Phase 4. Workspace Discovery

4.1 Add workspace root discovery model
4.2 Add package-root discovery model
4.3 Add cwd-upward root search
4.4 Add explicit path selection
4.5 Add frontend diagnostics for missing workspace/package roots
4.6 Add workspace discovery tests

### Phase 5. Init/New

5.1 Add `fol init`
5.2 Add current-directory package scaffolding
5.3 Add current-directory workspace scaffolding
5.4 Add `fol new <name>`
5.5 Add `--workspace`
5.6 Add `--bin`
5.7 Add `--lib`
5.8 Add starter source templates
5.9 Add starter `package.yaml`
5.10 Add starter `build.fol`
5.11 Add init/new integration tests

### Phase 6. Frontend Workspace Model

6.1 Add frontend workspace struct
6.2 Add member-package enumeration
6.3 Add workspace root config loading
6.4 Add std/package-store override loading
6.5 Add build/cache root policy
6.6 Add workspace info summary rendering
6.7 Add `work info`
6.8 Add `work list`
6.9 Add workspace-model tests

### Phase 7. Package Preparation / Fetch

7.1 Add frontend package preparation shell over `fol-package`
7.2 Add `fetch` command shell
7.3 Add package store root selection
7.4 Add fetch/materialize summaries
7.5 Add fetch diagnostics wrapping
7.6 Add frontend environment-variable support
7.7 Add fetch integration tests with local git/package fixtures

### Phase 8. Check/Build/Run

8.1 Add `check` command orchestration
8.2 Add `build` command orchestration
8.3 Add `run` command orchestration
8.4 Add profile-aware build output roots
8.5 Add build artifact summaries
8.6 Add binary arg passthrough for `run -- ...`
8.7 Add keep-build-dir integration
8.8 Add command integration tests for check/build/run

### Phase 9. Test Command

9.1 Add `test` command shell
9.2 Add package selection for test
9.3 Add workspace-wide test traversal
9.4 Add frontend result summaries
9.5 Add JSON test result shape
9.6 Add test command integration coverage

### Phase 10. Emit Commands

10.1 Add `emit` root subcommand
10.2 Add `emit rust`
10.3 Add `emit lowered`
10.4 Add artifact-path reporting for emit commands
10.5 Add emit integration tests

### Phase 11. Clean Command

11.1 Add `clean` command
11.2 Add build-root cleanup
11.3 Add cache-root cleanup policy
11.4 Add package-store safety boundaries
11.5 Add clean integration tests

### Phase 12. Completions

12.1 Add `completion` command
12.2 Add shell script generation for bash
12.3 Add shell script generation for zsh
12.4 Add shell script generation for fish
12.5 Add internal `_complete` command
12.6 Add dynamic completion hooks for command tree
12.7 Add completion integration tests

### Phase 13. Config / Env Hardening

13.1 Add env var loading
13.2 Add flag-over-env precedence tests
13.3 Add workspace-config-over-env precedence tests
13.4 Add output/color/profile precedence tests

### Phase 14. Diagnostics Integration

14.1 Add frontend-only diagnostic lowering
14.2 Add build/fetch/init/run failure wrapping
14.3 Add JSON diagnostic compatibility for frontend failures
14.4 Add human guidance notes for common workspace mistakes
14.5 Add diagnostics integration tests

### Phase 15. Roc-Style UX Hardening

15.1 Add visible aliases broadly
15.2 Add command examples in help
15.3 Add grouped command sections in help
15.4 Add path and action highlighting in human mode
15.5 Add stable plain-mode output for scripts
15.6 Add help output snapshot coverage

### Phase 16. Zig-Style Workflow Hardening

16.1 Add `init/new/build/run/test/fetch` happy-path walkthrough tests
16.2 Add generated workspace roots that feel like one canonical tool flow
16.3 Add explicit artifact root reporting similar to Zig build feedback
16.4 Add frontend docs that explain the workflow in one entrypoint-first story

### Phase 17. CLI Migration

17.1 Route current root binary through `fol-frontend`
17.2 Keep temporary compatibility with existing direct flags where feasible
17.3 Remove old root-main orchestration duplication
17.4 Add migration regression tests

### Phase 18. Docs Closeout

18.1 Sync `README.md`
18.2 Sync `PROGRESS.md`
18.3 Sync relevant book command/tooling chapters
18.4 Rewrite `PLAN.md` as a completion record

## 21. Definition Of Done

`fol-frontend` is done for this milestone when all of the following are true:

- there is a dedicated `fol-frontend` crate
- the top-level `fol` user flow runs through it
- argument parsing uses `clap` derive
- command aliases and visible aliases are real and tested
- `init`, `new`, `fetch`, `check`, `build`, `run`, `test`, `emit`, `clean`, and `completion` exist
- frontend orchestrates package preparation through `fol-package`
- frontend emits useful human/plain/json outputs
- workspace discovery and root policy are explicit
- generated artifacts and build roots are reported clearly
- completion generation exists
- real integration tests cover workspace creation, build/run, and failure diagnostics

At that point, `fol` stops feeling like "a compiler binary with flags" and
starts feeling like the actual user tool for the language.
