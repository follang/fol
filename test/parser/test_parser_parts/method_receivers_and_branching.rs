use super::*;

#[test]
fn test_procedure_method_receiver_syntax_rejects_missing_method_name() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_method_receiver_missing_name.fol")
            .expect("Should read procedure missing receiver method-name fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject procedure receiver syntax missing method name");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected procedure name after 'pro'"),
        "Missing procedure method name should report expected-name parse error, got: {}",
        first_message
    );
}

#[test]
fn test_function_method_receiver_syntax_rejects_missing_method_name() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_method_receiver_missing_name.fol")
            .expect("Should read function missing receiver method-name fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject function receiver syntax missing method name");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected function name after 'fun'"),
        "Missing function method name should report expected-name parse error, got: {}",
        first_message
    );
}

#[test]
fn test_procedure_method_receiver_syntax_rejects_missing_receiver_close_paren() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_method_receiver_missing_close_paren.fol")
            .expect("Should read procedure missing receiver close paren fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject procedure receiver syntax missing ')' token");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
            first_message.contains("Expected ')' after method receiver type"),
            "Missing procedure receiver close paren should report explicit receiver syntax error, got: {}",
            first_message
        );
}

#[test]
fn test_procedure_method_receiver_syntax_rejects_missing_receiver_type() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_method_receiver_missing_type.fol")
            .expect("Should read procedure missing receiver type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject procedure receiver syntax missing receiver type");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected type reference"),
        "Missing procedure receiver type should report type-reference parsing error, got: {}",
        first_message
    );
}

#[test]
fn test_function_method_receiver_supports_qualified_type_references() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_method_receiver_qualified.fol")
            .expect("Should read qualified function receiver fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept qualified method receiver type");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                program_surface_nodes(&declarations).into_iter().any(|node| {
                    matches!(
                        node,
                        AstNode::FunDecl { name, .. } if name == "parse_msg"
                    )
                }),
                "Qualified receiver function should parse as a function declaration"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_procedure_method_receiver_supports_bracketed_type_references() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_method_receiver_bracketed.fol")
            .expect("Should read bracketed procedure receiver fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept bracketed method receiver type");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                program_surface_nodes(&declarations).into_iter().any(|node| {
                    matches!(
                        node,
                        AstNode::ProDecl { name, .. } if name == "store"
                    )
                }),
                "Bracketed receiver procedure should parse as a procedure declaration"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_function_method_receiver_missing_bracket_close_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_method_receiver_bracket_missing_close.fol")
            .expect("Should read malformed bracketed function receiver fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject receiver type missing closing ']'");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected closing ']' in type reference"),
        "Malformed bracketed receiver type should report missing close bracket, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        1,
        "Malformed receiver type parse error should point to the signature line"
    );
}

#[test]
fn test_function_method_receiver_syntax_accepts_builtin_receiver_type() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_method_receiver_builtin_type.fol")
            .expect("Should read function builtin receiver type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept function receiver syntax with builtin receiver type");

    match ast {
        AstNode::Program { declarations } => assert!(
            program_surface_nodes(&declarations).into_iter().any(|node| matches!(
                node, AstNode::FunDecl { name, .. } if name == "parse_msg"
            )),
            "Builtin scalar receiver method should parse as a named function declaration"
        ),
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_procedure_method_receiver_syntax_accepts_builtin_receiver_type() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_method_receiver_builtin_type.fol")
            .expect("Should read procedure builtin receiver type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept procedure receiver syntax with builtin receiver type");

    match ast {
        AstNode::Program { declarations } => assert!(
            program_surface_nodes(&declarations).into_iter().any(|node| matches!(
                node, AstNode::ProDecl { name, .. } if name == "parse_msg"
            )),
            "Builtin scalar receiver method should parse as a named procedure declaration"
        ),
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_function_method_receiver_syntax_rejects_builtin_keyword_receiver_type() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_method_receiver_builtin_keyword.fol")
            .expect("Should read function builtin-keyword receiver type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject function receiver syntax with builtin keyword receiver type",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected type reference"),
        "Builtin keyword receiver type should report a type-reference diagnostic, got: {}",
        first_message
    );
}

#[test]
fn test_procedure_method_receiver_syntax_rejects_builtin_keyword_receiver_type() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_method_receiver_builtin_keyword.fol")
            .expect("Should read procedure builtin-keyword receiver type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject procedure receiver syntax with builtin keyword receiver type",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
            first_message.contains("Expected type reference"),
            "Procedure builtin keyword receiver type should report a type-reference diagnostic, got: {}",
            first_message
        );
}

#[test]
fn test_function_custom_error_type_accepts_compatible_report_local_var() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_error_type_report_local_var_ok.fol")
            .expect("Should read compatible custom-error report local var file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser
        .parse(&mut lexer)
        .expect("Parser should accept report local var compatible with custom error type");
}

#[test]
fn test_function_custom_error_type_rejects_incompatible_report_local_var() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_error_type_report_local_var_mismatch.fol")
            .expect("Should read incompatible custom-error report local var file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should fail when report local var is incompatible with custom error type",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Reported identifier")
            && first_message.contains("incompatible with routine error type"),
        "Custom-error routine should reject incompatible report local var type, got: {}",
        first_message
    );
}

