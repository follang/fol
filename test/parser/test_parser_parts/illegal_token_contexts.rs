use super::*;

fn parse_error_snapshot(path: &str) -> (String, usize, usize) {
    let mut file_stream = FileStream::from_file(path).expect("Should read parser error fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject malformed tokens in the fixture");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    (
        parse_error.to_string(),
        parse_error.line(),
        parse_error.column(),
    )
}

#[test]
fn test_call_argument_illegal_token_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_call_illegal_string_arg.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal call-argument token should report an explicit illegal-token diagnostic, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal call-argument token should report the call site line"
    );
    assert!(
        column > 0,
        "Illegal call-argument token should retain a concrete source column"
    );
}

#[test]
fn test_type_reference_illegal_token_reports_offending_token_location() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_param_illegal_type_ref.fol")
        .expect("Should read illegal type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject illegal tokens inside type references");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Parser encountered illegal token"),
        "Illegal type-reference token should report an explicit illegal-token diagnostic, got: {}",
        parse_error
    );
    assert_eq!(
        parse_error.line(),
        1,
        "Illegal type-reference token should report the signature line"
    );
    assert!(
        parse_error.column() > 0,
        "Illegal type-reference token should retain a concrete source column"
    );
}

#[test]
fn test_container_element_illegal_token_reports_offending_token_location() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_container_illegal_element.fol")
        .expect("Should read illegal container-element fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject illegal tokens inside container literals");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Parser encountered illegal token"),
        "Illegal container-element token should report an explicit illegal-token diagnostic, got: {}",
        parse_error
    );
    assert_eq!(
        parse_error.line(),
        2,
        "Illegal container-element token should report the container literal line"
    );
    assert!(
        parse_error.column() > 0,
        "Illegal container-element token should retain a concrete source column"
    );
}

#[test]
fn test_record_initializer_value_illegal_token_reports_offending_token_location() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_record_init_illegal_value.fol")
            .expect("Should read illegal record-init fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject illegal tokens inside record initializer values");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Parser encountered illegal token"),
        "Illegal record-initializer token should report an explicit illegal-token diagnostic, got: {}",
        parse_error
    );
    assert_eq!(
        parse_error.line(),
        2,
        "Illegal record-initializer token should report the record literal line"
    );
    assert!(
        parse_error.column() > 0,
        "Illegal record-initializer token should retain a concrete source column"
    );
}

#[test]
fn test_return_expression_illegal_token_reports_offending_token_location() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_return_illegal_value.fol")
        .expect("Should read illegal return-expression fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject illegal tokens inside return expressions");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Parser encountered illegal token"),
        "Illegal return-expression token should report an explicit illegal-token diagnostic, got: {}",
        parse_error
    );
    assert_eq!(
        parse_error.line(),
        2,
        "Illegal return-expression token should report the return line"
    );
    assert!(
        parse_error.column() > 0,
        "Illegal return-expression token should retain a concrete source column"
    );
}

#[test]
fn test_parameter_default_illegal_token_reports_offending_token_location() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_param_default_illegal_value.fol")
            .expect("Should read illegal parameter-default fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject illegal tokens inside parameter default values");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Parser encountered illegal token"),
        "Illegal parameter-default token should report an explicit illegal-token diagnostic, got: {}",
        parse_error
    );
    assert_eq!(
        parse_error.line(),
        1,
        "Illegal parameter-default token should report the signature line"
    );
    assert!(
        parse_error.column() > 0,
        "Illegal parameter-default token should retain a concrete source column"
    );
}

#[test]
fn test_unterminated_backtick_comment_in_call_reports_offending_token_location() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_call_unterminated_backtick_comment.fol")
            .expect("Should read unterminated backtick-comment fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject unterminated backtick comments inside call arguments");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Parser encountered illegal token"),
        "Malformed backtick comments should surface as explicit illegal-token diagnostics, got: {}",
        parse_error
    );
    assert_eq!(
        parse_error.line(),
        4,
        "Malformed backtick comments should anchor to the offending comment start line"
    );
    assert!(
        parse_error.column() > 0,
        "Malformed backtick comments should retain a concrete source column"
    );
}

#[test]
fn test_unterminated_slash_block_comment_in_call_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_call_unterminated_block_comment.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Malformed slash block comments should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 4,
        "Malformed slash block comments should anchor to the offending comment start line"
    );
    assert!(
        column > 0,
        "Malformed slash block comments should retain a concrete source column"
    );
}

#[test]
fn test_dot_builtin_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_dot_builtin_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal builtin-call names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal builtin-call names should report the builtin call line"
    );
    assert!(
        column > 0,
        "Illegal builtin-call names should retain a concrete source column"
    );
}

#[test]
fn test_postfix_member_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_field_access_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal postfix member names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal postfix member names should report the member access line"
    );
    assert!(
        column > 0,
        "Illegal postfix member names should retain a concrete source column"
    );
}
