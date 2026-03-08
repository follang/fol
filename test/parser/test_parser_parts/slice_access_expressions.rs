use super::*;

#[test]
fn test_bounded_slice_expression_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_slice_expr.fol")
        .expect("Should read slice expression fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse bounded slice expressions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::Return { value: Some(value) }
                        if matches!(
                            value.as_ref(),
                            AstNode::SliceAccess { start, end, reverse, .. }
                            if !reverse && start.is_some() && end.is_some()
                        )
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
