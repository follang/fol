use super::*;

#[test]
fn test_var_binding_empty_options_preserve_defaults() {
    let mut file_stream = FileStream::from_file("test/parser/simple_var_empty_options.fol")
        .expect("Should read var empty-options fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept empty binding options");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "counter"
                        && options.contains(&fol_parser::ast::VarOption::Mutable)
                        && options.contains(&fol_parser::ast::VarOption::Normal)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_binding_mutability_options_override_defaults() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_binding_mutability_options.fol")
            .expect("Should read binding mutability options fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept explicit binding mutability options");

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
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "message"
                        && options.contains(&fol_parser::ast::VarOption::Immutable)
                        && !options.contains(&fol_parser::ast::VarOption::Mutable)
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "answer"
                        && options.contains(&fol_parser::ast::VarOption::Immutable)
                        && !options.contains(&fol_parser::ast::VarOption::Mutable)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_binding_visibility_word_options_override_defaults() {
    let mut file_stream = FileStream::from_file("test/parser/simple_binding_visibility_words.fol")
        .expect("Should read binding visibility-word options fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept word-form binding visibility options");

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
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "hidden"
                        && options.contains(&fol_parser::ast::VarOption::Hidden)
                        && !options.contains(&fol_parser::ast::VarOption::Normal)
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "local"
                        && options.contains(&fol_parser::ast::VarOption::Normal)
                        && !options.contains(&fol_parser::ast::VarOption::Export)
                        && !options.contains(&fol_parser::ast::VarOption::Hidden)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_binding_symbol_options_override_defaults() {
    let mut file_stream = FileStream::from_file("test/parser/simple_binding_symbol_options.fol")
        .expect("Should read binding symbol options fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept symbolic binding options");

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
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "exported"
                        && options.contains(&fol_parser::ast::VarOption::Export)
                        && !options.contains(&fol_parser::ast::VarOption::Normal)
                )
            }));
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
fn test_binding_storage_and_ownership_options_parse() {
    let mut file_stream = FileStream::from_file("test/parser/simple_binding_storage_options.fol")
        .expect("Should read binding storage options fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept storage and ownership binding options");

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
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "derived"
                        && options.contains(&fol_parser::ast::VarOption::Reactive)
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "heap_value"
                        && options.contains(&fol_parser::ast::VarOption::New)
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { name, options, .. }
                    if name == "borrowed"
                        && options.contains(&fol_parser::ast::VarOption::Borrowing)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
