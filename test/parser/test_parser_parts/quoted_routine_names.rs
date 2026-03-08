use super::*;

#[test]
fn test_quoted_routine_names_parse_in_function_declarations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_quoted_routine_names.fol")
        .expect("Should read quoted routine name test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted routine names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::FunDecl { name, .. } if name == "$")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::FunDecl { name, .. } if name == "%")
            }));
        }
        _ => panic!("Expected program node"),
    }
}
