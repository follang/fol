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

#[cfg(test)]
mod integration_tests {
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
        use fol_diagnostics::{DiagnosticReport, OutputFormat};
        use fol_lexer::lexer::stage3::Elements;
        use fol_parser::ast::AstParser;
        use fol_stream::FileStream;

        // Test the complete compilation pipeline
        let mut diagnostics = DiagnosticReport::new();

        // 1. Stream
        let mut file_stream = match FileStream::from_file("test/parser/simple_var.fol") {
            Ok(stream) => stream,
            Err(e) => {
                diagnostics.add_error(e.as_ref(), None);
                panic!("Should read test file");
            }
        };

        // 2. Lexer
        let mut lexer = Elements::init(&mut file_stream);

        // 3. Parser
        let mut parser = AstParser::new();
        match parser.parse(&mut lexer) {
            Ok(_ast) => {
                println!("Full pipeline integration successful!");
            }
            Err(parse_errors) => {
                for error in parse_errors {
                    diagnostics.add_error(error.as_ref(), None);
                }
                println!("Parser stage completed with errors (may be expected for minimal parser)");
            }
        }

        // 4. Diagnostics
        let output = diagnostics.output(OutputFormat::Human);
        println!("Diagnostic output: {}", output);

        // The test passes if we get through the pipeline without crashing
        assert!(
            diagnostics.error_count <= diagnostics.diagnostics.len(),
            "Diagnostic counters should remain consistent"
        );
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
