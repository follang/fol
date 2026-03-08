use super::*;

#[test]
fn test_prefix_availability_expression_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_prefix_availability_expr.fol")
            .expect("Should read availability fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse prefix availability expressions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::AvailabilityAccess { target }
                            if matches!(target.as_ref(), AstNode::IndexAccess { .. }))
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
