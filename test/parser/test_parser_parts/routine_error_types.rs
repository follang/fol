use super::*;

    #[test]
    fn test_when_has_case_missing_close_paren_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_when_has_missing_close.fol")
                .expect("Should read malformed when-has fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject when-has case missing closing ')'");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Expected ')' after has member"),
            "Malformed when-has case should report missing close paren, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            3,
            "Malformed when-has case parse error should point to the case line"
        );
    }

    #[test]
    fn test_function_declaration_missing_bracket_close_in_parameter_type_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_bracket_types_missing_close.fol")
                .expect("Should read malformed bracketed function type test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when function parameter type is missing closing ']'",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Expected closing ']' in type reference"),
            "Malformed bracketed function type should report missing close bracket, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            1,
            "Bracketed function type parse error should point to the signature line"
        );
    }

    #[test]
    fn test_procedure_declaration_header_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_pro.fol").expect("Should read test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse procedure declaration");

        let procedure_decl = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::ProDecl {
                        name,
                        params,
                        return_type,
                        body,
                        ..
                    } = node
                    {
                        Some((
                            name.clone(),
                            params.len(),
                            return_type.is_some(),
                            body.len(),
                        ))
                    } else {
                        None
                    }
                })
                .expect("Program should include procedure declaration"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(procedure_decl.0, "update");
        assert_eq!(procedure_decl.1, 2, "Procedure should have two parameters");
        assert!(procedure_decl.2, "Procedure should have return type");
        assert!(
            procedure_decl.3 > 0,
            "Procedure body should include parsed statements"
        );
    }

    #[test]
    fn test_function_declaration_error_type_parsing() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_error_type.fol")
            .expect("Should read function error type test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse function error type signature");

        let function_decl = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::FunDecl {
                        name,
                        return_type,
                        error_type,
                        ..
                    } = node
                    {
                        Some((name.clone(), return_type.clone(), error_type.clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should include function declaration"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(function_decl.0, "read_data");
        assert!(
            matches!(function_decl.1, Some(FolType::Named { name }) if name == "str"),
            "Function should parse return type in first ':' slot"
        );
        assert!(
            matches!(function_decl.2, Some(FolType::Named { name }) if name == "io_err"),
            "Function should parse error type in second ':' slot"
        );
    }

    #[test]
    fn test_procedure_declaration_error_type_parsing() {
        let mut file_stream = FileStream::from_file("test/parser/simple_pro_error_type.fol")
            .expect("Should read procedure error type test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse procedure error type signature");

        let procedure_decl = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::ProDecl {
                        name,
                        return_type,
                        error_type,
                        ..
                    } = node
                    {
                        Some((name.clone(), return_type.clone(), error_type.clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should include procedure declaration"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(procedure_decl.0, "write_data");
        assert!(
            matches!(procedure_decl.1, Some(FolType::Int { size: None, signed: true })),
            "Procedure should parse return type in first ':' slot"
        );
        assert!(
            matches!(procedure_decl.2, Some(FolType::Named { name }) if name == "io_err"),
            "Procedure should parse error type in second ':' slot"
        );
    }

    #[test]
    fn test_function_declaration_missing_error_type_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_missing_type.fol")
                .expect("Should read malformed function error type file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when second ':' has no error type");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Expected type reference"),
            "Missing error type after second ':' should report type reference error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            1,
            "Missing error type parse error should point to signature line"
        );
    }

    #[test]
    fn test_function_custom_error_type_requires_report_value() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_no_arg.fol")
                .expect("Should read malformed custom-error report file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when custom-error routine has 'report' with no value");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("must report exactly one error value"),
            "Custom-error routine should enforce report value arity, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_with_two_values() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_two_args.fol")
                .expect("Should read malformed custom-error report two-args file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when custom-error routine reports two values");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("must report exactly one error value"),
            "Custom-error routine should reject report with two values, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_compatible_report_literal() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_literal_ok.fol")
                .expect("Should read compatible custom-error report file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept report literal compatible with custom error type");
    }

    #[test]
    fn test_function_custom_error_type_rejects_incompatible_report_literal() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_literal_mismatch.fol")
                .expect("Should read incompatible custom-error report file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when report literal is incompatible with custom error type",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("incompatible with routine error type"),
            "Custom-error routine should reject incompatible report literal type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_boolean_report_literal() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_bool_ok.fol")
                .expect("Should read boolean-compatible custom-error report file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept boolean report literal for bol error type");
    }

    #[test]
    fn test_function_custom_error_type_rejects_integer_report_for_boolean_error_type() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_bool_mismatch.fol")
                .expect("Should read boolean mismatch custom-error report file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject integer report literal for bol error type");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("incompatible with routine error type"),
            "Boolean error type should reject integer report literal, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_compatible_report_identifier() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_identifier_ok.fol")
                .expect("Should read compatible custom-error report identifier file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept report identifier compatible with custom error type");
    }

    #[test]
    fn test_function_custom_error_type_rejects_incompatible_report_identifier() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_identifier_mismatch.fol",
        )
        .expect("Should read incompatible custom-error report identifier file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when report identifier is incompatible with custom error type",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported identifier")
                && first_message.contains("incompatible with routine error type"),
            "Custom-error routine should reject incompatible report identifier type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_numeric_family_report_identifier() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_numeric_family_ok.fol")
                .expect("Should read numeric-family compatible report identifier file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept numeric-family compatible report identifier");
    }

    #[test]
    fn test_function_custom_error_type_rejects_numeric_to_boolean_report_identifier() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_numeric_family_mismatch.fol",
        )
        .expect("Should read numeric-to-boolean mismatch report identifier file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject numeric report identifier for boolean error type");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported identifier")
                && first_message.contains("incompatible with routine error type"),
            "Numeric-to-boolean mismatch should report incompatible identifier type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_compatible_report_expression() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_expression_ok.fol")
                .expect("Should read compatible custom-error report expression file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept report expression compatible with custom error type");
    }

    #[test]
    fn test_function_custom_error_type_rejects_incompatible_report_expression() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_expression_mismatch.fol",
        )
        .expect("Should read incompatible custom-error report expression file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject report expression incompatible with custom error type",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Expression mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_report_local_inferred_from_expression() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_expr_local_ok.fol")
                .expect("Should read expression-inferred local report compatible file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept report local inferred from expression compatible with custom error type");
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_local_inferred_from_expression_mismatch() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_expr_local_mismatch.fol",
        )
        .expect("Should read expression-inferred local report mismatch file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject report local inferred from expression incompatible with custom error type");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported identifier")
                && first_message.contains("incompatible with routine error type"),
            "Expression-inferred local mismatch should report incompatible identifier type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_unknown_report_identifier() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_unknown_identifier.fol",
        )
        .expect("Should read unknown report identifier fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject unknown report identifier in custom-error routine");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Unknown reported identifier 'missing_err'"),
            "Unknown report identifier should produce explicit diagnostic, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_unknown_identifier_inside_report_expression() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_unknown_identifier_expression.fol",
        )
        .expect("Should read unknown report identifier expression fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject unknown identifier inside report expression in custom-error routine",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Unknown reported identifier 'missing_err'"),
            "Unknown identifier inside report expression should produce explicit diagnostic, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_compatible_report_identifier() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_pro_error_type_report_identifier_ok.fol")
                .expect("Should read compatible custom-error procedure report identifier file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept procedure report identifier compatible with custom error type",
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_incompatible_report_identifier() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_identifier_mismatch.fol",
        )
        .expect("Should read incompatible custom-error procedure report identifier file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject procedure report identifier incompatible with custom error type",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported identifier")
                && first_message.contains("incompatible with routine error type"),
            "Procedure custom-error routine should reject incompatible report identifier type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_compatible_report_expression() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_pro_error_type_report_expression_ok.fol")
                .expect("Should read compatible custom-error procedure report expression file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept procedure report expression compatible with custom error type",
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_incompatible_report_expression() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_expression_mismatch.fol",
        )
        .expect("Should read incompatible custom-error procedure report expression file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject procedure report expression incompatible with custom error type",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Procedure expression mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_report_local_inferred_from_expression() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_pro_error_type_report_inferred_local_ok.fol")
                .expect("Should read compatible custom-error procedure inferred-local report file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept procedure report local inferred from expression compatible with custom error type",
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_local_inferred_from_expression_mismatch() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_inferred_local_mismatch.fol",
        )
        .expect("Should read incompatible custom-error procedure inferred-local report file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject procedure report local inferred from expression incompatible with custom error type",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported identifier")
                && first_message.contains("incompatible with routine error type"),
            "Procedure expression-inferred local mismatch should report incompatible identifier type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_report_call_result_compatible_with_error_type() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_call_result_ok.fol")
                .expect("Should read compatible report call-result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept report call result compatible with custom error type");
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_call_result_incompatible_with_error_type() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_call_result_mismatch.fol",
        )
        .expect("Should read incompatible report call-result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject report call result incompatible with custom error type",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Call-result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_report_forward_call_result_compatible_with_error_type(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_forward_call_result_ok.fol",
        )
        .expect("Should read compatible forward report call-result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept report call result compatible with custom error type when callee is declared later",
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_forward_call_result_incompatible_with_error_type(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_forward_call_result_mismatch.fol",
        )
        .expect("Should read incompatible forward report call-result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject report call result incompatible with custom error type when callee is declared later",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Forward call-result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_report_call_result_compatible_with_error_type() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_pro_error_type_report_call_result_ok.fol")
                .expect("Should read compatible procedure report call-result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept procedure report call result compatible with custom error type",
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_call_result_incompatible_with_error_type() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_call_result_mismatch.fol",
        )
        .expect("Should read incompatible procedure report call-result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject procedure report call result incompatible with custom error type",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Procedure call-result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_report_forward_call_result_compatible_with_error_type(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_forward_call_result_ok.fol",
        )
        .expect("Should read compatible forward procedure report call-result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept procedure report call result compatible with custom error type when callee is declared later",
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_forward_call_result_incompatible_with_error_type(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_forward_call_result_mismatch.fol",
        )
        .expect("Should read incompatible forward procedure report call-result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject procedure report call result incompatible with custom error type when callee is declared later",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Forward procedure call-result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_unknown_called_routine() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_unknown_call.fol")
                .expect("Should read unknown report routine-call fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject unknown called routine in report expression for custom-error routine",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Unknown reported routine 'missing_err_source'"),
            "Unknown called routine in report should produce explicit diagnostic, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_unknown_called_routine() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_pro_error_type_report_unknown_call.fol")
                .expect("Should read unknown procedure report routine-call fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject unknown called routine in procedure report expression for custom-error routine",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Unknown reported routine 'missing_err_source'"),
            "Unknown called routine in procedure report should produce explicit diagnostic, got: {}",
            first_message
        );
    }