#[test]
fn test_function_custom_error_type_accepts_compatible_report_inferred_local_var() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_error_type_report_inferred_local_ok.fol")
            .expect("Should read compatible inferred local var report file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser
        .parse(&mut lexer)
        .expect("Parser should accept report inferred local var compatible with custom error type");
}

#[test]
fn test_function_custom_error_type_rejects_incompatible_report_inferred_local_var() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_inferred_local_mismatch.fol",
    )
    .expect("Should read incompatible inferred local var report file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should fail when report inferred local var is incompatible with custom error type",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Reported identifier")
            && first_message.contains("incompatible with routine error type"),
        "Custom-error routine should reject incompatible inferred local var type, got: {}",
        first_message
    );
}

#[test]
fn test_function_custom_error_type_accepts_nested_inferred_local_report() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_nested_inferred_local_ok.fol",
    )
    .expect("Should read nested inferred-local compatible report file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept nested inferred-local report compatible with custom error type",
    );
}

#[test]
fn test_function_custom_error_type_rejects_nested_inferred_local_report_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_nested_inferred_local_mismatch.fol",
    )
    .expect("Should read nested inferred-local mismatch report file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject nested inferred-local report incompatible with custom error type",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Reported identifier")
            && first_message.contains("incompatible with routine error type"),
        "Nested inferred-local mismatch should report incompatible identifier type, got: {}",
        first_message
    );
}

#[test]
fn test_function_custom_error_type_accepts_loop_inferred_local_report() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_loop_inferred_local_ok.fol",
    )
    .expect("Should read loop inferred-local compatible report file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept loop inferred-local report compatible with custom error type",
    );
}

#[test]
fn test_function_custom_error_type_rejects_loop_inferred_local_report_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_loop_inferred_local_mismatch.fol",
    )
    .expect("Should read loop inferred-local mismatch report file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject loop inferred-local report incompatible with custom error type",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Reported identifier")
            && first_message.contains("incompatible with routine error type"),
        "Loop inferred-local mismatch should report incompatible identifier type, got: {}",
        first_message
    );
}

#[test]
fn test_function_custom_error_type_accepts_nested_shadowed_report_identifier() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_error_type_report_nested_shadow_ok.fol")
            .expect("Should read nested shadow compatible report file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser.parse(&mut lexer).expect(
        "Parser should accept nested shadowed identifier compatible with custom error type",
    );
}

#[test]
fn test_function_custom_error_type_rejects_nested_shadowed_report_identifier_mismatch() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_fun_error_type_report_nested_shadow_mismatch.fol",
    )
    .expect("Should read nested shadow mismatch report file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser.parse(&mut lexer).expect_err(
        "Parser should reject nested shadowed identifier incompatible with custom error type",
    );

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Reported identifier")
            && first_message.contains("incompatible with routine error type"),
        "Nested shadow mismatch should report incompatible identifier type, got: {}",
        first_message
    );
}

#[test]
fn test_when_statement_parsing_with_case_and_default() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_when.fol")
        .expect("Should read when test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse when statement");

    let when_stmt = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| {
                if let AstNode::When {
                    expr,
                    cases,
                    default,
                } = node
                {
                    Some((expr.as_ref().clone(), cases.clone(), default.clone()))
                } else {
                    None
                }
            })
            .expect("Program should include a when statement"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(when_stmt.0, AstNode::Identifier { name } if name == "a"),
        "When expression should parse identifier a"
    );
    assert_eq!(when_stmt.1.len(), 1, "When should include one case");
    assert!(when_stmt.2.is_some(), "When should include default body");
}

#[test]
fn test_when_statement_parsing_with_multiple_cases() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_when_multi_case.fol")
        .expect("Should read multi-case when test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse when statement with multiple cases");

    let when_stmt = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| {
                if let AstNode::When {
                    expr,
                    cases,
                    default,
                } = node
                {
                    Some((expr.as_ref().clone(), cases.clone(), default.clone()))
                } else {
                    None
                }
            })
            .expect("Program should include a when statement"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(when_stmt.0, AstNode::Identifier { name } if name == "a"),
        "When expression should parse identifier a"
    );
    assert_eq!(
        when_stmt.1.len(),
        2,
        "When should include two case branches"
    );
    assert!(when_stmt.2.is_some(), "When should include default body");
}

#[test]
fn test_when_case_body_supports_nested_if_and_loop() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_when_nested_control.fol")
        .expect("Should read nested-control when test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse nested control flow inside when case body");

    let when_cases = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| {
                if let AstNode::When { cases, .. } = node {
                    Some(cases.clone())
                } else {
                    None
                }
            })
            .expect("Program should include a when statement"),
        _ => panic!("Expected program node"),
    };

    let first_case_body = when_cases
        .iter()
        .find_map(|case| {
            if let fol_parser::ast::WhenCase::Case { body, .. } = case {
                Some(body.clone())
            } else {
                None
            }
        })
        .expect("When should include at least one case body");

    assert!(
        first_case_body
            .iter()
            .any(|node| matches!(node, AstNode::When { .. })),
        "Case body should include lowered if statement"
    );

    let lowered_if = first_case_body
        .iter()
        .find_map(|node| {
            if let AstNode::When { default, .. } = node {
                Some(default.clone())
            } else {
                None
            }
        })
        .expect("Case body should include lowered if node");

    let default_body = lowered_if.expect("Lowered if should include else/default body");
    assert!(
        default_body
            .iter()
            .any(|node| matches!(node, AstNode::Loop { .. })),
        "Case body should include loop statement from else branch"
    );
}

