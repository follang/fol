use super::*;

fn first_parse_error_message(path: &str) -> String {
    let mut file_stream = FileStream::from_file(path).expect("Should read parser fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail for diagnostic consistency fixture");

    errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError")
        .to_string()
}

#[test]
fn test_parser_owned_unknown_option_diagnostics_name_the_surface() {
    for (path, expected) in [
        ("test/parser/simple_use_unknown_option.fol", "Unknown use option"),
        (
            "test/parser/simple_imp_unknown_option.fol",
            "Unknown implementation option",
        ),
        (
            "test/parser/simple_std_unknown_options.fol",
            "Unknown standard option",
        ),
        ("test/parser/simple_typ_options_unknown.fol", "Unknown type option"),
        (
            "test/parser/simple_routine_options_unknown.fol",
            "Unknown routine option",
        ),
        (
            "test/parser/simple_binding_unknown_option.fol",
            "Unknown binding option",
        ),
        (
            "test/parser/simple_def_unknown_option.fol",
            "Unknown definition option",
        ),
        (
            "test/parser/simple_seg_unknown_option.fol",
            "Unknown segment option",
        ),
    ] {
        let message = first_parse_error_message(path);
        assert!(
            message.contains(expected),
            "Expected explicit parser-owned unknown-option diagnostic for fixture {path}, got: {message}",
        );
    }
}
