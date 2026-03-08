use super::*;

#[test]
fn test_anonymous_function_expression_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_anonymous_expr.fol")
        .expect("Should read anonymous function fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse anonymous function expressions");

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
                            AstNode::AnonymousFun { params, return_type, body, .. }
                            if params.len() == 2
                                && matches!(return_type, Some(FolType::Int { .. }))
                                && !body.is_empty()
                        )
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_anonymous_function_immediate_invocation_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_anonymous_invoke_expr.fol")
            .expect("Should read anonymous invoke fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse immediate anonymous invocation");

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
                            AstNode::Invoke { callee, args }
                            if args.len() == 2
                                && matches!(callee.as_ref(), AstNode::AnonymousFun { .. })
                        )
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
