use super::*;

#[test]
fn test_top_level_lab_declaration_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_lab_decl.fol")
        .expect("Should read top-level lab declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse top-level lab declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::LabDecl { name, options, value, .. }
                    if name == "label"
                        && options.contains(&fol_parser::ast::VarOption::Immutable)
                        && value.is_some()
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_lab_declaration_parsing_inside_function_bodies() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_lab_decl.fol")
        .expect("Should read function-body lab declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse lab declarations inside function bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(stmt, AstNode::LabDecl { name, .. } if name == "label"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
