# FOL Hardening Plan

Last updated: 2026-03-18

## Slices

### Slice 1 — Dead code removal (lexer)
- [ ] 1a. Remove `ANY` variant from all 5 token enums: `BUILDIN`, `SYMBOL`, `OPERATOR`, `LITERAL`, `VOID` (`token/*/mod.rs` line 6 each)
- [ ] 1b. Remove `SYMBOL::Greater`, `SYMBOL::Less` — never emitted, shadowed by `AngleO`/`AngleC` (`token/symbol/mod.rs`)
- [ ] 1c. Remove `SYMBOL::Tik` — backtick consumed by comment scanner, never emitted as symbol (`token/symbol/mod.rs`)
- [ ] 1d. Remove `LITERAL::Bool` variant + dead match arm in parser (`token/literal/mod.rs:9`, `binding_declaration_parsers.rs:23`)
- [ ] 1e. Remove `get_keyword()` — incomplete stub, never called (`token/mod.rs:188-192`)
- [ ] 1f. Remove `is_oct_digit`, `is_hex_digit`, `is_alphanumeric` — never called (`token/help.rs:32-41`)
- [ ] 1g. Remove `TokenStream` and `Lexer` traits — declared, never implemented (`lib.rs:15-30`)
- [ ] 1h. Remove commented-out `println!` (`stage2/elements.rs:160`)
- [ ] 1i. Remove `debug()`, `echo()`, `window()` println methods from all 4 stages (`stage0-3/elements.rs`)
- [ ] 1j. Remove `crate_name()` wrapper — only used in test, constant `CRATE_NAME` is already exported (`fol-intrinsics/src/lib.rs:38-40`)

### Slice 2 — Legacy removal
- [ ] 2a. Remove parser legacy `parse()` method and doc comment (`program_parsing.rs:19-32`)
- [ ] 2b. Remove `compatibility_identity_for_program()` shim + its test (`fol-lower/src/lib.rs:58-63, 132`)
- [ ] 2c. Delete `test/legacy/` directory
- [ ] 2d. Delete committed build artifacts: `test/app/build/*/.fol/build/` (64 files)
- [ ] 2e. Remove "Slash comments remain a compatibility surface" comment (`stage1/element.rs:95`)

### Slice 3 — Parser panics → errors
- [ ] 3a. `skip_layout` panic → return parser error (`expression_atoms_and_literal_lowering.rs:50`)
- [ ] 3b. `skip_ignorable` panic → return parser error (`expression_atoms_and_literal_lowering.rs:106`)
- [ ] 3c. Depth guard `.expect()` → return parser error (`parser.rs:167`)
- [ ] 3d. Fix locationless errors (`line: 0, column: 0, file: None`) in `pipe_expression_parsers.rs:7-13`, `binding_declaration_parsers.rs:341-350`, `routine_header_parsers.rs:667-673,480-486`

### Slice 4 — Resolver + typecheck + lower panics → errors
- [ ] 4a. Resolver: `panic!` in qualified-path scope matching → error (`traverse/resolve.rs:391`)
- [ ] 4b. Lower: 5 `panic!` in type table lookups → errors (`exprs/containers.rs:656,678,700,721,740`)
- [ ] 4c. Resolver: `eprintln!` in `mount_visible_symbols_from_scope` → proper warning/diagnostic (`model.rs:638`)
- [ ] 4d. Typecheck: `eprintln!` in catch-all type inference → proper diagnostic (`exprs/mod.rs:475`)

### Slice 5 — Backend + frontend fixes
- [ ] 5a. Backend: `compile_error!()` strings for Local/Global operands → `BackendError` (`instructions/helpers.rs:291-296`)
- [ ] 5b. Frontend: `eprintln!` in fetch.rs → use structured warning (`fetch.rs:245`)
- [ ] 5c. Frontend: `Frontend::run()` uses `println!` bypassing IO injection → use writer (`lib.rs:92`)
- [ ] 5d. Editor: remove `let _ = position;` dead param in `current_namespace_for_position()` (`lsp/semantic.rs:529`)
- [ ] 5e. Frontend: remove unused `_config` params from 4 editor command functions (`editor.rs:48,57,66,75`)

### Slice 6 — Cargo.toml cleanup
- [ ] 6a. Root `Cargo.toml`: `colored = "1"` → `"2"` to match all crates
- [ ] 6b. Root `Cargo.toml`: remove redundant direct deps (only `fol-frontend` is used by `main.rs`)
- [ ] 6c. Unify `dyn-clone` version strings: `"1.0"` → `"1"` in `fol-parser/Cargo.toml`
- [ ] 6d. Unify `serde` version strings: `"1.0"` → `"1"` in `fol-diagnostics/Cargo.toml`
- [ ] 6e. Root `Cargo.toml`: add `"env"` feature to clap, or remove clap (it's redundant)
- [ ] 6f. `fol-resolver/Cargo.toml`: remove `fol-lexer` and `fol-stream` from `[dev-dependencies]` (already in `[dependencies]`)
