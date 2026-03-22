# Replace clap with manual arg parsing

Last updated: 2026-03-22

## Goal

Remove `clap`, `clap_derive`, `clap_complete` and their 14 transitive crates by
replacing the derive-based CLI parser with a hand-written argument parser. This
cuts the external dependency count from ~24 to ~10 (serde ecosystem only).

## Crates eliminated

clap, clap_builder, clap_derive, clap_lex, clap_complete, anstyle,
anstyle-parse, anstyle-query, anstream, colorchoice, is_terminal_polyfill,
strsim, utf8parse, heck

## Constraints

- All ~30 existing CLI tests must keep passing (adapted to new parser)
- All ~1700+ workspace tests must keep passing
- Env var defaults (FOL_OUTPUT, FOL_PROFILE) must still work
- Conflict detection (--debug/--release/--profile, --bin/--lib) must still work
- Aliases (w→work, p→pack, c→code, t→tool, b→build, etc.) must still work
- Shell completion scripts (bash/zsh/fish) must still be generated
- Internal completion (_complete) must still work
- Help text and version display must still work
- `--` separator for passthrough args in `run` must still work

## Files touched

- `lang/tooling/fol-frontend/Cargo.toml` — remove clap, clap_complete deps
- `lang/tooling/fol-frontend/src/output.rs` — remove `clap::ValueEnum` derive
- `lang/tooling/fol-frontend/src/cli/args.rs` — remove all clap derives, keep plain structs
- `lang/tooling/fol-frontend/src/cli/parser.rs` — replace with hand-written parser
- `lang/tooling/fol-frontend/src/cli/tests.rs` — adapt to new parser API
- `lang/tooling/fol-frontend/src/completion.rs` — replace clap_complete with hand-written scripts
- `lang/tooling/fol-frontend/src/dispatch.rs` — remove clap error kind handling
- `lang/tooling/fol-frontend/src/lib.rs` — update parse error mapping

---

## Phase 1: Strip clap derives from data structs

Remove all `#[derive(Args)]`, `#[derive(Subcommand)]`, `#[derive(Parser)]`,
`#[derive(ValueEnum)]`, `#[arg(...)]`, `#[command(...)]` attributes. Keep the
structs and enums exactly as they are — they become plain Rust types.

Files: `args.rs`, `output.rs`

## Phase 2: Write the manual parser

Replace `parser.rs` with a hand-written `parse(args: &[String]) -> Result<FrontendCli, ParseError>`.

The parser is a simple state machine:
1. Consume global flags from the front (--output, --json, --profile, etc.)
2. Match first positional to a command group (work/w, pack/p, code/c, tool/t, _complete)
3. If no group match, treat it as a direct file/folder input
4. Within each group, consume group-level flags (--output, --profile for code)
5. Match next positional to subcommand (build/b/make, run/r, etc.)
6. Within subcommand, consume subcommand-specific flags
7. Remaining positionals become positional args (path, name, etc.)

Must handle:
- `--flag value` and `--flag=value` forms
- `-D value` short flag with append semantics
- `--` separator for trailing args
- Env var fallbacks for `--output` and `--profile`
- Conflict checks (--debug vs --release vs --profile, --bin vs --lib)

## Phase 3: Write help and version output

Hand-write help text matching the current format:
- `fol --help` shows root help with command groups and aliases
- `fol <group> --help` shows group help with subcommands
- `fol <group> <command> --help` shows command help with flags
- `fol --version` shows `fol-frontend 0.1.4`

## Phase 4: Replace shell completion generation

Replace `clap_complete::generate()` with hand-written completion scripts for
bash, zsh, fish. These are static strings with the command tree baked in.

Replace `internal_complete_matches` (currently walks `clap::Command` tree) with
a hand-written function that walks a static command/alias table.

## Phase 5: Update dispatch and lib.rs

- Remove `clap::error::ErrorKind::DisplayHelp` / `DisplayVersion` handling
- Replace with the new parser's error types (Help, Version, ParseError)
- Remove `FrontendCli::command()` (was for clap's CommandFactory)
- Update `run_command_from_args` and `run_from_args_with_io_inner`

## Phase 6: Remove clap from Cargo.toml

Remove `clap` and `clap_complete` from `[dependencies]`.
Run `cargo build` and `cargo test` to verify clean compile and all tests pass.

## Phase 7: Adapt tests

Update `cli/tests.rs` to use the new parser API instead of clap's `parse_from`
/ `try_parse_from`. The test assertions (struct equality checks) should remain
identical since the data types haven't changed.

---

## Execution order

Phases 1-2 are the core work. Phase 7 happens alongside phase 2 (tests drive the
parser). Phases 3-4 can be done in parallel. Phase 5-6 are the final cleanup.

Build and test after each phase. Commit after each phase.
