// Main test runner for FOL compiler components

mod stream {
    include!("stream/test_stream.rs");
}

mod lexer {
    include!("lexer/test_lexer.rs");
}

mod parser {
    include!("parser/test_parser.rs");
}

mod resolver {
    include!("resolver/test_resolver.rs");
}

mod typecheck {
    include!("typecheck/test_typecheck.rs");
}

#[cfg(test)]
mod integration_tests {
    use serde_json::Value;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fol_integration_{}_{}_{}",
            label,
            std::process::id(),
            stamp
        ))
    }

    fn run_fol(args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_fol"))
            .args(args)
            .output()
            .expect("Should run fol CLI")
    }

    fn parse_cli_json(output: &std::process::Output) -> Value {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_start = stdout
            .find('{')
            .expect("CLI JSON output should contain a JSON object");
        serde_json::from_str(&stdout[json_start..]).expect("CLI JSON output should stay valid")
    }

    fn write_combined_lowering_repro_fixture(root: &Path) -> PathBuf {
        let fixture = root.join("main.fol");
        std::fs::write(
            &fixture,
            concat!(
                "var enabled: bol = true\n",
                "var default_name: str = \"Ada\"\n",
                "var low_count: int = 1\n",
                "var high_count: int = 7\n",
                "typ NameTag: rec = {\n",
                "    label: str;\n",
                "    code: int\n",
                "}\n",
                "typ Audit: rec = {\n",
                "    active: bol;\n",
                "    marker: NameTag\n",
                "}\n",
                "typ User: rec = {\n",
                "    name: str;\n",
                "    count: int;\n",
                "    audit: Audit\n",
                "}\n",
                "fun[] build_tag(): NameTag = {\n",
                "    return { label = \"stable\", code = high_count }\n",
                "}\n",
                "fun[] build_user(flag: bol): User = {\n",
                "    return {\n",
                "        name = default_name,\n",
                "        count = high_count,\n",
                "        audit = {\n",
                "            active = flag,\n",
                "            marker = build_tag(),\n",
                "        },\n",
                "    }\n",
                "}\n",
                "fun[] choose_count(flag: bol): int = {\n",
                "    when(flag) {\n",
                "        case(true) { high_count }\n",
                "        * { low_count }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): int = {\n",
                "    var current: User = build_user(flag)\n",
                "    var names: seq[str] = {\"Ada\", \"Lin\"}\n",
                "    var counts: map[str, int] = {{\"ada\", 1}, {\"lin\", 2}}\n",
                "    loop(flag) {\n",
                "        break\n",
                "    }\n",
                "    when(flag) {\n",
                "        case(true) { return current.audit.marker.code }\n",
                "        * { return counts[\"lin\"] }\n",
                "    }\n",
                "}\n",
            ),
        )
        .expect("Should write combined lowering repro fixture");
        fixture
    }

    fn write_parameter_scope_lowering_fixture(root: &Path) -> PathBuf {
        let fixture = root.join("main.fol");
        std::fs::write(
            &fixture,
            concat!(
                "fun[] choose(flag: bol): int = {\n",
                "    when(flag) {\n",
                "        case(true) { 1 }\n",
                "        * { 0 }\n",
                "    }\n",
                "}\n",
                "fun[] echo(flag: bol): bol = {\n",
                "    return flag\n",
                "}\n",
                "fun[] main(flag: bol): int = {\n",
                "    when(echo(flag)) {\n",
                "        case(true) { return choose(flag) }\n",
                "        * { return 0 }\n",
                "    }\n",
                "}\n",
            ),
        )
        .expect("Should write parameter-scope lowering fixture");
        fixture
    }

    fn write_container_lowering_fixture(root: &Path) -> PathBuf {
        let fixture = root.join("main.fol");
        std::fs::write(
            &fixture,
            concat!(
                "fun[] main(): int = {\n",
                "    var names: seq[str] = {\"Ada\", \"Lin\"}\n",
                "    var counts: map[str, int] = {{\"ada\", 1}, {\"lin\", 2}}\n",
                "    return counts[\"lin\"]\n",
                "}\n",
            ),
        )
        .expect("Should write container lowering fixture");
        fixture
    }

    fn write_early_return_when_fixture(root: &Path) -> PathBuf {
        let fixture = root.join("main.fol");
        std::fs::write(
            &fixture,
            concat!(
                "fun[] main(flag: bol): int = {\n",
                "    when(flag) {\n",
                "        case(true) { return 7 }\n",
                "        * { return 3 }\n",
                "    }\n",
                "}\n",
            ),
        )
        .expect("Should write early-return when lowering fixture");
        fixture
    }

    #[test]
    fn test_stream_to_lexer_integration() {
        use fol_lexer::lexer::stage3::Elements;
        use fol_stream::FileStream;

        // Test that stream output works with lexer input
        let mut file_stream =
            FileStream::from_file("test/lexer/mixed.fol").expect("Should read test file");

        let lexer = Elements::init(&mut file_stream);

        // Should be able to get at least one token
        match lexer.curr(false) {
            Ok(token) => {
                println!("Integration test: First token = '{}'", token.con());
                // Check that we get a valid token (even if empty content like spaces)
                // or EOF - this verifies the integration is working
                assert!(
                    !token.key().is_illegal(),
                    "Integration token should be valid"
                );
            }
            Err(e) => panic!("Stream to lexer integration failed: {:?}", e),
        }
    }

    #[test]
    fn test_stream_to_lexer_order_stays_stable_across_multiple_files() {
        use fol_lexer::lexer::stage3::Elements;
        use fol_lexer::token::KEYWORD;
        use fol_stream::FileStream;
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after unix epoch")
            .as_nanos();
        let temp_root = std::env::temp_dir().join(format!(
            "fol_stream_lexer_order_{}_{}",
            std::process::id(),
            stamp
        ));

        fs::create_dir_all(temp_root.join("alpha_10")).expect("Should create alpha fixture dir");
        fs::create_dir_all(temp_root.join("beta_20")).expect("Should create beta fixture dir");
        fs::write(temp_root.join("00_root.fol"), "root_token").expect("Should write root fixture");
        fs::write(temp_root.join("alpha_10/entry.fol"), "alpha_token")
            .expect("Should write alpha fixture");
        fs::write(temp_root.join("beta_20/entry.fol"), "beta_token")
            .expect("Should write beta fixture");

        let mut file_stream = FileStream::from_folder(
            temp_root
                .to_str()
                .expect("Order fixture path should be valid utf-8"),
        )
        .expect("Should create file stream from ordered folder fixture");
        let mut lexer = Elements::init(&mut file_stream);
        let mut identifiers = Vec::new();

        for _ in 0..10_000 {
            let token = lexer
                .curr(false)
                .expect("Lexer should expose tokens while walking the ordered fixture");
            if token.key().is_eof() {
                break;
            }
            if matches!(token.key(), KEYWORD::Identifier) {
                identifiers.push(token.con().to_string());
            }
            if lexer.bump().is_none() {
                break;
            }
        }

        assert_eq!(
            identifiers,
            vec![
                "root_token".to_string(),
                "alpha_token".to_string(),
                "beta_token".to_string(),
            ],
            "Flattened lexer output should preserve the stream's deterministic file order"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_lexer_to_parser_literal_continuity_for_supported_forms() {
        use fol_lexer::lexer::stage3::Elements;
        use fol_parser::ast::{AstNode, AstParser, Literal};
        use fol_stream::FileStream;
        use std::fs;

        let temp_root = unique_temp_root("literal_continuity");
        fs::create_dir_all(&temp_root).expect("Should create temp literal fixture dir");
        let fixture = temp_root.join("literal_continuity.fol");
        fs::write(
            &fixture,
            "\"hello\"\n'c'\n'true'\n42\n3.5\n0x1A\n0o17\n0b1010\ntrue\nfalse\nnil\n",
        )
        .expect("Should write literal continuity fixture");

        let mut file_stream = FileStream::from_file(
            fixture
                .to_str()
                .expect("Literal continuity fixture path should be utf-8"),
        )
        .expect("Should open literal continuity fixture");
        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser.parse(&mut lexer).expect(
            "Supported literal forms should survive stream and lexer into exact AST literals",
        );

        match ast {
            AstNode::Program { declarations } => {
                assert_eq!(
                    declarations,
                    vec![
                        AstNode::Literal(Literal::String("hello".to_string())),
                        AstNode::Literal(Literal::Character('c')),
                        AstNode::Literal(Literal::String("true".to_string())),
                        AstNode::Literal(Literal::Integer(42)),
                        AstNode::Literal(Literal::Float(3.5)),
                        AstNode::Literal(Literal::Integer(0x1A)),
                        AstNode::Literal(Literal::Integer(0o17)),
                        AstNode::Literal(Literal::Integer(0b1010)),
                        AstNode::Literal(Literal::Boolean(true)),
                        AstNode::Literal(Literal::Boolean(false)),
                        AstNode::Literal(Literal::Nil),
                    ],
                    "Cross-phase literal continuity should preserve exact AST literal values for supported forms"
                );
            }
            _ => panic!("Expected program node"),
        }

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_lexer_to_parser_integration() {
        use fol_lexer::lexer::stage3::Elements;
        use fol_parser::ast::AstParser;
        use fol_stream::FileStream;

        // Test that lexer output works with parser input
        let mut file_stream =
            FileStream::from_file("test/parser/simple_var.fol").expect("Should read test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        // Should be able to parse without crashing
        match parser.parse(&mut lexer) {
            Ok(_ast) => {
                println!("Lexer to parser integration successful");
            }
            Err(errors) => {
                println!("Parser errors (may be expected): {:?}", errors);
                // For minimal parser, we mainly test that it doesn't crash
            }
        }
    }

    #[test]
    fn test_full_pipeline_integration() {
        use fol_lexer::lexer::stage3::Elements;
        use fol_parser::ast::AstParser;
        use fol_resolver::resolve_package;
        use fol_stream::FileStream;

        let mut file_stream =
            FileStream::from_file("test/parser/simple_var.fol").expect("Should read test file");
        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let parsed = parser
            .parse_package(&mut lexer)
            .expect("Full pipeline happy-path fixture should parse cleanly");
        let resolved =
            resolve_package(parsed).expect("Full pipeline happy-path fixture should resolve");

        assert_eq!(resolved.package_name(), "parser");
        assert_eq!(resolved.source_units.len(), 1);
        assert!(
            !resolved.symbols_in_scope(resolved.program_scope).is_empty(),
            "Resolver-backed full pipeline runs should produce top-level symbols"
        );
    }

    #[test]
    fn test_full_pipeline_cross_file_import_resolution() {
        use fol_lexer::lexer::stage3::Elements;
        use fol_parser::ast::AstParser;
        use fol_resolver::resolve_package;
        use fol_stream::FileStream;
        use std::fs;

        let temp_root = unique_temp_root("pipeline_cross_file_import");
        fs::create_dir_all(temp_root.join("net/http"))
            .expect("Should create a temporary integration fixture directory");
        fs::write(temp_root.join("net/http/route.fol"), "var handler: int = 1;\n")
            .expect("Should write the imported namespace fixture");
        fs::write(
            temp_root.join("main.fol"),
            "use http: loc = {net::http};\nfun[] main(): int = {\n    return http;\n}\n",
        )
        .expect("Should write the importing source unit fixture");

        let mut file_stream = FileStream::from_folder(
            temp_root
                .to_str()
                .expect("Integration fixture path should be valid UTF-8"),
        )
        .expect("Should read integration folder fixture");
        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let parsed = parser
            .parse_package(&mut lexer)
            .expect("Cross-file import fixture should parse cleanly");
        let resolved =
            resolve_package(parsed).expect("Cross-file import fixture should resolve cleanly");
        let import = resolved
            .imports_in_scope(resolved.program_scope)
            .into_iter()
            .find(|import| import.alias_name == "http")
            .expect("Resolved program should keep the import record");

        assert!(
            matches!(
                import
                    .target_scope
                    .and_then(|scope_id| resolved.scope(scope_id))
                    .map(|scope| &scope.kind),
                Some(fol_resolver::ScopeKind::ProgramRoot { package }) if package == "http"
            ),
            "Cross-file full pipeline runs should mount exact loc directories as imported root scopes"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_single_file_compile_succeeds_with_package_parser() {
        let output = run_fol(&["test/parser/simple_var.fol"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should accept declaration-only single-file input, got status {:?} and output:\n{}",
            output.status.code(),
            stdout
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful compile"
        );
    }

    #[test]
    fn test_cli_typecheck_accepts_loc_imported_symbols_after_workspace_handoff() {
        use std::fs;

        let temp_root = unique_temp_root("cli_loc_import");
        let shared_root = temp_root.join("shared");
        let app_root = temp_root.join("app");
        fs::create_dir_all(&shared_root).expect("Should create the shared fixture directory");
        fs::create_dir_all(&app_root).expect("Should create the app fixture directory");
        fs::write(shared_root.join("lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the shared export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write the loc import fixture");

        let output = run_fol(&[app_root
            .to_str()
            .expect("Temporary app fixture path should be valid UTF-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should typecheck imported loc symbols through the full workspace-aware chain, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful compile for loc-imported packages"
        );
    }

    #[test]
    fn test_cli_resolves_std_imports_with_explicit_std_root_configuration() {
        use std::fs;

        let temp_root = unique_temp_root("cli_std_root_import");
        let std_root = temp_root.join("std");
        let app_root = temp_root.join("app");
        fs::create_dir_all(std_root.join("fmt"))
            .expect("Should create the standard-library fixture directory");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing package root fixture directory");
        fs::write(std_root.join("fmt/value.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the standard-library export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use fmt: std = {fmt};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write the std import fixture");

        let output = run_fol(&[
            "--std-root",
            std_root
                .to_str()
                .expect("Temporary std-root fixture path should be valid UTF-8"),
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should resolve std imports through an explicit std-root flag, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful compile for std-imported packages",
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_accepts_explicit_package_store_root_configuration() {
        use std::fs;

        let temp_root = unique_temp_root("cli_package_store_root");
        let store_root = temp_root.join("store");
        let app_root = temp_root.join("app");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create the package-store fixture directory");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing package root fixture directory");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the installed package metadata fixture");
        fs::create_dir_all(store_root.join("json/src"))
            .expect("Should create the installed package export root fixture");
        fs::write(store_root.join("json/build.fol"), "def root: loc = \"src\";\n")
            .expect("Should write the installed package build fixture");
        fs::write(store_root.join("json/src/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the installed package export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use json: pkg = {json};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write the pkg import fixture");

        let output = run_fol(&[
            "--package-store-root",
            store_root
                .to_str()
                .expect("Temporary package-store fixture path should be valid UTF-8"),
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should accept an explicit package-store root and resolve pkg imports, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful compile for pkg-imported packages"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_loc_import_graphs() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_loc");
        let shared_root = temp_root.join("shared");
        let app_root = temp_root.join("app");
        fs::create_dir_all(&shared_root).expect("Should create the shared fixture directory");
        fs::create_dir_all(&app_root).expect("Should create the app fixture directory");
        fs::write(shared_root.join("lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the shared export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write the loc import fixture");

        let output = run_fol(&[
            "--dump-lowered",
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for loc-import graphs, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(stdout.contains("workspace entry=app"));
        assert!(stdout.contains("package app"));
        assert!(stdout.contains("package shared"));
        assert!(stdout.contains("entry-candidates"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_std_import_graphs() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_std");
        let std_root = temp_root.join("std");
        let app_root = temp_root.join("app");
        fs::create_dir_all(std_root.join("fmt"))
            .expect("Should create the standard-library fixture directory");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing package root fixture directory");
        fs::write(std_root.join("fmt/value.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the standard-library export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use fmt: std = {fmt};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write the std import fixture");

        let output = run_fol(&[
            "--dump-lowered",
            "--std-root",
            std_root
                .to_str()
                .expect("Temporary std-root fixture path should be valid UTF-8"),
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for std-import graphs, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(stdout.contains("workspace entry=app"));
        assert!(stdout.contains("package app"));
        assert!(stdout.contains("package fmt"));
        assert!(stdout.contains("entry-candidates"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_pkg_import_graphs() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_pkg");
        let store_root = temp_root.join("store");
        let app_root = temp_root.join("app");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create the package-store fixture directory");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing package root fixture directory");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the installed package metadata fixture");
        fs::create_dir_all(store_root.join("json/src"))
            .expect("Should create the installed package export root fixture");
        fs::write(store_root.join("json/build.fol"), "def root: loc = \"src\";\n")
            .expect("Should write the installed package build fixture");
        fs::write(store_root.join("json/src/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the installed package export fixture");
        fs::write(
            app_root.join("main.fol"),
            "use json: pkg = {json};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write the pkg import fixture");

        let output = run_fol(&[
            "--dump-lowered",
            "--package-store-root",
            store_root
                .to_str()
                .expect("Temporary package-store fixture path should be valid UTF-8"),
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for pkg-import graphs, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(stdout.contains("workspace entry=app"));
        assert!(stdout.contains("package app"));
        assert!(stdout.contains("package json"));
        assert!(stdout.contains("entry-candidates"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_intrinsic_comparison_calls() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_intrinsic_comparisons");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic comparison fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(): bol = {\n",
                "    var same: bol = .eq(1, 1)\n",
                "    var ordered: bol = .lt(\"Ada\", \"Lin\")\n",
                "    return .ge('z', 'a')\n",
                "}\n",
            ),
        )
        .expect("Should write intrinsic comparison fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture
                .to_str()
                .expect("Temporary intrinsic comparison fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for intrinsic comparison calls, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.matches("IntrinsicCall").count() >= 3,
            "Lowered dump should retain explicit intrinsic calls for comparison families, got:\n{}",
            stdout,
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_intrinsic_comparison_failures_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_intrinsic_comparison_failures");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic comparison failure fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(): bol = {\n",
                "    return .lt(true, false)\n",
                "}\n",
            ),
        )
        .expect("Should write intrinsic comparison failure fixture");

        let output = run_fol(&[
            "--json",
            fixture
                .to_str()
                .expect("Temporary intrinsic comparison failure fixture path should be valid UTF-8"),
        ]);

        assert!(
            !output.status.success(),
            "CLI should fail for invalid intrinsic comparison calls",
        );

        let json = parse_cli_json(&output);
        let diagnostics = json["diagnostics"]
            .as_array()
            .expect("CLI JSON output should expose diagnostics");
        let ordered_error = diagnostics.iter().find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .map(|message| message.contains(".lt(...) expects two ordered scalar operands"))
                .unwrap_or(false)
        });

        assert!(
            ordered_error.is_some(),
            "Expected intrinsic comparison diagnostic in CLI JSON output, got: {json}"
        );
        assert!(
            ordered_error
                .and_then(|diagnostic| diagnostic["location"].as_object())
                .is_some(),
            "Expected intrinsic comparison diagnostic to keep a structured location, got: {json}"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_succeeds_for_intrinsic_boolean_calls() {
        use std::fs;

        let temp_root = unique_temp_root("cli_dump_lowered_intrinsic_boolean");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic boolean fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(flag: bol): bol = {\n",
                "    var inverted: bol = .not(flag)\n",
                "    return .not(inverted)\n",
                "}\n",
            ),
        )
        .expect("Should write intrinsic boolean fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture
                .to_str()
                .expect("Temporary intrinsic boolean fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for intrinsic boolean calls, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.matches("IntrinsicCall").count() >= 2,
            "Lowered dump should retain explicit intrinsic calls for '.not', got:\n{}",
            stdout,
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_intrinsic_boolean_failures_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_intrinsic_boolean_failures");
        fs::create_dir_all(&temp_root).expect("Should create temp intrinsic boolean failure fixture");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] main(): bol = {\n",
                "    return .not(1)\n",
                "}\n",
            ),
        )
        .expect("Should write intrinsic boolean failure fixture");

        let output = run_fol(&[
            "--json",
            fixture
                .to_str()
                .expect("Temporary intrinsic boolean failure fixture path should be valid UTF-8"),
        ]);

        assert!(
            !output.status.success(),
            "CLI should fail for invalid intrinsic boolean calls",
        );

        let json = parse_cli_json(&output);
        let diagnostics = json["diagnostics"]
            .as_array()
            .expect("CLI JSON output should expose diagnostics");
        let boolean_error = diagnostics.iter().find(|diagnostic| {
            diagnostic["message"]
                .as_str()
                .map(|message| message.contains(".not(...) expects one boolean operand"))
                .unwrap_or(false)
        });

        assert!(
            boolean_error.is_some(),
            "Expected intrinsic boolean diagnostic in CLI JSON output, got: {json}"
        );
        assert!(
            boolean_error
                .and_then(|diagnostic| diagnostic["location"].as_object())
                .is_some(),
            "Expected intrinsic boolean diagnostic to keep a structured location, got: {json}"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_folder_compile_succeeds_with_package_parser() {
        use std::fs;

        let temp_root = unique_temp_root("cli_folder_compile");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI folder fixture");
        fs::write(temp_root.join("00_first.fol"), "var first = 1\n")
            .expect("Should write first declaration source");
        fs::write(temp_root.join("10_second.fol"), "var second = 2\n")
            .expect("Should write second declaration source");

        let output = run_fol(&[temp_root
            .to_str()
            .expect("CLI folder fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should accept declaration-only folders, got status {:?} and output:\n{}",
            output.status.code(),
            stdout
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful folder compile"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_single_file_compile_succeeds_with_builtin_str_types() {
        use std::fs;

        let temp_root = unique_temp_root("cli_builtin_str_compile");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI builtin-str fixture dir");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            "ali Text: str;\ntyp User: rec = {\n    var name: str;\n};\nfun[] main(path: str): Text = {\n    var local: str = path;\n    return local;\n}\n",
        )
        .expect("Should write builtin str CLI fixture");

        let output = run_fol(&[fixture
            .to_str()
            .expect("CLI builtin str fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should accept builtin str across alias, type, and routine surfaces, got status {:?} and output:\n{}",
            output.status.code(),
            stdout
        );
        assert!(
            !stdout.contains("could not resolve type 'str'"),
            "CLI output should no longer report resolver failures for builtin str"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_lowering_failures_surface_human_diagnostics() {
        use std::fs;

        let temp_root = unique_temp_root("cli_lowering_failure_human");
        fs::create_dir_all(&temp_root).expect("Should create temp lowering failure fixture dir");
        let fixture = temp_root.join("main.fol");
        fs::write(&fixture, "fun[] main(): int = {\n    return 1 + 2;\n}\n")
            .expect("Should write lowering failure fixture");

        let output = run_fol(&[fixture
            .to_str()
            .expect("CLI lowering failure fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !output.status.success(),
            "CLI should fail lowering on unsupported lowered V1 surfaces, got status {:?} and output:\n{}",
            output.status.code(),
            stdout
        );
        assert!(stdout.contains("LoweringUnsupported"));
        assert!(stdout.contains("binary operator lowering for 'add'"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_lowering_repro_program_now_succeeds_end_to_end() {
        use std::fs;

        let temp_root = unique_temp_root("cli_lowering_repro_boundary");
        fs::create_dir_all(&temp_root).expect("Should create temp lowering repro fixture dir");
        let fixture = write_combined_lowering_repro_fixture(&temp_root);

        let output = run_fol(&[fixture
            .to_str()
            .expect("CLI lowering repro fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "CLI should compile the combined lowering repro now, got status {:?} and output:\n{}\n{}",
            output.status.code(),
            stdout,
            stderr,
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Combined repro should compile cleanly through lowering, got stdout:\n{}\n\nstderr:\n{}",
            stdout,
            stderr,
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_dump_lowered_combined_repro_stays_stable_and_inspectable() {
        use std::fs;

        let temp_root = unique_temp_root("cli_lowering_repro_dump");
        fs::create_dir_all(&temp_root).expect("Should create temp lowering dump fixture dir");
        write_combined_lowering_repro_fixture(&temp_root);

        let output = run_fol(&[
            "--dump-lowered",
            temp_root
                .to_str()
                .expect("CLI lowering dump fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should dump lowered output for the repaired combined repro, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(stdout.contains("workspace entry="));
        assert!(stdout.contains("package"));
        assert!(stdout.contains("type-decl User"));
        assert!(stdout.contains("routine r1 build_user"));
        assert!(stdout.contains("routine r3 main"));
        assert!(stdout.contains("params [l0]"));
        assert!(stdout.contains("ConstructLinear { kind: Sequence"));
        assert!(stdout.contains("ConstructMap"));
        assert!(stdout.contains("FieldAccess { base:"));
        assert!(stdout.contains("IndexAccess { container:"));
        assert!(stdout.contains("entry-candidates"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_lowering_parameter_scope_regression_now_succeeds() {
        use std::fs;

        let temp_root = unique_temp_root("cli_lowering_parameter_scope");
        fs::create_dir_all(&temp_root).expect("Should create temp parameter regression fixture dir");
        let fixture = write_parameter_scope_lowering_fixture(&temp_root);

        let output = run_fol(&[fixture
            .to_str()
            .expect("CLI parameter regression fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "CLI should compile the repaired parameter-scope lowering repro, got status {:?} and output:\n{}\n{}",
            output.status.code(),
            stdout,
            stderr,
        );
        assert!(stdout.contains("Compilation successful"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_lowering_container_regression_now_succeeds() {
        use std::fs;

        let temp_root = unique_temp_root("cli_lowering_container_regression");
        fs::create_dir_all(&temp_root).expect("Should create temp container regression fixture dir");
        let fixture = write_container_lowering_fixture(&temp_root);

        let output = run_fol(&[fixture
            .to_str()
            .expect("CLI container regression fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "CLI should compile the repaired container lowering repro, got status {:?} and output:\n{}\n{}",
            output.status.code(),
            stdout,
            stderr,
        );
        assert!(stdout.contains("Compilation successful"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_lowering_early_return_when_regression_now_succeeds() {
        use std::fs;

        let temp_root = unique_temp_root("cli_lowering_early_return_when");
        fs::create_dir_all(&temp_root).expect("Should create temp early-return regression fixture dir");
        let fixture = write_early_return_when_fixture(&temp_root);

        let output = run_fol(&[fixture
            .to_str()
            .expect("CLI early-return regression fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "CLI should compile the repaired early-return when repro, got status {:?} and output:\n{}\n{}",
            output.status.code(),
            stdout,
            stderr,
        );
        assert!(stdout.contains("Compilation successful"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_error_propagation_lowers_successfully_across_multiple_routines() {
        use std::fs;

        let temp_root = unique_temp_root("cli_error_propagation");
        fs::create_dir_all(&temp_root).expect("Should create temp error propagation fixture dir");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] leaf(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] mid(flag: bol): int / str = {\n",
                "    return leaf(flag)\n",
                "}\n",
                "fun[] main(flag: bol): int / str = {\n",
                "    return mid(flag)\n",
                "}\n",
            ),
        )
        .expect("Should write error propagation fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture.to_str().expect("error propagation fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "error propagation fixture should compile, got:\n{stdout}");
        assert!(stdout.contains("CheckRecoverable"));
        assert!(stdout.contains("UnwrapRecoverable"));
        assert!(stdout.contains("ExtractRecoverableError"));
        assert!(stdout.contains("Report"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_check_handling_lowers_without_error_propagation() {
        use std::fs;

        let temp_root = unique_temp_root("cli_error_check");
        fs::create_dir_all(&temp_root).expect("Should create temp error check fixture dir");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] load(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): bol = {\n",
                "    var attempt = load(flag)\n",
                "    return check(attempt)\n",
                "}\n",
            ),
        )
        .expect("Should write error check fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture.to_str().expect("error check fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "error check fixture should compile, got:\n{stdout}");
        assert!(stdout.contains("CheckRecoverable"));
        assert!(!stdout.contains("ExtractRecoverableError"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_pipe_or_default_lowers_successfully() {
        use std::fs;

        let temp_root = unique_temp_root("cli_error_pipe_or_default");
        fs::create_dir_all(&temp_root).expect("Should create temp pipe-or default fixture dir");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] load(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): int = {\n",
                "    return load(flag) || 5\n",
                "}\n",
            ),
        )
        .expect("Should write pipe-or default fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture
                .to_str()
                .expect("pipe-or default fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "pipe-or default fixture should compile, got:\n{stdout}");
        assert!(stdout.contains("CheckRecoverable"));
        assert!(stdout.contains("UnwrapRecoverable"));
        assert!(stdout.contains("Const(Int(5))"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_pipe_or_report_lowers_successfully() {
        use std::fs;

        let temp_root = unique_temp_root("cli_error_pipe_or_report");
        fs::create_dir_all(&temp_root).expect("Should create temp pipe-or report fixture dir");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] load(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): int / str = {\n",
                "    return load(flag) || report \"fallback\"\n",
                "}\n",
            ),
        )
        .expect("Should write pipe-or report fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture
                .to_str()
                .expect("pipe-or report fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "pipe-or report fixture should compile, got:\n{stdout}");
        assert!(stdout.contains("CheckRecoverable"));
        assert!(stdout.contains("Report"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_pipe_or_panic_lowers_successfully() {
        use std::fs;

        let temp_root = unique_temp_root("cli_error_pipe_or_panic");
        fs::create_dir_all(&temp_root).expect("Should create temp pipe-or panic fixture dir");
        let fixture = temp_root.join("main.fol");
        fs::write(
            &fixture,
            concat!(
                "fun[] load(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): int = {\n",
                "    return load(flag) || panic \"fallback\"\n",
                "}\n",
            ),
        )
        .expect("Should write pipe-or panic fixture");

        let output = run_fol(&[
            "--dump-lowered",
            fixture
                .to_str()
                .expect("pipe-or panic fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "pipe-or panic fixture should compile, got:\n{stdout}");
        assert!(stdout.contains("CheckRecoverable"));
        assert!(stdout.contains("Panic"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_recoverable_abi_stays_stable_across_workspace_call_paths() {
        use std::fs;

        let temp_root = unique_temp_root("cli_error_abi_workspace");
        let app_root = temp_root.join("app");
        let shared_root = temp_root.join("shared");
        fs::create_dir_all(&app_root).expect("Should create app root");
        fs::create_dir_all(&shared_root).expect("Should create shared root");
        fs::write(
            shared_root.join("lib.fol"),
            concat!(
                "fun[exp] remote(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"shared-bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
            ),
        )
        .expect("Should write shared recoverable fixture");
        fs::write(
            app_root.join("00_leaf.fol"),
            concat!(
                "fun[] leaf(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"leaf-bad\" }\n",
                "        * { return 5 }\n",
                "    }\n",
                "}\n",
            ),
        )
        .expect("Should write local recoverable fixture");
        fs::write(
            app_root.join("05_mid.fol"),
            concat!(
                "use shared: loc = {\"../shared\"};\n",
                "fun[] mid(flag: bol): int / str = {\n",
                "    loop(flag) {\n",
                "        break\n",
                "    }\n",
                "    when(flag) {\n",
                "        case(true) { return remote(flag) }\n",
                "        * { return leaf(flag) }\n",
                "    }\n",
                "}\n",
            ),
        )
        .expect("Should write middle recoverable fixture");
        fs::write(
            app_root.join("10_main.fol"),
            concat!(
                "fun[] main(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { return mid(flag) }\n",
                "        * { return leaf(flag) }\n",
                "    }\n",
                "}\n",
            ),
        )
        .expect("Should write entry recoverable fixture");

        let output = run_fol(&[
            "--dump-lowered",
            app_root
                .to_str()
                .expect("app root should be valid utf-8 for dump-lowered"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "recoverable ABI workspace fixture should compile, got:\n{stdout}"
        );
        assert!(stdout.contains("recoverable-abi kind=tagged-result-object"));
        assert!(stdout.contains("package app"));
        assert!(stdout.contains("package shared"));
        assert!(stdout.contains("CheckRecoverable"));
        assert!(stdout.contains("ExtractRecoverableError"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_lowering_failures_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_lowering_failure_json");
        fs::create_dir_all(&temp_root).expect("Should create temp lowering failure fixture dir");
        let fixture = temp_root.join("main.fol");
        fs::write(&fixture, "fun[] main(): int = {\n    return 1 + 2;\n}\n")
            .expect("Should write lowering failure fixture");

        let output = run_fol(&[
            "--json",
            fixture
                .to_str()
                .expect("CLI lowering failure fixture path should be utf-8"),
        ]);
        let payload = parse_cli_json(&output);

        assert!(
            !output.status.success(),
            "CLI JSON lowering failures should still exit unsuccessfully"
        );
        assert_eq!(payload["error_count"], 1);
        assert_eq!(payload["diagnostics"][0]["severity"], "Error");
        assert_eq!(payload["diagnostics"][0]["code"], "L1001");
        assert!(
            payload["diagnostics"][0]["message"]
                .as_str()
                .expect("lowering failure message should stay textual")
                .contains("binary operator lowering for 'add'"),
            "Structured JSON lowering diagnostics should preserve the lowering failure message"
        );
        assert!(
            payload["diagnostics"][0]["labels"]
                .as_array()
                .expect("labels should stay arrays")
                .is_empty(),
            "Current lowering failures should keep an empty related-label list when no origin is available"
        );

        fs::remove_dir_all(&temp_root).ok();
    }


    #[test]
    fn test_cli_folder_parse_errors_keep_json_locations_with_package_parser() {
        use std::fs;

        let temp_root = unique_temp_root("cli_folder_parse_error");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI error fixture");
        fs::write(temp_root.join("00_good.fol"), "var ok = 1\n").expect("Should write good source");
        fs::write(temp_root.join("10_bad.fol"), "run(1, 2)\n")
            .expect("Should write invalid file-root source");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI error fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let compact = stdout
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        assert!(
            !output.status.success(),
            "CLI should fail on declaration-only package parse errors"
        );
        assert!(
            stdout.contains("10_bad.fol"),
            "JSON diagnostics should identify the failing second source unit"
        );
        assert!(
            compact.contains("\"line\":1"),
            "JSON diagnostics should preserve the failing line number"
        );
        assert!(
            compact.contains("\"column\":1"),
            "JSON diagnostics should preserve the failing column number"
        );
        assert!(
            stdout.contains("Executable calls are not allowed at file root"),
            "JSON diagnostics should keep the parser's file-root error wording"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_folder_resolver_errors_fail_parse_clean_programs() {
        use std::fs;

        let temp_root = unique_temp_root("cli_folder_resolver_error");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI resolver fixture");
        fs::write(temp_root.join("00_first.fol"), "var value = 1\n")
            .expect("Should write first declaration source");
        fs::write(temp_root.join("10_second.fol"), "var value = 2\n")
            .expect("Should write duplicate declaration source");

        let output = run_fol(&[temp_root
            .to_str()
            .expect("CLI resolver fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !output.status.success(),
            "CLI should fail when resolver rejects a parse-clean folder"
        );
        assert!(
            stdout.contains("duplicate symbol 'value'"),
            "CLI diagnostics should surface resolver duplicate-symbol messages"
        );
        assert!(
            stdout.contains("10_second.fol"),
            "CLI diagnostics should identify the duplicate source file"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_folder_resolver_errors_keep_json_locations() {
        use std::fs;

        let temp_root = unique_temp_root("cli_folder_resolver_error_json");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI resolver fixture");
        fs::write(temp_root.join("00_first.fol"), "var value = 1\n")
            .expect("Should write first declaration source");
        fs::write(temp_root.join("10_second.fol"), "var value = 2\n")
            .expect("Should write duplicate declaration source");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI resolver fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let compact = stdout
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        assert!(
            !output.status.success(),
            "CLI should fail in JSON mode when resolver rejects a parse-clean folder"
        );
        assert!(
            stdout.contains("10_second.fol"),
            "JSON resolver diagnostics should identify the duplicate source file"
        );
        assert!(
            compact.contains("\"line\":1"),
            "JSON resolver diagnostics should preserve the duplicate declaration line number"
        );
        assert!(
            compact.contains("\"column\":1"),
            "JSON resolver diagnostics should preserve the duplicate declaration column number"
        );
        assert!(
            stdout.contains("duplicate symbol 'value'"),
            "JSON resolver diagnostics should keep resolver duplicate-symbol wording"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_resolver_errors_keep_exact_json_locations_for_qualified_paths() {
        use std::fs;

        let temp_root = unique_temp_root("cli_resolver_qualified_location");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI resolver fixture");
        let main_file = temp_root.join("main.fol");
        fs::write(&main_file, "ali Broken: tools::Missing;\n")
            .expect("Should write unresolved qualified type fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI resolver fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let compact = stdout
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        assert!(
            !output.status.success(),
            "CLI should fail in JSON mode when resolver rejects an unresolved qualified path"
        );
        assert!(
            stdout.contains(
                main_file
                    .to_str()
                    .expect("Temporary resolver fixture path should be valid UTF-8")
            ),
            "JSON resolver diagnostics should keep the exact source file for qualified paths"
        );
        assert!(
            compact.contains("\"line\":1"),
            "JSON resolver diagnostics should preserve the exact failing line number"
        );
        assert!(
            compact.contains("\"column\":13"),
            "JSON resolver diagnostics should preserve the exact qualified-path column"
        );
        assert!(
            stdout.contains("could not resolve qualified type 'tools::Missing'"),
            "JSON resolver diagnostics should keep the exact qualified-type wording"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_resolver_errors_keep_exact_json_locations_for_plain_unresolved_names() {
        use std::fs;

        let temp_root = unique_temp_root("cli_resolver_plain_unresolved_location");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI resolver fixture");
        let main_file = temp_root.join("main.fol");
        fs::write(&main_file, "fun[] main(): int = {\n    return missing;\n}\n")
            .expect("Should write unresolved plain-name fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI resolver fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let compact = stdout
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        assert!(
            !output.status.success(),
            "CLI should fail in JSON mode when resolver rejects an unresolved plain name"
        );
        assert!(
            stdout.contains(
                main_file
                    .to_str()
                    .expect("Temporary resolver fixture path should be valid UTF-8")
            ),
            "JSON resolver diagnostics should keep the exact source file for plain unresolved names"
        );
        assert!(
            !compact.contains("\"file\":null"),
            "JSON resolver diagnostics for plain unresolved names should never drop the file field"
        );
        assert!(
            compact.contains("\"line\":2"),
            "JSON resolver diagnostics should preserve the exact failing line number"
        );
        assert!(
            compact.contains("\"column\":12"),
            "JSON resolver diagnostics should preserve the exact plain-name column"
        );
        assert!(
            stdout.contains("could not resolve name 'missing'"),
            "JSON resolver diagnostics should keep the exact unresolved plain-name wording"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_resolver_errors_keep_exact_json_locations_for_ambiguous_plain_names() {
        use std::fs;

        let temp_root = unique_temp_root("cli_resolver_plain_ambiguity_location");
        fs::create_dir_all(temp_root.join("alpha"))
            .expect("Should create first imported namespace fixture");
        fs::create_dir_all(temp_root.join("beta"))
            .expect("Should create second imported namespace fixture");
        fs::write(temp_root.join("alpha/values.fol"), "var[exp] answer: int = 1;\n")
            .expect("Should write first imported exported value fixture");
        fs::write(temp_root.join("beta/values.fol"), "var[exp] answer: int = 2;\n")
            .expect("Should write second imported exported value fixture");
        let main_file = temp_root.join("main.fol");
        fs::write(
            &main_file,
            "use alpha: loc = {alpha};\nuse beta: loc = {beta};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write ambiguous imported plain-name fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI resolver fixture path should be utf-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let compact = stdout
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

        assert!(
            !output.status.success(),
            "CLI should fail in JSON mode when resolver rejects an ambiguous plain name"
        );
        assert!(
            stdout.contains(
                main_file
                    .to_str()
                    .expect("Temporary resolver fixture path should be valid UTF-8")
            ),
            "JSON resolver diagnostics should keep the exact source file for ambiguous plain names"
        );
        assert!(
            !compact.contains("\"file\":null"),
            "JSON resolver diagnostics for ambiguous plain names should never drop the file field"
        );
        assert!(
            compact.contains("\"line\":4"),
            "JSON resolver diagnostics should preserve the exact ambiguous line number"
        );
        assert!(
            compact.contains("\"column\":12"),
            "JSON resolver diagnostics should preserve the exact ambiguous plain-name column"
        );
        assert!(
            stdout.contains("name 'answer' is ambiguous in lexical scope"),
            "JSON resolver diagnostics should keep the exact ambiguous plain-name wording"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_parser_errors_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_parser_structured");
        fs::create_dir_all(&temp_root).expect("Should create temp parser fixture");
        fs::write(temp_root.join("00_good.fol"), "var ok = 1\n").expect("Should write good source");
        let bad_file = temp_root.join("10_bad.fol");
        fs::write(&bad_file, "run(1, 2)\n").expect("Should write invalid file-root source");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI parser fixture path should be utf-8"),
        ]);
        let json = parse_cli_json(&output);
        let diagnostic = &json["diagnostics"][0];

        assert!(!output.status.success(), "Parser fixture should fail in JSON mode");
        assert_eq!(json["error_count"], 1);
        assert_eq!(json["warning_count"], 0);
        assert_eq!(diagnostic["severity"], "Error");
        assert!(diagnostic["code"].as_str().is_some());
        assert_eq!(diagnostic["message"], "Executable calls are not allowed at file root");
        assert_eq!(
            diagnostic["location"]["file"],
            bad_file
                .to_str()
                .expect("Temporary parser fixture path should be valid UTF-8")
        );
        assert_eq!(diagnostic["location"]["line"], 1);
        assert_eq!(diagnostic["location"]["column"], 1);
        assert_eq!(diagnostic["location"]["length"], 3);
        assert_eq!(diagnostic["labels"].as_array().map(|items| items.len()), Some(1));
        assert_eq!(diagnostic["notes"].as_array().map(|items| items.len()), Some(0));
        assert_eq!(diagnostic["helps"].as_array().map(|items| items.len()), Some(0));
        assert_eq!(
            diagnostic["suggestions"].as_array().map(|items| items.len()),
            Some(0)
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_package_errors_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_package_structured");
        let app_root = temp_root.join("app");
        let loc_root = temp_root.join("formal_pkg");
        fs::create_dir_all(&app_root).expect("Should create app fixture root");
        fs::create_dir_all(&loc_root).expect("Should create loc target fixture root");
        fs::write(loc_root.join("build.fol"), "def root: loc = \"src\";\n")
            .expect("Should write formal package control file");
        let main_file = app_root.join("main.fol");
        fs::write(
            &main_file,
            "use formal: loc = {../formal_pkg};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write loc misuse fixture");

        let output = run_fol(&[
            "--json",
            app_root
                .to_str()
                .expect("CLI package fixture path should be utf-8"),
        ]);
        let json = parse_cli_json(&output);
        let diagnostic = &json["diagnostics"][0];

        assert!(!output.status.success(), "Package fixture should fail in JSON mode");
        assert_eq!(json["error_count"], 1);
        assert_eq!(diagnostic["severity"], "Error");
        assert!(diagnostic["code"].as_str().is_some());
        assert_eq!(
            diagnostic["location"]["file"],
            main_file
                .to_str()
                .expect("Temporary package fixture path should be valid UTF-8")
        );
        assert_eq!(diagnostic["location"]["line"], 1);
        assert_eq!(diagnostic["location"]["column"], 1);
        assert_eq!(diagnostic["labels"].as_array().map(|items| items.len()), Some(1));
        assert_eq!(diagnostic["notes"].as_array().map(|items| items.len()), Some(0));
        assert_eq!(diagnostic["helps"].as_array().map(|items| items.len()), Some(1));
        assert_eq!(
            diagnostic["helps"][0],
            "replace the import source kind with pkg for formal packages"
        );
        let message = diagnostic["message"]
            .as_str()
            .expect("Package diagnostic message should stay a string");
        assert!(message.contains("build.fol"));
        assert!(message.contains("pkg instead of loc"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_resolver_errors_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_resolver_structured");
        fs::create_dir_all(temp_root.join("alpha"))
            .expect("Should create first imported namespace fixture");
        fs::create_dir_all(temp_root.join("beta"))
            .expect("Should create second imported namespace fixture");
        fs::write(temp_root.join("alpha/values.fol"), "var[exp] answer: int = 1;\n")
            .expect("Should write first imported exported value fixture");
        fs::write(temp_root.join("beta/values.fol"), "var[exp] answer: int = 2;\n")
            .expect("Should write second imported exported value fixture");
        let main_file = temp_root.join("main.fol");
        fs::write(
            &main_file,
            "use alpha: loc = {alpha};\nuse beta: loc = {beta};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write ambiguous imported plain-name fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI resolver fixture path should be utf-8"),
        ]);
        let json = parse_cli_json(&output);
        let diagnostic = &json["diagnostics"][0];

        assert!(!output.status.success(), "Resolver fixture should fail in JSON mode");
        assert_eq!(json["error_count"], 1);
        assert_eq!(diagnostic["severity"], "Error");
        assert!(diagnostic["code"].as_str().is_some());
        assert_eq!(
            diagnostic["location"]["file"],
            main_file
                .to_str()
                .expect("Temporary resolver fixture path should be valid UTF-8")
        );
        assert_eq!(diagnostic["location"]["line"], 4);
        assert_eq!(diagnostic["location"]["column"], 12);
        assert_eq!(diagnostic["location"]["length"], 6);
        assert_eq!(diagnostic["labels"].as_array().map(|items| items.len()), Some(3));
        assert_eq!(diagnostic["labels"][1]["kind"], "Secondary");
        assert_eq!(diagnostic["labels"][2]["kind"], "Secondary");
        assert_eq!(
            diagnostic["labels"][1]["message"],
            "candidate value binding declaration"
        );
        assert_eq!(
            diagnostic["labels"][2]["message"],
            "candidate value binding declaration"
        );
        assert_eq!(diagnostic["notes"].as_array().map(|items| items.len()), Some(0));
        assert_eq!(diagnostic["helps"].as_array().map(|items| items.len()), Some(0));
        let message = diagnostic["message"]
            .as_str()
            .expect("Resolver diagnostic message should stay a string");
        assert!(message.contains("ResolverAmbiguousReference"));
        assert!(message.contains("name 'answer' is ambiguous in lexical scope"));
        assert!(message.contains("candidates:"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_resolver_errors_keep_help_for_missing_std_roots() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_resolver_std_help");
        fs::create_dir_all(&temp_root).expect("Should create resolver fixture root");
        fs::write(
            temp_root.join("main.fol"),
            "use fmt: std = {fmt};\nfun[] main(): int = {\n    return 0;\n}\n",
        )
        .expect("Should write missing std-root fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("Resolver fixture path should be valid UTF-8"),
        ]);
        let json = parse_cli_json(&output);
        let diagnostic = &json["diagnostics"][0];

        assert!(!output.status.success(), "Missing std-root fixture should fail");
        assert_eq!(diagnostic["helps"].as_array().map(|items| items.len()), Some(1));
        assert_eq!(diagnostic["helps"][0], "rerun with --std-root <DIR>");

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_resolver_errors_keep_notes_for_unsupported_import_kinds() {
        use std::fs;

        let temp_root = unique_temp_root("cli_json_resolver_unsupported_note");
        fs::create_dir_all(&temp_root).expect("Should create resolver fixture root");
        fs::write(temp_root.join("main.fol"), "use fmt: mod = {core::fmt};\n")
            .expect("Should write unsupported import fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("Resolver fixture path should be valid UTF-8"),
        ]);
        let json = parse_cli_json(&output);
        let diagnostic = &json["diagnostics"][0];

        assert!(!output.status.success(), "Unsupported import fixture should fail");
        assert_eq!(diagnostic["notes"].as_array().map(|items| items.len()), Some(1));
        assert_eq!(
            diagnostic["notes"][0],
            "supported import source kinds are loc, std, and pkg"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_human_resolver_errors_render_secondary_labels() {
        use std::fs;

        let temp_root = unique_temp_root("cli_human_resolver_labels");
        fs::create_dir_all(temp_root.join("alpha"))
            .expect("Should create first imported namespace fixture");
        fs::create_dir_all(temp_root.join("beta"))
            .expect("Should create second imported namespace fixture");
        fs::write(temp_root.join("alpha/values.fol"), "var[exp] answer: int = 1;\n")
            .expect("Should write first imported exported value fixture");
        fs::write(temp_root.join("beta/values.fol"), "var[exp] answer: int = 2;\n")
            .expect("Should write second imported exported value fixture");
        fs::write(
            temp_root.join("main.fol"),
            "use alpha: loc = {alpha};\nuse beta: loc = {beta};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write ambiguous imported plain-name fixture");

        let output = run_fol(&[
            temp_root
                .to_str()
                .expect("Resolver fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(!output.status.success(), "Ambiguous resolver fixture should fail");
        assert!(stdout.contains("error: ResolverAmbiguousReference"));
        assert!(stdout.contains("note:"));
        assert!(stdout.contains("candidate value binding declaration"));
        assert!(stdout.contains("alpha/values.fol"));
        assert!(stdout.contains("beta/values.fol"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_human_package_errors_render_help_guidance() {
        use std::fs;

        let temp_root = unique_temp_root("cli_human_package_help");
        let app_root = temp_root.join("app");
        let loc_root = temp_root.join("formal_pkg");
        fs::create_dir_all(&app_root).expect("Should create app fixture root");
        fs::create_dir_all(&loc_root).expect("Should create loc target fixture root");
        fs::write(loc_root.join("build.fol"), "def root: loc = \"src\";\n")
            .expect("Should write formal package control file");
        fs::write(
            app_root.join("main.fol"),
            "use formal: loc = {../formal_pkg};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write loc misuse fixture");

        let output = run_fol(&[
            app_root
                .to_str()
                .expect("Package fixture path should be valid UTF-8"),
        ]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(!output.status.success(), "Formal package loc misuse should fail");
        assert!(stdout.contains("error: ResolverInvalidInput"));
        assert!(stdout.contains("pkg instead of loc"));
        assert!(stdout.contains("help: replace the import source kind with pkg for formal packages"));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_typecheck_accepts_v1_programs_after_resolution() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_success");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI typecheck fixture");
        fs::write(
            temp_root.join("main.fol"),
            "var value: int = 1\nfun[] main(): int = {\n    return value;\n}\n",
        )
        .expect("Should write the successful typecheck fixture");

        let output = run_fol(&[temp_root
            .to_str()
            .expect("CLI typecheck fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            output.status.success(),
            "CLI should accept parse-clean, resolve-clean, type-correct V1 programs, got status {:?} and output:\n{}",
            output.status.code(),
            stdout,
        );
        assert!(
            stdout.contains("Compilation successful"),
            "Human CLI output should still report a successful compile after typechecking"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_typecheck_errors_fail_parse_clean_programs() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_error");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI typecheck error fixture");
        fs::write(temp_root.join("main.fol"), "var[bor] borrowed: int = 1\n")
            .expect("Should write the unsupported typecheck fixture");

        let output = run_fol(&[temp_root
            .to_str()
            .expect("CLI typecheck error fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !output.status.success(),
            "CLI should fail when typechecking rejects a parse-clean, resolve-clean program"
        );
        assert!(
            stdout.contains("borrowing binding semantics are part of the V3 systems milestone"),
            "CLI diagnostics should surface the typecheck unsupported message"
        );
        assert!(
            stdout.contains("main.fol"),
            "CLI diagnostics should preserve the failing source-unit path"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_typecheck_imported_symbol_mismatches_fail_full_chain() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_imported_symbol_error");
        let shared_root = temp_root.join("shared");
        let app_root = temp_root.join("app");
        fs::create_dir_all(&shared_root).expect("Should create shared fixture root");
        fs::create_dir_all(&app_root).expect("Should create app fixture root");
        fs::write(shared_root.join("lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write imported binding fixture");
        fs::write(
            app_root.join("main.fol"),
            "use shared: loc = {\"../shared\"};\nvar label: str = answer;\n",
        )
        .expect("Should write imported binding consumer fixture");

        let output = run_fol(&[app_root
            .to_str()
            .expect("CLI imported binding fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !output.status.success(),
            "CLI should fail when imported symbol typing mismatches in the entry package"
        );
        assert!(
            stdout.contains("initializer for 'label' expects"),
            "CLI should preserve the imported binding mismatch wording"
        );
        assert!(
            stdout.contains("main.fol"),
            "CLI should preserve the imported binding consumer path"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_typecheck_imported_aggregate_mismatches_fail_full_chain() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_imported_aggregate_error");
        let shared_root = temp_root.join("shared");
        let app_root = temp_root.join("app");
        fs::create_dir_all(&shared_root).expect("Should create shared fixture root");
        fs::create_dir_all(&app_root).expect("Should create app fixture root");
        fs::write(
            shared_root.join("types.fol"),
            "typ[exp] Meta: rec = {\n    ok: bol\n}\n\
             typ[exp] User: rec = {\n    meta: Meta\n}\n",
        )
        .expect("Should write imported aggregate fixture");
        fs::write(
            app_root.join("main.fol"),
            "use shared: loc = {\"../shared\"};\n\
             fun[] main(): shared::User = {\n\
                 return { meta = { ok = 1 } };\n\
             }\n",
        )
        .expect("Should write imported aggregate consumer fixture");

        let output = run_fol(&[app_root
            .to_str()
            .expect("CLI imported aggregate fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !output.status.success(),
            "CLI should fail when imported aggregate typing mismatches in the entry package"
        );
        assert!(
            stdout.contains("record field 'ok' expects"),
            "CLI should preserve the imported aggregate mismatch wording"
        );
        assert!(
            stdout.contains("main.fol"),
            "CLI should preserve the imported aggregate consumer path"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_typecheck_imported_optional_shell_mismatches_fail_full_chain() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_imported_shell_error");
        let shared_root = temp_root.join("shared");
        let app_root = temp_root.join("app");
        fs::create_dir_all(&shared_root).expect("Should create shared fixture root");
        fs::create_dir_all(&app_root).expect("Should create app fixture root");
        fs::write(
            shared_root.join("types.fol"),
            "typ[exp] MaybeText: opt[str];\n",
        )
        .expect("Should write imported shell fixture");
        fs::write(
            app_root.join("main.fol"),
            "use shared: loc = {\"../shared\"};\nvar value: int = 1;\nvar label: shared::MaybeText = value;\n",
        )
        .expect("Should write imported shell consumer fixture");

        let output = run_fol(&[app_root
            .to_str()
            .expect("CLI imported shell fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !output.status.success(),
            "CLI should fail when imported optional shell typing mismatches in the entry package"
        );
        assert!(
            stdout.contains("initializer for 'label' expects"),
            "CLI should preserve the imported shell mismatch wording"
        );
        assert!(
            stdout.contains("MaybeText") || stdout.contains("opt[str]"),
            "CLI should preserve the imported shell type identity"
        );
        assert!(
            stdout.contains("main.fol"),
            "CLI should preserve the imported shell consumer path"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_typecheck_errors_keep_structured_fields() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_error_json");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI typecheck JSON fixture");
        fs::write(temp_root.join("main.fol"), "var[bor] borrowed: int = 1\n")
            .expect("Should write the unsupported typecheck fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI typecheck JSON fixture path should be utf-8"),
        ]);
        let report = parse_cli_json(&output);
        let diagnostics = report["diagnostics"]
            .as_array()
            .expect("CLI JSON diagnostics should stay array-shaped");
        let first = diagnostics
            .first()
            .expect("CLI JSON diagnostics should include one typecheck error");

        assert!(
            !output.status.success(),
            "CLI should fail in JSON mode when typechecking rejects a parse-clean program"
        );
        assert_eq!(first["code"], "T1002");
        assert_eq!(first["location"]["line"], 1);
        assert_eq!(first["location"]["column"], 1);
        assert!(
            first["location"]["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("main.fol")),
            "CLI JSON diagnostics should preserve the failing source-unit path"
        );
        assert!(
            first["message"]
                .as_str()
                .is_some_and(|message| message.contains("borrowing binding semantics")),
            "CLI JSON diagnostics should preserve the typecheck failure message"
        );
        assert_eq!(first["labels"].as_array().map(|items| items.len()), Some(1));

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_typecheck_rejects_invalid_check_calls_full_chain() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_invalid_check");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI invalid check fixture");
        fs::write(
            temp_root.join("main.fol"),
            "fun[] main(): bol = {\n    return check(1);\n}\n",
        )
        .expect("Should write invalid check fixture");

        let output = run_fol(&[temp_root
            .to_str()
            .expect("CLI invalid check fixture path should be utf-8")]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            !output.status.success(),
            "CLI should fail when check(...) is used on a plain value"
        );
        assert!(
            stdout.contains("check requires an errorful routine result in V1"),
            "CLI diagnostics should preserve the invalid check wording"
        );
        assert!(
            stdout.contains("main.fol"),
            "CLI diagnostics should preserve the failing source-unit path"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_typecheck_pipe_or_fallback_mismatches_keep_exact_locations() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_pipe_or_json");
        fs::create_dir_all(&temp_root).expect("Should create temp CLI pipe-or JSON fixture");
        fs::write(
            temp_root.join("main.fol"),
            "fun[] load(): int / str = {\n\
                 report \"bad\";\n\
                 return 1;\n\
             }\n\
             fun[] main(): int = {\n\
                 return load() || \"fallback\";\n\
             }\n",
        )
        .expect("Should write pipe-or JSON fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI pipe-or JSON fixture path should be utf-8"),
        ]);
        let report = parse_cli_json(&output);
        let diagnostics = report["diagnostics"]
            .as_array()
            .expect("CLI JSON diagnostics should stay array-shaped");
        let first = diagnostics
            .first()
            .expect("CLI JSON diagnostics should include one typecheck error");

        assert!(
            !output.status.success(),
            "CLI should fail in JSON mode when a pipe-or fallback is incompatible"
        );
        assert_eq!(first["code"], "T1003");
        assert_eq!(first["location"]["line"], 6);
        assert_eq!(first["location"]["column"], 8);
        assert!(
            first["message"]
                .as_str()
                .is_some_and(|message| message.contains("recoverable-error fallback")),
            "CLI JSON diagnostics should preserve the fallback mismatch wording"
        );
        assert!(
            first["location"]["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("main.fol")),
            "CLI JSON diagnostics should preserve the failing source-unit path"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_typecheck_imported_binding_mismatches_keep_exact_locations() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_imported_binding_json");
        let shared_root = temp_root.join("shared");
        let app_root = temp_root.join("app");
        fs::create_dir_all(&shared_root).expect("Should create shared fixture root");
        fs::create_dir_all(&app_root).expect("Should create app fixture root");
        fs::write(shared_root.join("lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write imported binding fixture");
        fs::write(
            app_root.join("main.fol"),
            "use shared: loc = {\"../shared\"};\nvar label: str = answer;\n",
        )
        .expect("Should write imported binding consumer fixture");

        let output = run_fol(&[
            "--json",
            app_root
                .to_str()
                .expect("CLI imported binding fixture path should be utf-8"),
        ]);
        let report = parse_cli_json(&output);
        let first = report["diagnostics"][0].clone();

        assert!(!output.status.success(), "CLI should fail on imported binding mismatches");
        assert_eq!(first["code"], "T1003");
        assert_eq!(first["location"]["line"], 2);
        assert_eq!(first["location"]["column"], 18);
        assert_eq!(first["location"]["length"], 6);
        assert!(
            first["location"]["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("/app/main.fol")),
            "CLI JSON diagnostics should preserve the imported binding consumer path"
        );
        assert!(
            first["message"]
                .as_str()
                .is_some_and(|message| message.contains("initializer for 'label' expects")),
            "CLI JSON diagnostics should preserve the imported binding mismatch message"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_typecheck_nil_shell_errors_keep_exact_locations() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_nil_json");
        fs::create_dir_all(&temp_root).expect("Should create nil fixture root");
        fs::write(
            temp_root.join("main.fol"),
            "ali MaybeText: opt[str]\nvar label = nil\n",
        )
        .expect("Should write nil fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI nil fixture path should be utf-8"),
        ]);
        let report = parse_cli_json(&output);
        let first = report["diagnostics"][0].clone();

        assert!(!output.status.success(), "CLI should fail on unsupported nil shell contexts");
        assert_eq!(first["code"], "T1001");
        assert_eq!(first["location"]["line"], 2);
        assert_eq!(first["location"]["column"], 1);
        assert_eq!(first["location"]["length"], 3);
        assert!(
            first["location"]["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("/main.fol")),
            "CLI JSON diagnostics should preserve the nil fixture path"
        );
        assert!(
            first["message"].as_str().is_some_and(|message| {
                message.contains("nil literals require an expected opt[...] or err[...] shell type in V1")
            }),
            "CLI JSON diagnostics should preserve the nil shell message"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_typecheck_nested_record_mismatches_keep_exact_locations() {
        use std::fs;

        let temp_root = unique_temp_root("cli_typecheck_nested_record_json");
        fs::create_dir_all(&temp_root).expect("Should create nested record fixture root");
        fs::write(
            temp_root.join("main.fol"),
            "typ Meta: rec = {\n\
                 ok: bol\n\
             }\n\
             typ User: rec = {\n\
                 meta: Meta\n\
             }\n\
             fun[] main(): User = {\n\
                 return { meta = { ok = 1 } };\n\
             }\n",
        )
        .expect("Should write nested record mismatch fixture");

        let output = run_fol(&[
            "--json",
            temp_root
                .to_str()
                .expect("CLI nested record fixture path should be utf-8"),
        ]);
        let report = parse_cli_json(&output);
        let first = report["diagnostics"][0].clone();

        assert!(
            !output.status.success(),
            "CLI should fail on nested record mismatches"
        );
        assert_eq!(first["code"], "T1003");
        assert_eq!(first["location"]["line"], 8);
        assert_eq!(first["location"]["column"], 17);
        assert_eq!(first["location"]["length"], 1);
        assert!(
            first["location"]["file"]
                .as_str()
                .is_some_and(|file| file.ends_with("/main.fol")),
            "CLI JSON diagnostics should preserve the nested record fixture path"
        );
        assert!(
            first["message"]
                .as_str()
                .is_some_and(|message| message.contains("record field 'ok' expects")),
            "CLI JSON diagnostics should preserve the nested record mismatch message"
        );

        fs::remove_dir_all(&temp_root).ok();
    }

    #[test]
    fn test_cli_json_modern_compiler_errors_do_not_fall_back_to_unknown_codes() {
        use std::fs;

        let parser_root = unique_temp_root("cli_json_known_parser_code");
        fs::create_dir_all(&parser_root).expect("Should create parser fixture root");
        fs::write(parser_root.join("bad.fol"), "run(1, 2)\n").expect("Should write parser fixture");
        let parser_output = run_fol(&[
            "--json",
            parser_root
                .to_str()
                .expect("Parser fixture path should be valid UTF-8"),
        ]);
        let parser_json = parse_cli_json(&parser_output);
        assert_ne!(
            parser_json["diagnostics"][0]["code"],
            "E0000",
            "Parser diagnostics should use explicit structured parser codes"
        );

        let package_root = unique_temp_root("cli_json_known_package_code");
        let app_root = package_root.join("app");
        let loc_root = package_root.join("formal_pkg");
        fs::create_dir_all(&app_root).expect("Should create app fixture root");
        fs::create_dir_all(&loc_root).expect("Should create loc target fixture root");
        fs::write(loc_root.join("build.fol"), "def root: loc = \"src\";\n")
            .expect("Should write formal package control file");
        fs::write(
            app_root.join("main.fol"),
            "use formal: loc = {../formal_pkg};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write package fixture");
        let package_output = run_fol(&[
            "--json",
            app_root
                .to_str()
                .expect("Package fixture path should be valid UTF-8"),
        ]);
        let package_json = parse_cli_json(&package_output);
        assert_ne!(
            package_json["diagnostics"][0]["code"],
            "E0000",
            "Package diagnostics should use explicit structured package codes"
        );

        let resolver_root = unique_temp_root("cli_json_known_resolver_code");
        fs::create_dir_all(resolver_root.join("alpha"))
            .expect("Should create first imported namespace fixture");
        fs::create_dir_all(resolver_root.join("beta"))
            .expect("Should create second imported namespace fixture");
        fs::write(resolver_root.join("alpha/values.fol"), "var[exp] answer: int = 1;\n")
            .expect("Should write first imported exported value fixture");
        fs::write(resolver_root.join("beta/values.fol"), "var[exp] answer: int = 2;\n")
            .expect("Should write second imported exported value fixture");
        fs::write(
            resolver_root.join("main.fol"),
            "use alpha: loc = {alpha};\nuse beta: loc = {beta};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write resolver fixture");
        let resolver_output = run_fol(&[
            "--json",
            resolver_root
                .to_str()
                .expect("Resolver fixture path should be valid UTF-8"),
        ]);
        let resolver_json = parse_cli_json(&resolver_output);
        assert_ne!(
            resolver_json["diagnostics"][0]["code"],
            "E0000",
            "Resolver diagnostics should use explicit structured resolver codes"
        );

        fs::remove_dir_all(&parser_root).ok();
        fs::remove_dir_all(&package_root).ok();
        fs::remove_dir_all(&resolver_root).ok();
    }

    #[test]
    fn test_parser_error_locations_reach_diagnostics_outputs() {
        use fol_diagnostics::{DiagnosticLocation, DiagnosticReport, OutputFormat};
        use fol_lexer::lexer::stage3::Elements;
        use fol_lexer::token::KEYWORD;
        use fol_parser::ast::{AstParser, ParseError};
        use fol_stream::FileStream;

        let mut file_stream =
            FileStream::from_file("test/parser/simple_var.fol").expect("Should read test file");

        let mut lexer = Elements::init(&mut file_stream);
        lexer
            .set_key(KEYWORD::Illegal)
            .expect("Should force illegal token");

        let mut parser = AstParser::new();
        let mut diagnostics = DiagnosticReport::new();

        let parse_errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail on illegal token");

        for error in parse_errors {
            let location = error
                .as_any()
                .downcast_ref::<ParseError>()
                .map(|parse_error| DiagnosticLocation {
                    file: parse_error.file(),
                    line: parse_error.line(),
                    column: parse_error.column(),
                    length: Some(parse_error.length()),
                });

            diagnostics.add_error(error.as_ref(), location);
        }

        let human = diagnostics.output(OutputFormat::Human);
        assert!(
            human.contains("-->"),
            "Human diagnostics should include location"
        );
        assert!(
            human.contains("simple_var.fol"),
            "Human diagnostics should include source file"
        );

        let json = diagnostics.output(OutputFormat::Json);
        assert!(
            json.contains("\"line\""),
            "JSON diagnostics should include line field"
        );
        assert!(
            json.contains("\"column\""),
            "JSON diagnostics should include column field"
        );
    }

    #[test]
    fn test_parser_human_diagnostics_keep_snippet_shape() {
        use fol_diagnostics::{DiagnosticLocation, DiagnosticReport, OutputFormat};
        use fol_lexer::lexer::stage3::Elements;
        use fol_lexer::token::KEYWORD;
        use fol_parser::ast::{AstParser, ParseError};
        use fol_stream::FileStream;

        let mut file_stream =
            FileStream::from_file("test/parser/simple_var.fol").expect("Should read test file");

        let mut lexer = Elements::init(&mut file_stream);
        lexer
            .set_key(KEYWORD::Illegal)
            .expect("Should force illegal token");

        let mut parser = AstParser::new();
        let mut diagnostics = DiagnosticReport::new();
        let parse_errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail on illegal token");

        for error in parse_errors {
            let location = error
                .as_any()
                .downcast_ref::<ParseError>()
                .map(|parse_error| DiagnosticLocation {
                    file: parse_error.file(),
                    line: parse_error.line(),
                    column: parse_error.column(),
                    length: Some(parse_error.length()),
                });
            diagnostics.add_error(error.as_ref(), location);
        }

        let human = diagnostics.output(OutputFormat::Human);
        assert!(human.contains("| var x: int = 42;"));
        assert!(human.contains("^"));
        assert!(human.contains("simple_var.fol"));
    }

    #[test]
    fn test_multi_file_parser_errors_keep_second_file_locations() {
        use fol_diagnostics::{DiagnosticLocation, DiagnosticReport, OutputFormat};
        use fol_lexer::lexer::stage3::Elements;
        use fol_parser::ast::{AstParser, ParseError};
        use fol_stream::FileStream;
        use std::fs;

        let temp_root = unique_temp_root("parser_multifile_locations");
        let first = temp_root.join("00_good.fol");
        let second = temp_root.join("10_bad.fol");
        fs::create_dir_all(&temp_root).expect("Should create temp parser error fixture dir");
        fs::write(&first, "var ok = 1\n").expect("Should write first source");
        fs::write(&second, "\"unterminated").expect("Should write malformed second source");

        let mut file_stream = FileStream::from_folder(
            temp_root
                .to_str()
                .expect("Multi-file parser fixture path should be utf-8"),
        )
        .expect("Should build a multi-file stream");
        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Second source should surface a parser-visible error");

        let parse_error = errors
            .iter()
            .filter_map(|error| error.as_ref().as_any().downcast_ref::<ParseError>())
            .find(|error| {
                error
                    .file()
                    .as_deref()
                    .is_some_and(|path| path.ends_with("10_bad.fol"))
            })
            .expect("A parse error should point at the malformed second file");

        assert_eq!(
            parse_error.line(),
            1,
            "Second file should restart at line 1"
        );
        assert_eq!(
            parse_error.column(),
            1,
            "Second file should restart at column 1 for its first token"
        );

        let mut diagnostics = DiagnosticReport::new();
        diagnostics.add_error(
            parse_error,
            Some(DiagnosticLocation {
                file: parse_error.file(),
                line: parse_error.line(),
                column: parse_error.column(),
                length: Some(parse_error.length()),
            }),
        );
        let human = diagnostics.output(OutputFormat::Human);
        assert!(
            human.contains("10_bad.fol"),
            "Diagnostic output should name the second file that actually failed"
        );

        fs::remove_dir_all(&temp_root).ok();
    }
}
