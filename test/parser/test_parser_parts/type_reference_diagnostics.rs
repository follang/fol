use super::*;

fn first_parse_error_message(path: &str) -> String {
    let mut file_stream = FileStream::from_file(path).expect("Should read parser fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail for malformed type-reference fixture");

    errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError")
        .to_string()
}

#[test]
fn test_any_none_never_missing_close_report_type_reference_close() {
    for path in [
        "test/parser/simple_any_type_missing_close.fol",
        "test/parser/simple_none_type_missing_close.fol",
        "test/parser/simple_never_type_missing_close.fol",
    ] {
        let message = first_parse_error_message(path);
        assert!(
            message.contains("Expected closing ']' in type reference"),
            "Expected normalized missing-close diagnostic for fixture {path}, got: {message}",
        );
    }
}

#[test]
fn test_scalar_type_missing_close_report_type_reference_close() {
    for path in [
        "test/parser/simple_int_type_missing_close.fol",
        "test/parser/simple_float_type_missing_close.fol",
        "test/parser/simple_char_type_missing_close.fol",
    ] {
        let message = first_parse_error_message(path);
        assert!(
            message.contains("Expected closing ']' in type reference"),
            "Expected normalized missing-close diagnostic for fixture {path}, got: {message}",
        );
    }
}

#[test]
fn test_optional_multiple_union_missing_close_report_type_reference_close() {
    for path in [
        "test/parser/simple_opt_type_missing_close.fol",
        "test/parser/simple_mul_type_missing_close.fol",
        "test/parser/simple_uni_type_missing_close.fol",
    ] {
        let message = first_parse_error_message(path);
        assert!(
            message.contains("Expected closing ']' in type reference"),
            "Expected normalized missing-close diagnostic for fixture {path}, got: {message}",
        );
    }
}
