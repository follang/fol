use super::*;

#[test]
fn test_top_level_leading_dot_builtin_call_statement() {
    let mut file_stream = FileStream::from_file("test/parser/simple_top_level_dot_echo.fol")
        .expect("Should read top-level leading-dot builtin fixture");
    let mut tokens = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    let ast = parser
        .parse(&mut tokens)
        .expect("Parser should accept top-level leading-dot builtin calls");

    assert!(
        matches!(
            ast,
            AstNode::Program { declarations }
                if declarations.iter().any(|node| matches!(
                    node,
                    AstNode::FunctionCall { name, args }
                        if name == "echo"
                            && matches!(args.as_slice(), [AstNode::Literal(Literal::String(_))])
                ))
        ),
        "Expected top-level leading-dot builtin call to lower as FunctionCall"
    );
}

#[test]
fn test_leading_dot_builtin_call_expression() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_dot_len_expr.fol")
        .expect("Should read leading-dot builtin expression fixture");
    let mut tokens = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    let ast = parser
        .parse(&mut tokens)
        .expect("Parser should accept leading-dot builtin calls in expression position");

    let has_dot_expr = match ast {
        AstNode::Program { declarations } => declarations.iter().any(|node| {
            matches!(
                node,
                AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::VarDecl {
                            name,
                            value: Some(value),
                            ..
                        } if name == "size"
                            && matches!(
                                value.as_ref(),
                                AstNode::FunctionCall { name, args }
                                    if name == "len"
                                        && matches!(args.as_slice(), [AstNode::Identifier { name }] if name == "items")
                            )
                    ))
            )
        }),
        _ => false,
    };

    assert!(
        has_dot_expr,
        "Expected leading-dot builtin expression to lower as FunctionCall"
    );
}
