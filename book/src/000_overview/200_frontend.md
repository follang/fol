# Frontend Workflow

FOL now has a dedicated frontend layer above the compiler pipeline.

That layer is the `fol` tool itself.

The frontend is implemented in `fol-frontend`, and it is the user-facing
workflow shell for:

- project/workspace setup
- fetch/update flows
- build/run/test/emit flows
- editor tooling dispatch under `fol tool`

The detailed reference has moved to the Tooling section:

- [Tooling](../050_tooling/_index.md)
- [Frontend Workflow](../050_tooling/100_frontend.md)
- [Tool Commands](../050_tooling/200_tool_commands.md)

Use this overview page only as the entrypoint pointer.

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
