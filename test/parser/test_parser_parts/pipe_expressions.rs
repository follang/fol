use super::*;
use fol_parser::ast::BinaryOperator;

#[test]
fn test_single_pipe_expression_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_pipe_expr.fol")
        .expect("Should read pipe expression fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse single pipe expressions");

    let return_value = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| match node {
                AstNode::FunDecl { body, .. } => body.iter().find_map(|stmt| match stmt {
                    AstNode::Return { value: Some(value) } => Some(value.as_ref().clone()),
                    _ => None,
                }),
                _ => None,
            })
            .expect("Expected return statement"),
        _ => panic!("Expected program node"),
    };

    assert!(matches!(
        return_value,
        AstNode::BinaryOp { op: BinaryOperator::Pipe, .. }
    ));
}

#[test]
fn test_pipe_expression_supports_if_call_stage() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_pipe_if_stage.fol")
        .expect("Should read pipe-if stage fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse if(...) stages on pipes");

    let return_value = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| match node {
                AstNode::FunDecl { body, .. } => body.iter().find_map(|stmt| match stmt {
                    AstNode::Return { value: Some(value) } => Some(value.as_ref().clone()),
                    _ => None,
                }),
                _ => None,
            })
            .expect("Expected return statement"),
        _ => panic!("Expected program node"),
    };

    assert!(matches!(
        return_value,
        AstNode::BinaryOp {
            op: BinaryOperator::Pipe,
            right,
            ..
        } if matches!(right.as_ref(), AstNode::FunctionCall { name, .. } if name == "if")
    ));
}

#[test]
fn test_pipe_expression_supports_bare_builtin_stage() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_pipe_builtin_stage.fol")
        .expect("Should read pipe builtin stage fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse bare builtin stages on pipes");

    let return_value = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| match node {
                AstNode::FunDecl { body, .. } => body.iter().find_map(|stmt| match stmt {
                    AstNode::Return { value: Some(value) } => Some(value.as_ref().clone()),
                    _ => None,
                }),
                _ => None,
            })
            .expect("Expected return statement"),
        _ => panic!("Expected program node"),
    };

    assert!(matches!(
        return_value,
        AstNode::BinaryOp {
            op: BinaryOperator::Pipe,
            right,
            ..
        } if matches!(right.as_ref(), AstNode::FunctionCall { name, .. } if name == "panic")
    ));
}

#[test]
fn test_pipe_expression_supports_return_stage() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_pipe_return_stage.fol")
        .expect("Should read pipe return stage fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse return stages on pipes");

    let return_value = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| match node {
                AstNode::FunDecl { body, .. } => body.iter().find_map(|stmt| match stmt {
                    AstNode::Return { value: Some(value) } => Some(value.as_ref().clone()),
                    _ => None,
                }),
                _ => None,
            })
            .expect("Expected return statement"),
        _ => panic!("Expected program node"),
    };

    assert!(matches!(
        return_value,
        AstNode::BinaryOp {
            op: BinaryOperator::PipeOr,
            right,
            ..
        } if matches!(right.as_ref(), AstNode::Return { value: Some(_) })
    ));
}

#[test]
fn test_pipe_expression_rejects_missing_rhs() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_pipe_missing_rhs.fol")
        .expect("Should read malformed pipe fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject missing pipe RHS");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Expected expression after '|'"),
        "Expected missing pipe RHS error, got: {}",
        parse_error
    );
}

#[test]
fn test_pipe_or_expression_rejects_missing_rhs() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_pipe_or_missing_rhs.fol")
        .expect("Should read malformed pipe-or fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject missing pipe-or RHS");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Expected expression after '||'"),
        "Expected missing pipe-or RHS error, got: {}",
        parse_error
    );
}

#[test]
fn test_double_pipe_expression_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_pipe_or_expr.fol")
        .expect("Should read pipe-or expression fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse double-pipe expressions");

    let return_value = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| match node {
                AstNode::FunDecl { body, .. } => body.iter().find_map(|stmt| match stmt {
                    AstNode::Return { value: Some(value) } => Some(value.as_ref().clone()),
                    _ => None,
                }),
                _ => None,
            })
            .expect("Expected return statement"),
        _ => panic!("Expected program node"),
    };

    assert!(matches!(
        return_value,
        AstNode::BinaryOp { op: BinaryOperator::PipeOr, .. }
    ));
}
