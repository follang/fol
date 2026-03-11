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

#[cfg(test)]
mod integration_tests {

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
        let temp_root = std::env::temp_dir()
            .join(format!("fol_stream_lexer_order_{}_{}", std::process::id(), stamp));

        fs::create_dir_all(temp_root.join("10_alpha")).expect("Should create alpha fixture dir");
        fs::create_dir_all(temp_root.join("20_beta")).expect("Should create beta fixture dir");
        fs::write(temp_root.join("00_root.fol"), "root_token").expect("Should write root fixture");
        fs::write(temp_root.join("10_alpha/entry.fol"), "alpha_token")
            .expect("Should write alpha fixture");
        fs::write(temp_root.join("20_beta/entry.fol"), "beta_token")
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
}
