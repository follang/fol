use super::*;

#[test]
fn test_function_inquiry_clause_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_inquiry_clause.fol")
        .expect("Should read function inquiry clause test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse inquiry clauses attached to functions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { name, inquiries, .. }
                    if name == "sum"
                        && inquiries.len() == 1
                        && matches!(&inquiries[0], AstNode::Inquiry { target, body } if target == "self" && body.len() == 3 && matches!(&body[0], AstNode::BinaryOp { .. }))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_procedure_inquiry_clause_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pro_inquiry_clause.fol")
        .expect("Should read procedure inquiry clause test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse inquiry clauses attached to procedures");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ProDecl { name, inquiries, .. }
                    if name == "store"
                        && inquiries.len() == 1
                        && matches!(&inquiries[0], AstNode::Inquiry { target, body } if target == "self" && body.len() == 2 && matches!(&body[0], AstNode::BinaryOp { .. }))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_duplicate_function_inquiry_clause_rejected() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_duplicate_inquiry_clause.fol")
            .expect("Should read duplicate function inquiry clause test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate inquiry clauses on functions");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Duplicate inquiry clause for 'self'"),
        "Expected duplicate inquiry error, got: {}",
        parse_error
    );
}

#[test]
fn test_duplicate_procedure_inquiry_clause_rejected() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_duplicate_inquiry_clause.fol")
            .expect("Should read duplicate procedure inquiry clause test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate inquiry clauses on procedures");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Duplicate inquiry clause for 'self'"),
        "Expected duplicate inquiry error, got: {}",
        parse_error
    );
}

#[test]
fn test_this_inquiry_clause_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_inquiry_this.fol")
        .expect("Should read this-target inquiry clause test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse 'where(this)' inquiry clauses");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, inquiries, .. }
                if name == "show"
                    && matches!(&inquiries[0], AstNode::Inquiry { target, body } if target == "this" && body.len() == 1)
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_distinct_inquiry_targets_can_coexist() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_inquiry_multi_target.fol")
        .expect("Should read multi-target inquiry clause test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should allow distinct inquiry targets");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { inquiries, .. }
                if inquiries.len() == 2
                    && matches!(&inquiries[0], AstNode::Inquiry { target, .. } if target == "self")
                    && matches!(&inquiries[1], AstNode::Inquiry { target, .. } if target == "this")
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_inquiry_clause_accepts_comma_separated_targets() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_inquiry_target_list.fol")
        .expect("Should read comma-separated inquiry target fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse comma-separated inquiry targets");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { name, inquiries, .. }
                    if name == "show"
                        && inquiries.len() == 2
                        && inquiries.iter().any(|node| matches!(node, AstNode::Inquiry { target, .. } if target == "self"))
                        && inquiries.iter().any(|node| matches!(node, AstNode::Inquiry { target, .. } if target == "this"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_inquiry_clause_accepts_semicolon_separated_targets() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_inquiry_target_semicolons.fol")
            .expect("Should read semicolon-separated inquiry target fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse semicolon-separated inquiry targets");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { name, inquiries, .. }
                    if name == "show"
                        && inquiries.len() == 2
                        && inquiries.iter().any(|node| matches!(node, AstNode::Inquiry { target, .. } if target == "self"))
                        && inquiries.iter().any(|node| matches!(node, AstNode::Inquiry { target, .. } if target == "this"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_inquiry_clause_accepts_flow_body() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_inquiry_flow_body.fol")
        .expect("Should read inquiry flow-body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied inquiry clauses");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, inquiries, .. }
                if name == "show"
                    && matches!(&inquiries[0], AstNode::Inquiry { target, body } if target == "self" && matches!(body.as_slice(), [AstNode::Identifier { name }] if name == "self"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_duplicate_inquiry_target_in_single_clause_rejected() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_duplicate_inquiry_target_list.fol")
            .expect("Should read duplicate inquiry target-list fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate targets in one inquiry clause");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Duplicate inquiry clause for 'self'"),
        "Expected duplicate inquiry target error, got: {}",
        parse_error
    );
}
