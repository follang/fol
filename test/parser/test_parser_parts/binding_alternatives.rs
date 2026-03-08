use super::*;

#[test]
fn test_export_binding_alternative_parses_top_level() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_plus_var.fol").expect("Should read +var fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept +var binding alternative");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "exported"
                        && options.contains(&fol_parser::ast::VarOption::Export)
                        && !options.contains(&fol_parser::ast::VarOption::Normal)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_mutable_binding_alternative_parses_top_level() {
    let mut file_stream = FileStream::from_file("test/parser/simple_tilde_var.fol")
        .expect("Should read ~var fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept ~var binding alternative");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "counter"
                        && options.contains(&fol_parser::ast::VarOption::Mutable)
                        && !options.contains(&fol_parser::ast::VarOption::Immutable)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_hidden_binding_alternative_parses_top_level() {
    let mut file_stream = FileStream::from_file("test/parser/simple_minus_var.fol")
        .expect("Should read -var fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept -var binding alternative");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "hidden"
                        && options.contains(&fol_parser::ast::VarOption::Hidden)
                        && !options.contains(&fol_parser::ast::VarOption::Normal)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_static_binding_alternative_parses_top_level() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_bang_var.fol").expect("Should read !var fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept !var binding alternative");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "cached"
                        && options.contains(&fol_parser::ast::VarOption::Static)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_reactive_binding_alternative_parses_top_level() {
    let mut file_stream = FileStream::from_file("test/parser/simple_query_var.fol")
        .expect("Should read ?var fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept ?var binding alternative");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "derived"
                        && options.contains(&fol_parser::ast::VarOption::Reactive)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_heap_binding_alternative_parses_top_level() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_at_var.fol").expect("Should read @var fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept @var binding alternative");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "heap_value"
                        && options.contains(&fol_parser::ast::VarOption::New)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
