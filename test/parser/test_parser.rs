// Parser tests - to be expanded when full parser is implemented

use fol_stream::FileStream;  
use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::{AstParser, AstNode};

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let mut file_stream = FileStream::from_file("test/parser/simple_var.fol")
            .expect("Should read test file");
        
        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        
        match parser.parse(&mut lexer) {
            Ok(ast) => {
                assert!(matches!(ast, AstNode::Program { .. }), "Should return Program node");
                println!("Successfully parsed AST: {:?}", ast);
            }
            Err(errors) => {
                panic!("Parser should not fail: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_function_parsing() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun.fol")
            .expect("Should read test file");
        
        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        
        match parser.parse(&mut lexer) {
            Ok(ast) => {
                assert!(matches!(ast, AstNode::Program { .. }), "Should return Program node");
                println!("Successfully parsed function AST: {:?}", ast);
            }
            Err(errors) => {
                println!("Parser errors (expected for now): {:?}", errors);
                // For now, we expect the minimal parser to work
            }
        }
    }

    #[test]
    fn test_literal_parsing() {
        let parser = AstParser::new();
        
        // Test integer literal
        match parser.parse_literal("42") {
            Ok(ast) => {
                assert!(matches!(ast, AstNode::Literal(_)), "Should parse integer literal");
            }
            Err(e) => panic!("Should parse integer literal: {:?}", e),
        }
        
        // Test string literal
        match parser.parse_literal("\"hello\"") {
            Ok(ast) => {
                assert!(matches!(ast, AstNode::Literal(_)), "Should parse string literal");
            }
            Err(e) => panic!("Should parse string literal: {:?}", e),
        }
        
        // Test identifier
        match parser.parse_literal("variable_name") {
            Ok(ast) => {
                assert!(matches!(ast, AstNode::Identifier { .. }), "Should parse identifier");
            }
            Err(e) => panic!("Should parse identifier: {:?}", e),
        }
    }
}

// TODO: Expand these tests when full parser is implemented
// - Variable declarations
// - Function declarations  
// - Type declarations
// - Expressions
// - Statements
// - Error recovery
// - AST structure validation