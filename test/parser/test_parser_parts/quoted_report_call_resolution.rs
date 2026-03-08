use super::*;

#[test]
fn test_function_custom_error_type_accepts_report_forward_quoted_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_quoted_call_result_ok.fol",
    )
    .expect("Should read forward quoted report call-result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward quoted report call result when return type is compatible",
    );
}

#[test]
fn test_function_custom_error_type_rejects_report_forward_quoted_call_result_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_quoted_call_result_mismatch.fol",
    )
    .expect("Should read forward quoted report call-result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward quoted report call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward quoted call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_procedure_custom_error_type_accepts_report_forward_quoted_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_quoted_call_result_ok.fol",
    )
    .expect("Should read forward quoted procedure report call-result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward quoted procedure report call result when return type is compatible",
    );
}

#[test]
fn test_procedure_custom_error_type_rejects_report_forward_quoted_call_result_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_quoted_call_result_mismatch.fol",
    )
    .expect("Should read forward quoted procedure report call-result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward quoted procedure report call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward quoted procedure call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_function_custom_error_type_accepts_report_forward_keyword_named_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_keyword_call_result_ok.fol",
    )
    .expect("Should read forward keyword-named report call-result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward keyword-named report call result when return type is compatible",
    );
}

#[test]
fn test_function_custom_error_type_rejects_report_forward_keyword_named_call_result_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_keyword_call_result_mismatch.fol",
    )
    .expect("Should read forward keyword-named report call-result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward keyword-named report call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward keyword-named call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_procedure_custom_error_type_accepts_report_forward_keyword_named_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_keyword_call_result_ok.fol",
    )
    .expect("Should read forward keyword-named procedure report call-result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward keyword-named procedure report call result when return type is compatible",
    );
}

#[test]
fn test_procedure_custom_error_type_rejects_report_forward_keyword_named_call_result_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_keyword_call_result_mismatch.fol",
    )
    .expect("Should read forward keyword-named procedure report call-result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward keyword-named procedure report call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward keyword-named procedure call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_function_custom_error_type_accepts_report_forward_single_quoted_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_single_quoted_call_result_ok.fol",
    )
    .expect("Should read forward single-quoted report call-result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward single-quoted report call result when return type is compatible",
    );
}

#[test]
fn test_function_custom_error_type_rejects_report_forward_single_quoted_call_result_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_single_quoted_call_result_mismatch.fol",
    )
    .expect("Should read forward single-quoted report call-result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward single-quoted report call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward single-quoted call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_procedure_custom_error_type_accepts_report_forward_single_quoted_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_single_quoted_call_result_ok.fol",
    )
    .expect("Should read forward single-quoted procedure report call-result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward single-quoted procedure report call result when return type is compatible",
    );
}

#[test]
fn test_procedure_custom_error_type_rejects_report_forward_single_quoted_call_result_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_single_quoted_call_result_mismatch.fol",
    )
    .expect("Should read forward single-quoted procedure report call-result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward single-quoted procedure report call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward single-quoted procedure call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_function_custom_error_type_accepts_report_forward_quoted_method_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_quoted_method_call_result_ok.fol",
    )
    .expect("Should read forward quoted report method-call result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward quoted report method call result when return type is compatible",
    );
}

#[test]
fn test_function_custom_error_type_rejects_report_forward_quoted_method_call_result_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_quoted_method_call_result_mismatch.fol",
    )
    .expect("Should read forward quoted report method-call result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward quoted report method call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward quoted method-call result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_procedure_custom_error_type_accepts_report_forward_quoted_method_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_quoted_method_call_result_ok.fol",
    )
    .expect("Should read forward quoted procedure report method-call result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward quoted procedure report method call result when return type is compatible",
    );
}

#[test]
fn test_procedure_custom_error_type_rejects_report_forward_quoted_method_call_result_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_quoted_method_call_result_mismatch.fol",
    )
    .expect("Should read forward quoted procedure report method-call result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward quoted procedure report method call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward quoted procedure method-call result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_function_custom_error_type_accepts_report_forward_keyword_named_method_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_keyword_method_call_result_ok.fol",
    )
    .expect("Should read forward keyword-named report method-call result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward keyword-named report method call result when return type is compatible",
    );
}

#[test]
fn test_function_custom_error_type_rejects_report_forward_keyword_named_method_call_result_mismatch(
) {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_keyword_method_call_result_mismatch.fol",
    )
    .expect("Should read forward keyword-named report method-call result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward keyword-named report method call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward keyword-named method-call result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_procedure_custom_error_type_accepts_report_forward_keyword_named_method_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_keyword_method_call_result_ok.fol",
    )
    .expect(
        "Should read forward keyword-named procedure report method-call result compatible fixture",
    );

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward keyword-named procedure report method call result when return type is compatible",
    );
}

