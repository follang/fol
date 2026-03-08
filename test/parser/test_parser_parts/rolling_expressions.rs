use super::*;

#[test]
fn test_simple_rolling_expression_parses_in_return_position() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_rolling_expr.fol")
            .expect("Should read rolling expression fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept rolling expressions");

    let return_value = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| match node {
                AstNode::Return { value: Some(value) } => Some(value.as_ref().clone()),
                _ => None,
            })
            .expect("Program should contain return value"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(
            return_value,
            AstNode::Rolling { bindings, condition: None, .. }
                if bindings.len() == 1
                    && bindings[0].name == "x"
                    && matches!(bindings[0].iterable, AstNode::Identifier { ref name } if name == "items")
        ),
        "Return value should parse as rolling expression"
    );
}

#[test]
fn test_parenthesized_multi_binding_rolling_expression_parses() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_rolling_multi_binding.fol")
            .expect("Should read multi-binding rolling expression fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept parenthesized multi-binding rolling expressions");

    let return_value = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| match node {
                AstNode::Return { value: Some(value) } => Some(value.as_ref().clone()),
                _ => None,
            })
            .expect("Program should contain return value"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(
            return_value,
            AstNode::Rolling { bindings, .. }
                if bindings.len() == 2
                    && bindings[0].name == "x"
                    && bindings[1].name == "y"
        ),
        "Return value should keep both rolling binders"
    );
}

#[test]
fn test_rolling_expression_supports_optional_filter() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_rolling_filtered.fol")
            .expect("Should read filtered rolling expression fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept filtered rolling expressions");

    let return_value = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| match node {
                AstNode::Return { value: Some(value) } => Some(value.as_ref().clone()),
                _ => None,
            })
            .expect("Program should contain return value"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(
            return_value,
            AstNode::Rolling {
                condition: Some(_),
                ..
            }
        ),
        "Filtered rolling expression should keep the trailing condition"
    );
}

#[test]
fn test_rolling_expression_requires_in_keyword() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_rolling_missing_in.fol")
            .expect("Should read malformed rolling expression fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject rolling expressions without 'in'");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Expected 'in' in rolling binding"),
        "Malformed rolling expression should report missing 'in', got: {}",
        parse_error
    );
}

#[test]
fn test_rolling_expression_rejects_duplicate_binders() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_rolling_duplicate_binders.fol")
            .expect("Should read duplicate rolling binder fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate rolling binders");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Duplicate rolling binding 'x'"),
        "Duplicate rolling binders should report the repeated name, got: {}",
        parse_error
    );
}
