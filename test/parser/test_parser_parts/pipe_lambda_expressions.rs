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

#[test]
fn test_zero_and_multi_param_pipe_lambda_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_multi_expr.fol")
            .expect("Should read zero/multi pipe lambda fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse zero and multi-param pipe lambdas");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { body, .. }
                if body.iter().filter(|stmt| matches!(
                    stmt,
                    AstNode::Return { value: Some(value) }
                    if matches!(value.as_ref(), AstNode::AnonymousFun { params, .. } if params.is_empty())
                )).count() == 1
                    && body.iter().filter(|stmt| matches!(
                        stmt,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::AnonymousFun { params, .. } if params.len() == 2)
                    )).count() == 1
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_block_bodied_pipe_lambda_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_block.fol")
            .expect("Should read block-bodied pipe lambda fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse block-bodied pipe lambdas");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { body, .. }
                if body.iter().any(|stmt| matches!(
                    stmt,
                    AstNode::Return { value: Some(value) }
                    if matches!(value.as_ref(), AstNode::AnonymousFun { body, .. } if body.iter().any(|node| matches!(node, AstNode::Return { .. })))
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_pipe_lambda_capture_lists_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pipe_lambda_capture_expr.fol")
        .expect("Should read pipe lambda capture fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse capture lists on pipe lambdas");

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
                        AstNode::AnonymousFun { captures, params, .. }
                        if captures == &vec!["left".to_string(), "right".to_string()]
                            && params.len() == 1
                            && params[0].name == "x"
                    )
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_pipe_lambda_inquiry_clauses_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pipe_lambda_inquiry_expr.fol")
        .expect("Should read pipe lambda inquiry fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve inquiry clauses on block pipe lambdas");

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
                        AstNode::AnonymousFun { inquiries, .. }
                        if inquiries.len() == 1
                            && matches!(&inquiries[0], AstNode::Inquiry { target, body } if target == "self" && body.len() == 1)
                    )
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_pipe_lambda_rejects_duplicate_parameters() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_duplicate_param.fol")
            .expect("Should read duplicate pipe lambda parameter fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate pipe lambda parameters");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Duplicate parameter name 'x'"),
        "Expected duplicate pipe lambda parameter error, got: {}",
        parse_error
    );
}

#[test]
fn test_pipe_lambda_supports_grouped_parameters() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_grouped_params.fol")
            .expect("Should read grouped pipe lambda parameter fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse grouped pipe lambda parameters");

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
                        if params.len() == 3
                            && matches!(params[0].param_type, FolType::Int { .. })
                            && matches!(params[1].param_type, FolType::Int { .. })
                            && matches!(params[2].param_type, FolType::Named { ref name } if name == "str")
                    )
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_pipe_lambda_supports_default_parameters() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_default_params.fol")
            .expect("Should read default pipe lambda parameter fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse default pipe lambda parameters");

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
                        if params.len() == 3
                            && params[0].default.is_some()
                            && params[1].default.is_none()
                            && params[2].default.is_none()
                    )
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_pipe_lambda_supports_variadic_parameters() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_variadic_params.fol")
            .expect("Should read variadic pipe lambda fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse variadic pipe lambda parameters");

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
                            && matches!(params[1].param_type, FolType::Sequence { .. })
                    )
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_pipe_lambda_rejects_non_last_variadic_parameter() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_variadic_not_last.fol")
            .expect("Should read non-last variadic pipe lambda fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject non-last variadic pipe lambda parameters");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Variadic parameter must be the last parameter"),
        "Expected non-last variadic pipe lambda error, got: {}",
        parse_error
    );
}

#[test]
fn test_pipe_lambda_rejects_variadic_default_values() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_variadic_default_forbidden.fol")
            .expect("Should read variadic default pipe lambda fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject default values on variadic pipe lambda parameters");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Variadic parameters cannot have default values"),
        "Expected variadic-default pipe lambda error, got: {}",
        parse_error
    );
}

#[test]
fn test_pipe_lambda_marks_borrowable_parameters() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pipe_lambda_borrowable_params.fol")
            .expect("Should read borrowable pipe lambda parameter fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should mark borrowable pipe lambda parameters");

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
                            && params[0].name == "BUF"
                            && params[0].is_borrowable
                            && !params[1].is_borrowable
                    )
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}