#[test]
fn test_if_statement_lowers_to_when_shape() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_if.fol").expect("Should read if test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse if statement");

    let lowered_if = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| {
                if let AstNode::When {
                    expr,
                    cases,
                    default,
                } = node
                {
                    Some((expr.as_ref().clone(), cases.clone(), default.clone()))
                } else {
                    None
                }
            })
            .expect("Program should include lowered if/when node"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(
            lowered_if.0,
            AstNode::BinaryOp {
                op: fol_parser::ast::BinaryOperator::Eq,
                ..
            }
        ),
        "If condition should parse equality expression"
    );
    assert_eq!(lowered_if.1.len(), 1, "Lowered if should include one case");
    assert!(
        lowered_if.2.is_some(),
        "Lowered if should include default branch body"
    );
}

#[test]
fn test_if_chain_lowers_to_nested_when_default() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_if_chain.fol")
        .expect("Should read if-chain test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse chained if statements");

    let lowered_if = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| {
                if let AstNode::When {
                    expr,
                    cases,
                    default,
                } = node
                {
                    Some((expr.as_ref().clone(), cases.clone(), default.clone()))
                } else {
                    None
                }
            })
            .expect("Program should include lowered if/when node"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(
            lowered_if.0,
            AstNode::BinaryOp {
                op: fol_parser::ast::BinaryOperator::Eq,
                ..
            }
        ),
        "Outer if condition should parse equality expression"
    );
    let default = lowered_if
        .2
        .expect("Outer if should include default chain/default block");
    assert!(
        default
            .iter()
            .any(|node| matches!(node, AstNode::When { .. })),
        "Outer if default should contain nested lowered if"
    );
}

#[test]
fn test_if_statement_without_else_has_no_default_branch() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_if_no_else.fol")
        .expect("Should read if-no-else test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse if statement without else");

    let lowered_if = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| {
                if let AstNode::When {
                    expr,
                    cases,
                    default,
                } = node
                {
                    Some((expr.as_ref().clone(), cases.clone(), default.clone()))
                } else {
                    None
                }
            })
            .expect("Program should include lowered if/when node"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(
            lowered_if.0,
            AstNode::BinaryOp {
                op: fol_parser::ast::BinaryOperator::Lt,
                ..
            }
        ),
        "If condition should parse less-than expression"
    );
    assert_eq!(lowered_if.1.len(), 1, "If should include one case");
    assert!(
        lowered_if.2.is_none(),
        "If without else should not include default branch"
    );
}

#[test]
fn test_else_if_keyword_chain_lowers_to_nested_when_default() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_else_if.fol")
        .expect("Should read else-if test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse else-if keyword chain");

    let lowered_if = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| {
                if let AstNode::When {
                    expr,
                    cases,
                    default,
                } = node
                {
                    Some((expr.as_ref().clone(), cases.clone(), default.clone()))
                } else {
                    None
                }
            })
            .expect("Program should include lowered if/when node"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(
            lowered_if.0,
            AstNode::BinaryOp {
                op: fol_parser::ast::BinaryOperator::Eq,
                ..
            }
        ),
        "Outer if condition should parse equality expression"
    );
    let default = lowered_if
        .2
        .expect("Else-if chain should include default branch body");
    assert!(
        default
            .iter()
            .any(|node| matches!(node, AstNode::When { .. })),
        "Else-if should lower to nested when in default branch"
    );
}

#[test]
fn test_else_keyword_block_maps_to_direct_default_body() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_else_only.fol")
        .expect("Should read else-only test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse if-else keyword block");

    let lowered_if = match ast {
        AstNode::Program { declarations } => declarations
            .iter()
            .find_map(|node| {
                if let AstNode::When {
                    expr,
                    cases,
                    default,
                } = node
                {
                    Some((expr.as_ref().clone(), cases.clone(), default.clone()))
                } else {
                    None
                }
            })
            .expect("Program should include lowered if/when node"),
        _ => panic!("Expected program node"),
    };

    assert!(
        matches!(
            lowered_if.0,
            AstNode::BinaryOp {
                op: fol_parser::ast::BinaryOperator::Lt,
                ..
            }
        ),
        "If condition should parse less-than expression"
    );
    let default = lowered_if
        .2
        .expect("Else block should produce default body");
    assert!(
        default
            .iter()
            .all(|node| !matches!(node, AstNode::When { .. })),
        "Else-only block should not introduce nested when nodes"
    );
    assert!(
        default
            .iter()
            .any(|node| matches!(node, AstNode::Return { .. })),
        "Else-only default body should include return statement"
    );
}
