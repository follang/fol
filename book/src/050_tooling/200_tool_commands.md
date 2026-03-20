# Tool Commands

This chapter lists the current frontend surface by workflow area.

## Work

Project and workspace commands:

- `fol work init`
- `fol work new`
- `fol work info`
- `fol work list`
- `fol work deps`
- `fol work status`

Examples:

```text
fol work init --bin
fol work init --workspace
fol work new demo --lib
fol work info
fol work deps
```

Use `work` for:

- creating package/workspace roots
- inspecting workspace structure
- seeing member and dependency state

## Pack

Package acquisition commands:

- `fol pack fetch`
- `fol pack update`

Examples:

```text
fol pack fetch
fol pack fetch --locked
fol pack fetch --offline
fol pack fetch --refresh
fol pack update
```

Use `pack` for:

- materializing dependencies
- writing or honoring `fol.lock`
- refreshing pinned git dependencies

## Code

Build-oriented commands:

- `fol code check`
- `fol code build`
- `fol code run`
- `fol code test`
- `fol code emit rust`
- `fol code emit lowered`

Examples:

```text
fol code check
fol code build --release
fol code run -- --flag value
fol code emit rust
fol code emit lowered
```

Use `code` for:

- driving the compile pipeline
- building binaries through the current Rust backend
- running produced binaries
- emitting backend/debug artifacts

## Tool

Tooling commands:

- `fol tool lsp`
- `fol tool parse <PATH>`
- `fol tool highlight <PATH>`
- `fol tool symbols <PATH>`
- `fol tool semantic-tokens <PATH>`
- `fol tool tree generate <PATH>`
- `fol tool clean`
- `fol tool completion`

Examples:

```text
fol tool parse src/main.fol
fol tool highlight src/main.fol
fol tool symbols src/main.fol
fol tool semantic-tokens src/main.fol
fol tool tree generate /tmp/fol
fol tool lsp
fol tool completion bash
```

Use `tool` for:

- editor integration
- Tree-sitter debugging
- LSP serving
- generated tool assets

The public editor surface stays under `fol tool ...`.
There is no parallel `fol editor ...` command group.
Future editor features are not exposed as placeholder commands.
Only the shipped `fol tool` subcommands above are public.

## Artifact Reporting

Frontend commands report explicit artifact roots when applicable, including:

- emitted Rust crate roots
- lowered snapshot roots
- final binary paths
- fetch/store/cache roots where relevant
