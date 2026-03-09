use super::*;

#[test]
fn test_indexed_callee_invocation_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_indexed_invoke.fol")
        .expect("Should read indexed invoke fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse indexed callee invocation");

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
                            if args.len() == 1
                                && matches!(callee.as_ref(), AstNode::IndexAccess { .. })
                        )
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_nested_invoke_expression_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_nested_invoke.fol")
        .expect("Should read nested invoke fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse nested invoke expressions");

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
                            if args.len() == 1
                                && matches!(callee.as_ref(), AstNode::Invoke { .. })
                        )
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_indexed_invoke_statement_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_indexed_invoke_stmt.fol")
            .expect("Should read indexed invoke statement fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse indexed invoke statements");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(stmt, AstNode::Invoke { .. }))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_grouped_invoke_statement_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_grouped_invoke_stmt.fol")
            .expect("Should read grouped invoke statement fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse grouped invoke statements");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(stmt, AstNode::Invoke { .. }))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_prefix_availability_invoke_expression_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_prefix_availability_invoke_expr.fol")
            .expect("Should read prefix availability invoke expression fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse prefix availability invoke expressions");

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
                            if args.len() == 1
                                && matches!(callee.as_ref(), AstNode::AvailabilityAccess { .. })
                        )
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_prefix_availability_invoke_statement_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_prefix_availability_invoke_stmt.fol")
            .expect("Should read prefix availability invoke statement fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse prefix availability invoke statements");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::Invoke { callee, args }
                        if args.len() == 1
                            && matches!(callee.as_ref(), AstNode::AvailabilityAccess { .. })
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_suffix_index_availability_invoke_statement_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_suffix_index_availability_invoke_stmt.fol")
            .expect("Should read suffix index availability invoke statement fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse suffix index availability invoke statements");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::Invoke { callee, args }
                        if args.len() == 1
                            && matches!(callee.as_ref(), AstNode::AvailabilityAccess { target }
                                if matches!(target.as_ref(), AstNode::IndexAccess { .. }))
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
