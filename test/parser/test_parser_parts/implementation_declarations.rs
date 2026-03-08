use super::*;

#[test]
fn test_basic_implementation_declaration_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_basic.fol")
        .expect("Should read implementation declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse basic implementation declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ImpDecl { name, target, body, generics }
                    if name == "Self"
                        && generics.is_empty()
                        && matches!(target, FolType::Named { name } if name == "ID")
                        && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, .. } if name == "ready"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_declaration_parsing_inside_function_bodies() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_imp_basic.fol")
        .expect("Should read function-body implementation declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse implementation declarations inside function bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(stmt, AstNode::ImpDecl { name, .. } if name == "Local"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
