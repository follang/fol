use super::*;
use fol_parser::ast::UseOption;

#[test]
fn test_use_declaration_accepts_visibility_options() {
    let mut file_stream = FileStream::from_file("test/parser/simple_use_visibility_options.fol")
        .expect("Should read use visibility options fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept use visibility options");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::UseDecl { name, options, .. }
                    if name == "pub_math"
                        && options == &vec![UseOption::Export]
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::UseDecl { name, options, .. }
                    if name == "hid_math"
                        && options == &vec![UseOption::Hidden]
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::UseDecl { name, options, .. }
                    if name == "nor_math"
                        && options == &vec![UseOption::Normal]
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_accepts_symbolic_visibility_options() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_use_symbol_visibility_options.fol")
            .expect("Should read symbolic use visibility options fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept symbolic use visibility options");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::UseDecl { name, options, .. }
                    if name == "pub_math"
                        && options == &vec![UseOption::Export]
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::UseDecl { name, options, .. }
                    if name == "hid_math"
                        && options == &vec![UseOption::Hidden]
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_accepts_multiple_visibility_options() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_use_multi_options.fol")
            .expect("Should read multi-option use fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept multiple use options");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::UseDecl { name, options, .. }
                    if name == "math"
                        && options == &vec![UseOption::Export, UseOption::Normal]
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
