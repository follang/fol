use super::*;

#[test]
fn test_select_statement_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_select_stmt.fol")
        .expect("Should read select statement fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse select statements");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ProDecl { body, .. }
                        if body.iter().any(|stmt| matches!(
                            stmt,
                            AstNode::Select { binding, body, .. }
                                if binding.as_deref() == Some("c") && !body.is_empty()
                        ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_select_statement_without_binding_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_select_stmt_no_binding.fol")
            .expect("Should read select statement without binding fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse binding-free select statements");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ProDecl { body, .. }
                        if body.iter().any(|stmt| matches!(
                            stmt,
                            AstNode::Select { binding, .. } if binding.is_none()
                        ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_select_pipe_stage_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_pipe_select_stage.fol")
        .expect("Should read pipe select-stage fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse select stages on pipes");

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
            op: fol_parser::ast::BinaryOperator::Pipe,
            right,
            ..
        } if matches!(right.as_ref(), AstNode::Select { binding, .. } if binding.as_deref() == Some("c"))
    ));
}
