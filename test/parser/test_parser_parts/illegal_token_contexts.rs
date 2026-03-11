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
