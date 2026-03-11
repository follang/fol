use super::*;

#[test]
fn test_call_argument_illegal_token_reports_offending_token_location() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_call_illegal_string_arg.fol")
            .expect("Should read illegal call-argument fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject illegal tokens inside call arguments");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Parser encountered illegal token"),
        "Illegal call-argument token should report an explicit illegal-token diagnostic, got: {}",
        parse_error
    );
    assert_eq!(
        parse_error.line(),
        2,
        "Illegal call-argument token should report the call site line"
    );
    assert!(
        parse_error.column() > 0,
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