#[test]
fn test_procedure_custom_error_type_rejects_report_forward_keyword_named_method_call_result_mismatch(
) {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_keyword_method_call_result_mismatch.fol",
    )
    .expect(
        "Should read forward keyword-named procedure report method-call result mismatch fixture",
    );

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward keyword-named procedure report method call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward keyword-named procedure method-call result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_function_custom_error_type_accepts_report_forward_single_quoted_method_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_single_quoted_method_call_result_ok.fol",
    )
    .expect("Should read forward single-quoted report method-call result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward single-quoted report method call result when return type is compatible",
    );
}

#[test]
fn test_function_custom_error_type_rejects_report_forward_single_quoted_method_call_result_mismatch(
) {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_single_quoted_method_call_result_mismatch.fol",
    )
    .expect("Should read forward single-quoted report method-call result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward single-quoted report method call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward single-quoted method-call result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_procedure_custom_error_type_accepts_report_forward_single_quoted_method_call_result() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_single_quoted_method_call_result_ok.fol",
    )
    .expect(
        "Should read forward single-quoted procedure report method-call result compatible fixture",
    );

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward single-quoted procedure report method call result when return type is compatible",
    );
}

#[test]
fn test_procedure_custom_error_type_rejects_report_forward_single_quoted_method_call_result_mismatch(
) {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_single_quoted_method_call_result_mismatch.fol",
    )
    .expect(
        "Should read forward single-quoted procedure report method-call result mismatch fixture",
    );

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward single-quoted procedure report method call result when return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward single-quoted procedure method-call result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_function_custom_error_type_accepts_report_forward_overloaded_quoted_call_result_by_arity() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_quoted_overload_arity_ok.fol",
    )
    .expect("Should read forward overloaded quoted report call-result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward overloaded quoted report call result when selected arity return type is compatible",
    );
}

#[test]
fn test_function_custom_error_type_rejects_report_forward_overloaded_quoted_call_result_by_arity_mismatch(
) {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_quoted_overload_arity_mismatch.fol",
    )
    .expect("Should read forward overloaded quoted report call-result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward overloaded quoted report call result when selected arity return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward overloaded quoted call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_procedure_custom_error_type_accepts_report_forward_overloaded_quoted_call_result_by_arity()
{
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_quoted_overload_arity_ok.fol",
    )
    .expect("Should read forward overloaded quoted procedure report call-result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward overloaded quoted procedure report call result when selected arity return type is compatible",
    );
}

#[test]
fn test_procedure_custom_error_type_rejects_report_forward_overloaded_quoted_call_result_by_arity_mismatch(
) {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_quoted_overload_arity_mismatch.fol",
    )
    .expect("Should read forward overloaded quoted procedure report call-result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward overloaded quoted procedure report call result when selected arity return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward overloaded quoted procedure call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_function_custom_error_type_accepts_report_forward_overloaded_keyword_call_result_by_arity()
{
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_keyword_overload_arity_ok.fol",
    )
    .expect("Should read forward overloaded keyword-named report call-result compatible fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward overloaded keyword-named report call result when selected arity return type is compatible",
    );
}

#[test]
fn test_function_custom_error_type_rejects_report_forward_overloaded_keyword_call_result_by_arity_mismatch(
) {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_forward_keyword_overload_arity_mismatch.fol",
    )
    .expect("Should read forward overloaded keyword-named report call-result mismatch fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward overloaded keyword-named report call result when selected arity return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward overloaded keyword-named call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}

#[test]
fn test_procedure_custom_error_type_accepts_report_forward_overloaded_keyword_call_result_by_arity()
{
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_keyword_overload_arity_ok.fol",
    )
    .expect(
        "Should read forward overloaded keyword-named procedure report call-result compatible fixture",
    );

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept forward overloaded keyword-named procedure report call result when selected arity return type is compatible",
    );
}

#[test]
fn test_procedure_custom_error_type_rejects_report_forward_overloaded_keyword_call_result_by_arity_mismatch(
) {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_pro_error_type_report_forward_keyword_overload_arity_mismatch.fol",
    )
    .expect(
        "Should read forward overloaded keyword-named procedure report call-result mismatch fixture",
    );

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject forward overloaded keyword-named procedure report call result when selected arity return type is incompatible",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Reported expression type")
            && parse_error
                .to_string()
                .contains("incompatible with routine error type"),
        "Forward overloaded keyword-named procedure call-result mismatch should report incompatible expression type, got: {}",
        parse_error
    );
}
