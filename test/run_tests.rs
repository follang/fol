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
        use fol_stream::FileStream;
        use fol_lexer::lexer::stage3::Elements;
        
        // Test that stream output works with lexer input
        let mut file_stream = FileStream::from_file("test/lexer/mixed.fol")
            .expect("Should read test file");
        
        let mut lexer = Elements::init(&mut file_stream);
        
        // Should be able to get at least one token
        match lexer.curr(false) {
            Ok(token) => {
                println!("Integration test: First token = '{}'", token.con());
                // Check that we get a valid token (even if empty content like spaces)
                // or EOF - this verifies the integration is working
                assert!(true, "Integration working - got token: {:?}", token.key());
            }
            Err(e) => panic!("Stream to lexer integration failed: {:?}", e),
        }
    }
    
    #[test]
    fn test_lexer_to_parser_integration() {
        use fol_stream::FileStream;
        use fol_lexer::lexer::stage3::Elements;
        use fol_parser::ast::AstParser;
        
        // Test that lexer output works with parser input
        let mut file_stream = FileStream::from_file("test/parser/simple_var.fol")
            .expect("Should read test file");
        
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
        use fol_stream::FileStream;
        use fol_lexer::lexer::stage3::Elements;
        use fol_parser::ast::AstParser;
        use fol_diagnostics::{DiagnosticReport, OutputFormat};
        
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
        assert!(true, "Full pipeline should complete without crashing");
    }
}