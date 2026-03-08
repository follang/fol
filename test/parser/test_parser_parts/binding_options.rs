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
