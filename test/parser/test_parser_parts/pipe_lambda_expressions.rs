use super::*;

#[test]
fn test_pipe_lambda_expression_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pipe_lambda_expr.fol")
        .expect("Should read pipe lambda fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse expression-bodied pipe lambdas");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { body, .. }
                if body.iter().any(|stmt| matches!(
                    stmt,
                    AstNode::Return { value: Some(value) }
                    if matches!(value.as_ref(), AstNode::AnonymousFun { params, .. } if params.len() == 1 && params[0].name == "x")
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_typed_pipe_lambda_expression_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_typed_expr.fol")
            .expect("Should read typed pipe lambda fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse typed pipe lambdas");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { body, .. }
                if body.iter().any(|stmt| matches!(
                    stmt,
                    AstNode::Return { value: Some(value) }
                    if matches!(
                        value.as_ref(),
                        AstNode::AnonymousFun { params, .. }
                        if params.len() == 2
                            && matches!(params[0].param_type, FolType::Int { .. })
                            && matches!(params[1].param_type, FolType::Named { ref name } if name == "pkg::Score")
                    )
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}
