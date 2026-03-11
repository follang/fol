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

#[test]
fn test_alias_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_alias_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal alias names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(line, 1, "Illegal alias names should report the declaration line");
    assert!(column > 0, "Illegal alias names should retain a concrete source column");
}

#[test]
fn test_type_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_typ_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal type names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(line, 1, "Illegal type names should report the declaration line");
    assert!(column > 0, "Illegal type names should retain a concrete source column");
}

#[test]
fn test_segment_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_seg_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal segment names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(line, 1, "Illegal segment names should report the declaration line");
    assert!(column > 0, "Illegal segment names should retain a concrete source column");
}

#[test]
fn test_implementation_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_imp_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal implementation names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 1,
        "Illegal implementation names should report the declaration line"
    );
    assert!(
        column > 0,
        "Illegal implementation names should retain a concrete source column"
    );
}

#[test]
fn test_standard_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_std_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal standard names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(line, 1, "Illegal standard names should report the declaration line");
    assert!(
        column > 0,
        "Illegal standard names should retain a concrete source column"
    );
}

#[test]
fn test_definition_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_def_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal definition names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 1,
        "Illegal definition names should report the declaration line"
    );
    assert!(
        column > 0,
        "Illegal definition names should retain a concrete source column"
    );
}

#[test]
fn test_function_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal function names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(line, 1, "Illegal function names should report the declaration line");
    assert!(
        column > 0,
        "Illegal function names should retain a concrete source column"
    );
}

#[test]
fn test_logical_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_log_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal logical names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(line, 1, "Illegal logical names should report the declaration line");
    assert!(
        column > 0,
        "Illegal logical names should retain a concrete source column"
    );
}

#[test]
fn test_procedure_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_pro_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal procedure names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 1,
        "Illegal procedure names should report the declaration line"
    );
    assert!(
        column > 0,
        "Illegal procedure names should retain a concrete source column"
    );
}

#[test]
fn test_type_entry_variant_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_typ_entry_illegal_variant_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal type entry variant names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal type entry variant names should report the variant line"
    );
    assert!(
        column > 0,
        "Illegal type entry variant names should retain a concrete source column"
    );
}

#[test]
fn test_type_record_field_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_typ_record_illegal_field_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal type record field names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal type record field names should report the field line"
    );
    assert!(
        column > 0,
        "Illegal type record field names should retain a concrete source column"
    );
}

#[test]
fn test_use_declaration_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_use_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal use names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(line, 1, "Illegal use names should report the declaration line");
    assert!(column > 0, "Illegal use names should retain a concrete source column");
}

#[test]
fn test_inquiry_target_illegal_segment_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_inquiry_illegal_target_segment.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal inquiry target segments should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 3,
        "Illegal inquiry target segments should report the inquiry clause line"
    );
    assert!(
        column > 0,
        "Illegal inquiry target segments should retain a concrete source column"
    );
}

#[test]
fn test_assignment_target_illegal_field_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_assignment_illegal_field_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal assignment-target field names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal assignment-target field names should report the assignment line"
    );
    assert!(
        column > 0,
        "Illegal assignment-target field names should retain a concrete source column"
    );
}

#[test]
fn test_method_call_illegal_method_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_method_call_illegal_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal method names should surface as explicit illegal-token diagnostics, got: {}",
        message
    );
    assert_eq!(line, 2, "Illegal method names should report the call line");
    assert!(
        column > 0,
        "Illegal method names should retain a concrete source column"
    );
}

#[test]
fn test_rolling_binding_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_rolling_illegal_binding_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal rolling binder names should report an explicit illegal-token diagnostic, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal rolling binder names should anchor to the rolling expression line"
    );
    assert!(
        column > 0,
        "Illegal rolling binder names should retain a concrete source column"
    );
}

#[test]
fn test_access_capture_illegal_binding_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_access_capture_illegal_binding.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal access-capture bindings should report an explicit illegal-token diagnostic, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal access-capture bindings should anchor to the access expression line"
    );
    assert!(
        column > 0,
        "Illegal access-capture bindings should retain a concrete source column"
    );
}

#[test]
fn test_pipe_lambda_illegal_parameter_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_pipe_lambda_illegal_param.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal pipe-lambda parameters should report an explicit illegal-token diagnostic, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal pipe-lambda parameters should anchor to the lambda line"
    );
    assert!(
        column > 0,
        "Illegal pipe-lambda parameters should retain a concrete source column"
    );
}

#[test]
fn test_routine_capture_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_illegal_capture_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal routine captures should report an explicit illegal-token diagnostic, got: {}",
        message
    );
    assert_eq!(
        line, 2,
        "Illegal routine captures should anchor to the routine header line"
    );
    assert!(
        column > 0,
        "Illegal routine captures should retain a concrete source column"
    );
}

#[test]
fn test_routine_parameter_illegal_name_reports_offending_token_location() {
    let (message, line, column) =
        parse_error_snapshot("test/parser/simple_fun_illegal_param_name.fol");

    assert!(
        message.contains("Parser encountered illegal token"),
        "Illegal routine parameter names should report an explicit illegal-token diagnostic, got: {}",
        message
    );
    assert_eq!(
        line, 1,
        "Illegal routine parameter names should anchor to the routine header line"
    );
    assert!(
        column > 0,
        "Illegal routine parameter names should retain a concrete source column"
    );
}
