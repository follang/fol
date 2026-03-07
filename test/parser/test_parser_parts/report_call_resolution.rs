use super::*;

    #[test]
    fn test_function_custom_error_type_rejects_report_call_with_unknown_arity() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_unknown_arity_call.fol",
        )
        .expect("Should read unknown-arity report routine-call fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject report routine call with unknown arity in custom-error routine",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Unknown reported callable 'make_err' with 2 argument(s)")
                && first_message.contains("available arity(s): 1"),
            "Unknown-arity report routine call should include available arities, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_call_with_unknown_arity() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_unknown_arity_call.fol",
        )
        .expect("Should read procedure unknown-arity report routine-call fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject procedure report routine call with unknown arity in custom-error routine",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Unknown reported callable 'make_err' with 2 argument(s)")
                && first_message.contains("available arity(s): 1"),
            "Procedure unknown-arity report routine call should include available arities, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_unknown_called_method() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_unknown_method_call.fol",
        )
        .expect("Should read unknown report method-call fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject unknown called method in report expression for custom-error routine",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Unknown reported method 'parser.missing_err_source'"),
            "Unknown called method in report should produce explicit diagnostic, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_unknown_called_method() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_unknown_method_call.fol",
        )
        .expect("Should read unknown procedure report method-call fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject unknown called method in procedure report expression for custom-error routine",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Unknown reported method 'parser.missing_err_source'"),
            "Unknown called method in procedure report should produce explicit diagnostic, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_method_call_with_unknown_arity() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_unknown_arity_method_call.fol",
        )
        .expect("Should read unknown-arity report method-call fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject report method call with unknown arity in custom-error routine",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message
                .contains("Unknown reported callable 'parser.make_err' with 2 argument(s)")
                && first_message.contains("available arity(s): 1"),
            "Unknown-arity report method call should include available arities, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_method_call_with_unknown_arity() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_unknown_arity_method_call.fol",
        )
        .expect("Should read procedure unknown-arity report method-call fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject procedure report method call with unknown arity in custom-error routine",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message
                .contains("Unknown reported callable 'parser.make_err' with 2 argument(s)")
                && first_message.contains("available arity(s): 1"),
            "Procedure unknown-arity report method call should include available arities, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_report_method_call_result_from_receiver_decl() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_method_call_result_ok.fol",
        )
        .expect("Should read compatible report method-call result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept report method call result compatible via receiver declaration",
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_method_call_result_mismatch_from_receiver_decl(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_method_call_result_mismatch.fol",
        )
        .expect("Should read incompatible report method-call result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject report method call result incompatible via receiver declaration",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Method-call result mismatch via receiver declaration should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_report_method_call_result_from_receiver_decl() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_method_call_result_ok.fol",
        )
        .expect("Should read compatible procedure report method-call result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept procedure report method call result compatible via receiver declaration",
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_method_call_result_mismatch_from_receiver_decl(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_method_call_result_mismatch.fol",
        )
        .expect("Should read incompatible procedure report method-call result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject procedure report method call result incompatible via receiver declaration",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Procedure method-call result mismatch via receiver declaration should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_report_forward_method_call_result_from_receiver_decl(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_forward_method_call_result_ok.fol",
        )
        .expect("Should read compatible forward report method-call result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept report method call result compatible via receiver declaration when method is declared later",
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_forward_method_call_result_mismatch_from_receiver_decl(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_forward_method_call_result_mismatch.fol",
        )
        .expect("Should read incompatible forward report method-call result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject report method call result incompatible via receiver declaration when method is declared later",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Forward method-call result mismatch via receiver declaration should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_report_forward_method_call_result_from_receiver_decl(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_forward_method_call_result_ok.fol",
        )
        .expect("Should read compatible forward procedure report method-call result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept procedure report method call result compatible via receiver declaration when method is declared later",
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_forward_method_call_result_mismatch_from_receiver_decl(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_forward_method_call_result_mismatch.fol",
        )
        .expect("Should read incompatible forward procedure report method-call result file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject procedure report method call result incompatible via receiver declaration when method is declared later",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Forward procedure method-call result mismatch via receiver declaration should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_report_overloaded_call_result_by_arity() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_error_type_report_overload_arity_ok.fol")
                .expect("Should read overloaded report call-result compatible fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept overloaded report call result when selected arity return type is compatible");
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_overloaded_call_result_by_arity_mismatch() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_overload_arity_mismatch.fol",
        )
        .expect("Should read overloaded report call-result mismatch fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject overloaded report call result when selected arity return type is incompatible",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Overloaded call-result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_report_overloaded_call_result_by_arity() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_pro_error_type_report_overload_arity_ok.fol")
                .expect("Should read overloaded procedure report call-result compatible fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept overloaded procedure report call result when selected arity return type is compatible");
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_overloaded_call_result_by_arity_mismatch() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_overload_arity_mismatch.fol",
        )
        .expect("Should read overloaded procedure report call-result mismatch fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject overloaded procedure report call result when selected arity return type is incompatible",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Overloaded procedure call-result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_report_overloaded_method_call_result_by_arity() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_method_overload_arity_ok.fol",
        )
        .expect("Should read overloaded report method-call result compatible fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept overloaded report method call result when selected arity return type is compatible");
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_overloaded_method_call_result_by_arity_mismatch(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_method_overload_arity_mismatch.fol",
        )
        .expect("Should read overloaded report method-call result mismatch fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject overloaded report method call result when selected arity return type is incompatible",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Overloaded method-call result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_report_overloaded_method_call_result_by_arity() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_method_overload_arity_ok.fol",
        )
        .expect("Should read overloaded procedure report method-call result compatible fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser
            .parse(&mut lexer)
            .expect("Parser should accept overloaded procedure report method call result when selected arity return type is compatible");
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_overloaded_method_call_result_by_arity_mismatch(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_method_overload_arity_mismatch.fol",
        )
        .expect("Should read overloaded procedure report method-call result mismatch fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject overloaded procedure report method call result when selected arity return type is incompatible",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Overloaded procedure method-call result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_report_forward_overloaded_call_result_by_arity() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_forward_overload_arity_ok.fol",
        )
        .expect("Should read forward overloaded report call-result compatible fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept forward overloaded report call result when selected arity return type is compatible",
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_forward_overloaded_call_result_by_arity_mismatch(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_forward_overload_arity_mismatch.fol",
        )
        .expect("Should read forward overloaded report call-result mismatch fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject forward overloaded report call result when selected arity return type is incompatible",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Forward overloaded call-result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_report_forward_overloaded_call_result_by_arity() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_forward_overload_arity_ok.fol",
        )
        .expect("Should read forward overloaded procedure report call-result compatible fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept forward overloaded procedure report call result when selected arity return type is compatible",
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_forward_overloaded_call_result_by_arity_mismatch(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_forward_overload_arity_mismatch.fol",
        )
        .expect("Should read forward overloaded procedure report call-result mismatch fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject forward overloaded procedure report call result when selected arity return type is incompatible",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Forward overloaded procedure call-result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_custom_error_type_accepts_report_forward_overloaded_method_call_result_by_arity(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_forward_method_overload_arity_ok.fol",
        )
        .expect("Should read forward overloaded report method-call result compatible fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept forward overloaded report method call result when selected arity return type is compatible",
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_report_forward_overloaded_method_call_result_by_arity_mismatch(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_forward_method_overload_arity_mismatch.fol",
        )
        .expect("Should read forward overloaded report method-call result mismatch fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject forward overloaded report method call result when selected arity return type is incompatible",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Forward overloaded method-call result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_custom_error_type_accepts_report_forward_overloaded_method_call_result_by_arity(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_forward_method_overload_arity_ok.fol",
        )
        .expect(
            "Should read forward overloaded procedure report method-call result compatible fixture",
        );

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept forward overloaded procedure report method call result when selected arity return type is compatible",
        );
    }

    #[test]
    fn test_procedure_custom_error_type_rejects_report_forward_overloaded_method_call_result_by_arity_mismatch(
    ) {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_error_type_report_forward_method_overload_arity_mismatch.fol",
        )
        .expect(
            "Should read forward overloaded procedure report method-call result mismatch fixture",
        );

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject forward overloaded procedure report method call result when selected arity return type is incompatible",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Reported expression type")
                && first_message.contains("incompatible with routine error type"),
            "Forward overloaded procedure method-call result mismatch should report incompatible expression type, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_rejects_conflicting_duplicate_return_type_signature() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_conflicting_return_signature.fol")
                .expect("Should read conflicting duplicate function return signature fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject conflicting duplicate function return signatures");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Conflicting return type for routine 'parse'"),
            "Duplicate conflicting function signature should report explicit conflict, got: {}",
            first_message
        );
    }

    #[test]
    fn test_receiver_method_rejects_conflicting_duplicate_return_type_signature() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_receiver_conflicting_return_signature.fol",
        )
        .expect("Should read conflicting duplicate receiver method return signature fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject conflicting duplicate receiver method return signatures",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Conflicting return type for routine")
                && first_message.contains("parse"),
            "Duplicate conflicting receiver method signature should report explicit conflict, got: {}",
            first_message
        );
    }

    #[test]
    fn test_procedure_rejects_conflicting_duplicate_return_type_signature() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_pro_conflicting_return_signature.fol")
                .expect("Should read conflicting duplicate procedure return signature fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject conflicting duplicate procedure return signatures");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Conflicting return type for routine 'parse'"),
            "Duplicate conflicting procedure signature should report explicit conflict, got: {}",
            first_message
        );
    }

    #[test]
    fn test_receiver_procedure_method_rejects_conflicting_duplicate_return_type_signature() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_pro_receiver_conflicting_return_signature.fol",
        )
        .expect(
            "Should read conflicting duplicate receiver procedure method return signature fixture",
        );

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should reject conflicting duplicate receiver procedure method return signatures",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Conflicting return type for routine")
                && first_message.contains("parse"),
            "Duplicate conflicting receiver procedure method signature should report explicit conflict, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_accepts_duplicate_signature_with_same_return_type() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_duplicate_same_return_signature.fol")
                .expect("Should read duplicate same-return function signature fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept duplicate function signature when return type is the same",
        );
    }

    #[test]
    fn test_procedure_accepts_duplicate_signature_with_same_return_type() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_pro_duplicate_same_return_signature.fol")
                .expect("Should read duplicate same-return procedure signature fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept duplicate procedure signature when return type is the same",
        );
    }

    #[test]
    fn test_function_accepts_same_name_different_arity_with_different_return_types() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_overload_arity_distinct_returns.fol")
                .expect("Should read overloaded function-by-arity fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept same function name with different arity and return types",
        );
    }

    #[test]
    fn test_receiver_method_accepts_same_name_different_arity_with_different_return_types() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_receiver_overload_arity_distinct_returns.fol",
        )
        .expect("Should read overloaded receiver method-by-arity fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        parser.parse(&mut lexer).expect(
            "Parser should accept same receiver method name with different arity and return types",
        );
    }

    #[test]
    fn test_function_method_receiver_syntax_rejects_missing_receiver_close_paren() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_method_receiver_missing_close_paren.fol")
                .expect("Should read function missing receiver close paren fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject function receiver syntax missing ')' token");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Expected ')' after method receiver type"),
            "Missing receiver close paren should report explicit receiver syntax error, got: {}",
            first_message
        );
    }

    #[test]
    fn test_function_method_receiver_syntax_rejects_missing_receiver_type() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_method_receiver_missing_type.fol")
                .expect("Should read function missing receiver type fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject function receiver syntax missing receiver type");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains("Expected type reference"),
            "Missing receiver type should report type-reference parsing error, got: {}",
            first_message
        );
    }

