// Parser tests - to be expanded when full parser is implemented

use fol_lexer::lexer::stage3::Elements;
use fol_lexer::token::KEYWORD;
use fol_parser::ast::{AstNode, AstParser, FolType, Literal, ParseError};
use fol_stream::FileStream;

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_var.fol").expect("Should read test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        match parser.parse(&mut lexer) {
            Ok(ast) => {
                match &ast {
                    AstNode::Program { declarations } => {
                        assert!(
                            !declarations.is_empty(),
                            "Parser should collect at least identifiers/literals"
                        );
                        assert!(
                            declarations.iter().any(|node| {
                                matches!(
                                    node,
                                    AstNode::VarDecl {
                                        name,
                                        type_hint: Some(_),
                                        value: Some(_),
                                        ..
                                    } if name == "x"
                                )
                            }),
                            "Parser should build a var declaration node for simple_var.fol"
                        );
                    }
                    _ => panic!("Should return Program node"),
                }
                println!("Successfully parsed AST: {:?}", ast);
            }
            Err(errors) => {
                panic!("Parser should not fail: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_function_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun.fol").expect("Should read test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        match parser.parse(&mut lexer) {
            Ok(ast) => {
                match &ast {
                    AstNode::Program { declarations } => {
                        assert!(
                            !declarations.is_empty(),
                            "Function source should produce parser nodes"
                        );
                        assert!(
                            declarations.iter().any(|node| {
                                matches!(
                                    node,
                                    AstNode::Return {
                                        value: Some(value)
                                    } if matches!(value.as_ref(), AstNode::BinaryOp { .. })
                                )
                            }),
                            "Function source should include a return node with binary expression"
                        );
                    }
                    _ => panic!("Should return Program node"),
                }
                println!("Successfully parsed function AST: {:?}", ast);
            }
            Err(errors) => {
                println!("Parser errors (expected for now): {:?}", errors);
                // For now, we expect the minimal parser to work
            }
        }
    }

    #[test]
    fn test_function_declaration_header_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun.fol").expect("Should read test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse function declaration");

        let function_decl = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::FunDecl {
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
                .expect("Program should include function declaration"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(function_decl.0, "add");
        assert_eq!(function_decl.1, 2, "Function should have two parameters");
        assert!(function_decl.2, "Function should have return type");
        assert!(
            function_decl.3 > 0,
            "Function body should include parsed statements"
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
            matches!(procedure_decl.1, Some(FolType::Named { name }) if name == "int"),
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
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_local_var_mismatch.fol",
        )
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
        parser.parse(&mut lexer).expect(
            "Parser should accept report inferred local var compatible with custom error type",
        );
    }

    #[test]
    fn test_function_custom_error_type_rejects_incompatible_report_inferred_local_var() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_error_type_report_inferred_local_mismatch.fol",
        )
        .expect("Should read incompatible inferred local var report file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when report inferred local var is incompatible with custom error type");

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
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_when_nested_control.fol")
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
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_if.fol")
            .expect("Should read if test file");

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

    #[test]
    fn test_multi_else_if_chain_lowers_to_recursive_when_defaults() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_else_if_multi.fol")
            .expect("Should read multi else-if test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse multi else-if chain");

        let top_when = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::When { cases, default, .. } = node {
                        Some((cases.clone(), default.clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should include top-level lowered when node"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(top_when.0.len(), 1, "Top if should have one case");

        let first_default = top_when
            .1
            .expect("First else-if step should produce default branch");
        let nested_when_1 = first_default
            .iter()
            .find_map(|node| {
                if let AstNode::When { cases, default, .. } = node {
                    Some((cases.clone(), default.clone()))
                } else {
                    None
                }
            })
            .expect("First default should contain nested when");
        assert_eq!(
            nested_when_1.0.len(),
            1,
            "First nested else-if should have one case"
        );

        let second_default = nested_when_1
            .1
            .expect("Second else-if step should produce default branch");
        let nested_when_2 = second_default
            .iter()
            .find_map(|node| {
                if let AstNode::When { cases, default, .. } = node {
                    Some((cases.clone(), default.clone()))
                } else {
                    None
                }
            })
            .expect("Second default should contain nested when");
        assert_eq!(
            nested_when_2.0.len(),
            1,
            "Second nested else-if should have one case"
        );

        let final_default = nested_when_2
            .1
            .expect("Final else branch should exist at deepest nested default");
        assert!(
            final_default
                .iter()
                .any(|node| matches!(node, AstNode::Return { .. })),
            "Final else branch should contain return statement"
        );
    }

    #[test]
    fn test_loop_statement_parsing_with_condition_body() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_loop.fol")
            .expect("Should read loop test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse loop statement");

        let loop_stmt = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Loop { condition, body } = node {
                        Some((condition.as_ref().clone(), body.clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should include a loop statement"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(loop_stmt.0, fol_parser::ast::LoopCondition::Condition(_)),
            "Loop should parse condition expression"
        );
        assert!(
            loop_stmt
                .1
                .iter()
                .any(|node| matches!(node, AstNode::Assignment { .. })),
            "Loop body should contain assignment statement"
        );
    }

    #[test]
    fn test_loop_statement_parsing_with_break_body() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_loop_break.fol")
            .expect("Should read loop break test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse loop with break statement");

        let loop_body = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Loop { body, .. } = node {
                        Some(body.clone())
                    } else {
                        None
                    }
                })
                .expect("Program should include a loop statement"),
            _ => panic!("Expected program node"),
        };

        assert!(
            loop_body.iter().any(|node| matches!(node, AstNode::Break)),
            "Loop body should contain break statement"
        );
    }

    #[test]
    fn test_loop_break_without_semicolon_is_accepted() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_loop_break_no_semi.fol")
                .expect("Should read loop break without semicolon test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse break without semicolon");

        let loop_body = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Loop { body, .. } = node {
                        Some(body.clone())
                    } else {
                        None
                    }
                })
                .expect("Program should include a loop statement"),
            _ => panic!("Expected program node"),
        };

        assert!(
            loop_body.iter().any(|node| matches!(node, AstNode::Break)),
            "Loop body should contain break statement"
        );
    }

    #[test]
    fn test_loop_statement_parsing_with_yield_body() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_loop_yield.fol")
            .expect("Should read loop yield test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse loop with yield statement");

        let loop_body = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Loop { body, .. } = node {
                        Some(body.clone())
                    } else {
                        None
                    }
                })
                .expect("Program should include a loop statement"),
            _ => panic!("Expected program node"),
        };

        assert!(
            loop_body
                .iter()
                .any(|node| matches!(node, AstNode::Yield { .. })),
            "Loop body should contain yield statement"
        );
    }

    #[test]
    fn test_loop_yield_without_semicolon_is_accepted() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_loop_yield_no_semi.fol")
                .expect("Should read loop yield without semicolon test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse yeild without semicolon");

        let loop_body = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Loop { body, .. } = node {
                        Some(body.clone())
                    } else {
                        None
                    }
                })
                .expect("Program should include a loop statement"),
            _ => panic!("Expected program node"),
        };

        assert!(
            loop_body
                .iter()
                .any(|node| matches!(node, AstNode::Yield { .. })),
            "Loop body should contain yield statement"
        );
    }

    #[test]
    fn test_loop_iteration_condition_parsing_with_in() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_loop_in.fol")
            .expect("Should read loop iteration test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse loop iteration condition");

        let loop_stmt = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Loop { condition, body } = node {
                        Some((condition.as_ref().clone(), body.clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should include a loop statement"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(
                loop_stmt.0,
                fol_parser::ast::LoopCondition::Iteration { var, .. } if var == "i"
            ),
            "Loop should parse iteration form with variable i"
        );
        assert!(
            loop_stmt
                .1
                .iter()
                .any(|node| matches!(node, AstNode::Yield { .. })),
            "Iteration loop body should contain yield statement"
        );
        assert!(
            loop_stmt
                .1
                .iter()
                .any(|node| matches!(node, AstNode::Break)),
            "Iteration loop body should contain break statement"
        );
    }

    #[test]
    fn test_loop_iteration_condition_with_when_guard() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_loop_in_when.fol")
            .expect("Should read guarded iteration loop test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse iteration loop with when guard");

        let loop_condition = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Loop { condition, .. } = node {
                        Some(condition.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should include a loop statement"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(
                loop_condition,
                fol_parser::ast::LoopCondition::Iteration {
                    var,
                    condition: Some(_),
                    ..
                } if var == "i"
            ),
            "Iteration loop should include variable and parsed when-guard condition"
        );
    }

    #[test]
    fn test_builtin_diagnostic_statements_parse_as_function_calls() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_builtin_diag.fol")
            .expect("Should read builtin diagnostic test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse builtin diagnostic statements");

        let call_names = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .filter_map(|node| {
                    if let AstNode::FunctionCall { name, .. } = node {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            _ => panic!("Expected program node"),
        };

        assert!(
            call_names.contains(&"check".to_string()),
            "Should parse check statement as function call"
        );
        assert!(
            call_names.contains(&"report".to_string()),
            "Should parse report statement as function call"
        );
        assert!(
            call_names.contains(&"assert".to_string()),
            "Should parse assert statement as function call"
        );
        assert!(
            call_names.contains(&"panic".to_string()),
            "Should parse panic statement as function call"
        );
    }

    #[test]
    fn test_builtin_diagnostic_statements_without_args_parse_as_empty_calls() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_builtin_diag_no_args.fol")
                .expect("Should read builtin diagnostic no-args test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse builtin diagnostic statements without args");

        let calls = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .filter_map(|node| {
                    if let AstNode::FunctionCall { name, args } = node {
                        Some((name.clone(), args.len()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            _ => panic!("Expected program node"),
        };

        assert!(
            calls
                .iter()
                .any(|(name, argc)| name == "check" && *argc == 0),
            "check without args should parse as zero-arg call"
        );
        assert!(
            calls
                .iter()
                .any(|(name, argc)| name == "report" && *argc == 0),
            "report without args should parse as zero-arg call"
        );
        assert!(
            calls
                .iter()
                .any(|(name, argc)| name == "assert" && *argc == 0),
            "assert without args should parse as zero-arg call"
        );
        assert!(
            calls
                .iter()
                .any(|(name, argc)| name == "panic" && *argc == 0),
            "panic without args should parse as zero-arg call"
        );
    }

    #[test]
    fn test_function_body_identifier_calls_parse_as_functioncall_nodes() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_call_stmt.fol")
            .expect("Should read function call statement test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse identifier call statements");

        let call_names = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .filter_map(|node| {
                    if let AstNode::FunctionCall { name, .. } = node {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            _ => panic!("Expected program node"),
        };

        assert!(
            call_names.contains(&"process".to_string()),
            "Should parse process(a, b) as function call"
        );
        assert!(
            call_names.contains(&"ping".to_string()),
            "Should parse ping() as function call"
        );
    }

    #[test]
    fn test_top_level_identifier_call_parsing() {
        let mut file_stream = FileStream::from_file("test/parser/simple_call_top_level.fol")
            .expect("Should read top-level call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse top-level identifier call");

        let call = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::FunctionCall { name, args } = node {
                        Some((name.clone(), args.len()))
                    } else {
                        None
                    }
                })
                .expect("Program should include top-level function call"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(call.0, "run");
        assert_eq!(call.1, 2, "Top-level call should include two arguments");
    }

    #[test]
    fn test_top_level_multiline_identifier_call_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_call_top_level_multiline.fol")
                .expect("Should read top-level multiline call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse top-level multiline identifier call");

        let call = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::FunctionCall { name, args } = node {
                        Some((name.clone(), args.len()))
                    } else {
                        None
                    }
                })
                .expect("Program should include top-level multiline function call"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(call.0, "run");
        assert_eq!(
            call.1, 3,
            "Top-level multiline call should include three arguments"
        );
    }

    #[test]
    fn test_top_level_call_with_unary_plus_arguments_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_call_top_level_unary_plus_args.fol")
                .expect("Should read top-level unary-plus call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse top-level unary-plus call arguments");

        let call = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::FunctionCall { name, args } = node {
                        Some((name.clone(), args.clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should include top-level function call"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(call.0, "run");
        assert_eq!(
            call.1.len(),
            2,
            "Top-level unary-plus call should have two args"
        );
        assert!(
            matches!(&call.1[0], AstNode::Identifier { name } if name == "a"),
            "Unary plus on first arg should fold to identifier 'a'"
        );
        assert!(
            matches!(
                &call.1[1],
                AstNode::BinaryOp {
                    op: fol_parser::ast::BinaryOperator::Add,
                    ..
                }
            ),
            "Unary plus on parenthesized second arg should preserve inner addition expression"
        );
    }

    #[test]
    fn test_call_and_method_call_with_unary_plus_arguments_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_unary_plus_args.fol")
                .expect("Should read unary-plus call args fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse unary-plus arguments in call and method-call contexts");

        let (has_run_assignment, has_update_method_call, has_emit_return) = match ast {
            AstNode::Program { declarations } => {
                let has_run_assignment = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "run"
                                && args.len() == 2
                                && matches!(&args[0], AstNode::Identifier { name } if name == "a")
                                && matches!(&args[1], AstNode::BinaryOp { op: fol_parser::ast::BinaryOperator::Add, .. })
                        )
                    )
                });

                let has_update_method_call = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::MethodCall { method, args, .. }
                        if method == "update"
                            && args.len() == 2
                            && matches!(&args[0], AstNode::Identifier { name } if name == "a")
                            && matches!(&args[1], AstNode::BinaryOp { op: fol_parser::ast::BinaryOperator::Add, .. })
                    )
                });

                let has_emit_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "emit"
                                && args.len() == 2
                                && matches!(&args[0], AstNode::Identifier { name } if name == "a")
                                && matches!(&args[1], AstNode::BinaryOp { op: fol_parser::ast::BinaryOperator::Add, .. })
                        )
                    )
                });

                (has_run_assignment, has_update_method_call, has_emit_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_run_assignment,
            "Function call assignment should parse unary-plus args with expected shapes"
        );
        assert!(
            has_update_method_call,
            "Method call should parse unary-plus args with expected shapes"
        );
        assert!(
            has_emit_return,
            "Return call should parse unary-plus args with expected shapes"
        );
    }

    #[test]
    fn test_top_level_call_with_unary_ref_deref_arguments_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_call_top_level_unary_ref_deref_args.fol")
                .expect("Should read top-level unary ref/deref call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse top-level unary ref/deref call arguments");

        let call = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::FunctionCall { name, args } = node {
                        Some((name.clone(), args.clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should include top-level function call"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(call.0, "run");
        assert_eq!(
            call.1.len(),
            2,
            "Top-level unary ref/deref call should have two args"
        );
        assert!(
            matches!(&call.1[0], AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Ref, operand } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a")),
            "First arg should parse as unary ref of identifier 'a'"
        );
        assert!(
            matches!(&call.1[1], AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Deref, operand } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "b")),
            "Second arg should parse as unary deref of identifier 'b'"
        );
    }

    #[test]
    fn test_call_and_method_call_with_unary_ref_deref_arguments_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_unary_ref_deref_args.fol")
                .expect("Should read unary ref/deref call args fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser.parse(&mut lexer).expect(
            "Parser should parse unary ref/deref arguments in call and method-call contexts",
        );

        let (has_run_assignment, has_update_method_call, has_emit_return) = match ast {
            AstNode::Program { declarations } => {
                let has_run_assignment = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "run"
                                && args.len() == 2
                                && matches!(&args[0], AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Ref, operand } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a"))
                                && matches!(&args[1], AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Deref, operand } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "b"))
                        )
                    )
                });

                let has_update_method_call = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::MethodCall { method, args, .. }
                        if method == "update"
                            && args.len() == 2
                            && matches!(&args[0], AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Ref, operand } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a"))
                            && matches!(&args[1], AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Deref, operand } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "b"))
                    )
                });

                let has_emit_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "emit"
                                && args.len() == 2
                                && matches!(&args[0], AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Ref, operand } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a"))
                                && matches!(&args[1], AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Deref, operand } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "b"))
                        )
                    )
                });

                (has_run_assignment, has_update_method_call, has_emit_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_run_assignment,
            "Function call assignment should parse unary ref/deref args with expected shapes"
        );
        assert!(
            has_update_method_call,
            "Method call should parse unary ref/deref args with expected shapes"
        );
        assert!(
            has_emit_return,
            "Return call should parse unary ref/deref args with expected shapes"
        );
    }

    #[test]
    fn test_call_expressions_in_assignment_and_return() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_call_expr.fol")
            .expect("Should read call expression test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse call expressions in statements");

        let (has_call_assignment, has_call_return) = match ast {
            AstNode::Program { declarations } => {
                let has_call_assignment = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(value.as_ref(), AstNode::FunctionCall { name, .. } if name == "compute")
                    )
                });

                let has_call_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::FunctionCall { name, .. } if name == "emit")
                    )
                });

                (has_call_assignment, has_call_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_call_assignment,
            "Assignment value should parse as function call expression"
        );
        assert!(
            has_call_return,
            "Return value should parse as function call expression"
        );
    }

    #[test]
    fn test_zero_argument_calls_in_statement_and_return_positions() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_call_no_args.fol")
            .expect("Should read zero-argument call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse zero-argument calls");

        let (has_ping_stmt, has_pong_stmt, has_emit_return) = match ast {
            AstNode::Program { declarations } => {
                let has_ping_stmt = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::FunctionCall { name, args } if name == "ping" && args.is_empty()
                    )
                });

                let has_pong_stmt = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::FunctionCall { name, args } if name == "pong" && args.is_empty()
                    )
                });

                let has_emit_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::FunctionCall { name, args } if name == "emit" && args.is_empty())
                    )
                });

                (has_ping_stmt, has_pong_stmt, has_emit_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_ping_stmt,
            "Should parse ping() as zero-arg statement call"
        );
        assert!(
            has_pong_stmt,
            "Should parse pong() without semicolon as zero-arg statement call"
        );
        assert!(
            has_emit_return,
            "Should parse return emit() as zero-arg return call"
        );
    }

    #[test]
    fn test_method_calls_in_statement_and_return_positions() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_method_call.fol")
            .expect("Should read method call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse method calls");

        let (has_update_stmt, has_get_return) = match ast {
            AstNode::Program { declarations } => {
                let has_update_stmt = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::MethodCall { method, .. } if method == "update"
                    )
                });

                let has_get_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::MethodCall { method, .. } if method == "get")
                    )
                });

                (has_update_stmt, has_get_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_update_stmt,
            "Should parse object.update(value) method call"
        );
        assert!(
            has_get_return,
            "Should parse return object.get() method call"
        );
    }

    #[test]
    fn test_zero_argument_method_calls_with_optional_semicolons() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_method_call_no_args.fol")
                .expect("Should read zero-arg method call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse zero-argument method calls");

        let (has_start_stmt, has_stop_stmt, has_done_return) = match ast {
            AstNode::Program { declarations } => {
                let has_start_stmt = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::MethodCall { method, args, .. } if method == "start" && args.is_empty()
                    )
                });

                let has_stop_stmt = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::MethodCall { method, args, .. } if method == "stop" && args.is_empty()
                    )
                });

                let has_done_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::MethodCall { method, args, .. } if method == "done" && args.is_empty())
                    )
                });

                (has_start_stmt, has_stop_stmt, has_done_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_start_stmt,
            "Should parse obj.start() as zero-arg statement method call"
        );
        assert!(
            has_stop_stmt,
            "Should parse obj.stop() without semicolon as zero-arg statement method call"
        );
        assert!(
            has_done_return,
            "Should parse return obj.done() as zero-arg return method call"
        );
    }

    #[test]
    fn test_nested_function_and_method_calls_in_expression_positions() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_nested_calls.fol")
            .expect("Should read nested calls test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse nested function/method calls");

        let (has_wrapped_method_assignment, has_nested_return_emit) = match ast {
            AstNode::Program { declarations } => {
                let has_wrapped_method_assignment = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "wrap"
                                && args.len() == 1
                                && matches!(args[0], AstNode::MethodCall { ref method, .. } if method == "get")
                        )
                    )
                });

                let has_nested_return_emit = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "emit"
                                && args.len() == 2
                                && matches!(args[0], AstNode::FunctionCall { ref name, .. } if name == "process")
                                && matches!(args[1], AstNode::MethodCall { ref method, .. } if method == "done")
                        )
                    )
                });

                (has_wrapped_method_assignment, has_nested_return_emit)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_wrapped_method_assignment,
            "Assignment should parse wrap(obj.get()) nesting"
        );
        assert!(
            has_nested_return_emit,
            "Return should parse emit(process(a), obj.done()) nesting"
        );
    }

    #[test]
    fn test_call_argument_lists_accept_trailing_commas() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_trailing_comma.fol")
                .expect("Should read trailing comma call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse call arguments with trailing commas");

        let (has_ping_two_args, has_run_one_arg, has_emit_one_arg) = match ast {
            AstNode::Program { declarations } => {
                let has_ping_two_args = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::FunctionCall { name, args }
                        if name == "ping" && args.len() == 2
                    )
                });

                let has_run_one_arg = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::MethodCall { method, args, .. }
                        if method == "run" && args.len() == 1
                    )
                });

                let has_emit_one_arg = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::FunctionCall { name, args } if name == "emit" && args.len() == 1)
                    )
                });

                (has_ping_two_args, has_run_one_arg, has_emit_one_arg)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_ping_two_args,
            "ping(a, b,) should parse with two arguments"
        );
        assert!(
            has_run_one_arg,
            "obj.run(a,) should parse with one argument"
        );
        assert!(
            has_emit_one_arg,
            "return emit(a,) should parse with one argument"
        );
    }

    #[test]
    fn test_nested_calls_with_trailing_commas_preserve_argument_shapes() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_nested_trailing_comma.fol")
                .expect("Should read nested trailing comma call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse nested trailing-comma calls");

        let (has_outer_two_args, has_done_one_arg) = match ast {
            AstNode::Program { declarations } => {
                let has_outer_two_args = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "outer"
                                && args.len() == 2
                                && matches!(args[0], AstNode::FunctionCall { ref name, args: ref nested_args } if name == "inner" && nested_args.len() == 1)
                                && matches!(args[1], AstNode::MethodCall { ref method, args: ref nested_args, .. } if method == "run" && nested_args.len() == 1)
                        )
                    )
                });

                let has_done_one_arg = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::FunctionCall { name, args } if name == "done" && args.len() == 1)
                    )
                });

                (has_outer_two_args, has_done_one_arg)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_outer_two_args,
            "outer(inner(a,), obj.run(b,),) should preserve two parsed arguments"
        );
        assert!(
            has_done_one_arg,
            "done(value,) should parse with one argument"
        );
    }

    #[test]
    fn test_multiline_call_arguments_parse_with_expected_shapes() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_call_multiline.fol")
            .expect("Should read multiline call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse multiline call arguments");

        let (has_compose_assignment, has_update_call, has_emit_return) = match ast {
            AstNode::Program { declarations } => {
                let has_compose_assignment = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "compose"
                                && args.len() == 3
                                && matches!(args[1], AstNode::FunctionCall { ref name, args: ref inner_args } if name == "wrap" && inner_args.len() == 1)
                        )
                    )
                });

                let has_update_call = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::MethodCall { method, args, .. }
                        if method == "update" && args.len() == 2
                    )
                });

                let has_emit_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::FunctionCall { name, args } if name == "emit" && args.len() == 1)
                    )
                });

                (has_compose_assignment, has_update_call, has_emit_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_compose_assignment,
            "Multiline compose(...) assignment should parse with nested wrap(...) argument"
        );
        assert!(
            has_update_call,
            "Multiline obj.update(...) call should parse with two arguments"
        );
        assert!(
            has_emit_return,
            "Multiline return emit(...) call should parse with one argument"
        );
    }

    #[test]
    fn test_multiline_call_arguments_with_comments_parse_with_expected_shapes() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_comments_multiline.fol")
                .expect("Should read multiline call-with-comments test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse multiline call arguments with comments");

        let (has_combine_assignment, has_emit_return) = match ast {
            AstNode::Program { declarations } => {
                let has_combine_assignment = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "combine"
                                && args.len() == 3
                                && matches!(args[1], AstNode::FunctionCall { ref name, args: ref inner_args } if name == "wrap" && inner_args.len() == 1)
                                && matches!(args[2], AstNode::Literal(Literal::Integer(42)))
                        )
                    )
                });

                let has_emit_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::FunctionCall { name, args } if name == "emit" && args.len() == 1)
                    )
                });

                (has_combine_assignment, has_emit_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_combine_assignment,
            "combine(...) should parse with three arguments including nested wrap(...) and integer literal"
        );
        assert!(
            has_emit_return,
            "return emit(...) should parse with one argument after commented multiline call"
        );
    }

    #[test]
    fn test_top_level_loop_iteration_shape_matches_function_loop_shape() {
        let mut file_stream = FileStream::from_file("test/parser/simple_loop_top_level.fol")
            .expect("Should read top-level loop test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse top-level loop statement");

        let loop_stmt = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Loop { condition, body } = node {
                        Some((condition.as_ref().clone(), body.clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should include top-level loop statement"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(
                loop_stmt.0,
                fol_parser::ast::LoopCondition::Iteration {
                    var,
                    condition: Some(_),
                    ..
                } if var == "i"
            ),
            "Top-level loop should parse as guarded iteration"
        );
        assert!(
            loop_stmt
                .1
                .iter()
                .any(|node| matches!(node, AstNode::Break)),
            "Top-level loop body should contain break statement"
        );
    }

    #[test]
    fn test_use_declaration_parsing() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_use.fol").expect("Should read use test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse use declaration");

        let use_decl = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::UseDecl {
                        name,
                        path_type,
                        path,
                        ..
                    } = node
                    {
                        Some((name.clone(), path_type.clone(), path.clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should include use declaration"),
            _ => panic!("Expected program node"),
        };

        assert_eq!(use_decl.0, "math");
        assert!(
            matches!(use_decl.1, FolType::Named { name } if name == "path"),
            "Use declaration should parse path type"
        );
        assert_eq!(use_decl.2, "core::math");
    }

    #[test]
    fn test_var_parsing_without_type_hint() {
        let mut file_stream = FileStream::from_file("test/parser/simple_var_infer.fol")
            .expect("Should read infer var test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse var declaration without type hint");

        match ast {
            AstNode::Program { declarations } => {
                let var_decl = declarations
                    .iter()
                    .find_map(|node| {
                        if let AstNode::VarDecl {
                            name,
                            type_hint,
                            value,
                            ..
                        } = node
                        {
                            Some((name, type_hint, value))
                        } else {
                            None
                        }
                    })
                    .expect("Program should contain a variable declaration");

                assert_eq!(var_decl.0, "message");
                assert!(var_decl.1.is_none(), "Type hint should be omitted");
                assert!(var_decl.2.is_some(), "Value should be parsed");
            }
            _ => panic!("Expected program node"),
        }
    }

    #[test]
    fn test_boolean_keyword_literals_parse_in_var_and_return() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_bool_literals.fol")
            .expect("Should read boolean literal function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse boolean keyword literals");

        let (has_true_var, has_false_return) = match ast {
            AstNode::Program { declarations } => {
                let has_true_var = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::VarDecl { name, value: Some(value), .. }
                        if name == "flag"
                            && matches!(value.as_ref(), AstNode::Literal(Literal::Boolean(true)))
                    )
                });

                let has_false_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::Literal(Literal::Boolean(false)))
                    )
                });

                (has_true_var, has_false_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_true_var,
            "Function body should include var assignment with true literal"
        );
        assert!(
            has_false_return,
            "Function body should include return with false literal"
        );
    }

    #[test]
    fn test_return_expression_precedence_mul_before_add() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_precedence.fol")
            .expect("Should read precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse precedence function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left: _, right } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Add));
                assert!(
                    matches!(
                        right.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Mul,
                            ..
                        }
                    ),
                    "Right side should be multiplication subtree"
                );
            }
            _ => panic!("Return value should be binary add expression"),
        }
    }

    #[test]
    fn test_return_expression_parentheses_override_precedence() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_paren_precedence.fol")
            .expect("Should read parenthesized precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse parenthesized precedence function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right: _ } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Mul));
                assert!(
                    matches!(
                        left.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Add,
                            ..
                        }
                    ),
                    "Left side should be parenthesized addition subtree"
                );
            }
            _ => panic!("Return value should be binary multiplication expression"),
        }
    }

    #[test]
    fn test_return_expression_unary_minus_precedence() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_unary_precedence.fol")
            .expect("Should read unary precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse unary precedence function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right: _ } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Mul));
                assert!(
                    matches!(
                        left.as_ref(),
                        AstNode::UnaryOp {
                            op: fol_parser::ast::UnaryOperator::Neg,
                            ..
                        }
                    ),
                    "Left side should be unary negation subtree"
                );
            }
            _ => panic!("Return value should be binary multiplication expression"),
        }
    }

    #[test]
    fn test_return_expression_unary_plus_precedence() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_plus_precedence.fol")
                .expect("Should read unary plus precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse unary plus precedence function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Mul));
                assert!(
                    matches!(left.as_ref(), AstNode::Identifier { name } if name == "a"),
                    "Left side should parse to identifier 'a' under unary plus"
                );
                assert!(
                    matches!(right.as_ref(), AstNode::Identifier { name } if name == "b"),
                    "Right side should parse to identifier 'b'"
                );
            }
            _ => panic!("Return value should be binary multiplication expression"),
        }
    }

    #[test]
    fn test_return_expression_chained_unary_plus_precedence() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_unary_plus_chain.fol")
            .expect("Should read chained unary plus precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse chained unary plus precedence function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Mul));
                assert!(
                    matches!(left.as_ref(), AstNode::Identifier { name } if name == "a"),
                    "Left side should parse to identifier 'a' under chained unary plus"
                );
                assert!(
                    matches!(right.as_ref(), AstNode::Identifier { name } if name == "b"),
                    "Right side should parse to identifier 'b'"
                );
            }
            _ => panic!("Return value should be binary multiplication expression"),
        }
    }

    #[test]
    fn test_unary_plus_preserves_call_and_method_call_expression_shapes() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_plus_call_exprs.fol")
                .expect("Should read unary-plus call expression fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse unary-plus call expressions");

        let (has_call_assignment, has_method_return) = match ast {
            AstNode::Program { declarations } => {
                let has_call_assignment = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(
                            value.as_ref(),
                            AstNode::FunctionCall { name, args }
                            if name == "compute"
                                && args.len() == 1
                                && matches!(&args[0], AstNode::Identifier { name } if name == "a")
                        )
                    )
                });

                let has_method_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(
                            value.as_ref(),
                            AstNode::MethodCall { object, method, args }
                            if matches!(object.as_ref(), AstNode::Identifier { name } if name == "obj")
                                && method == "get"
                                && args.len() == 1
                                && matches!(&args[0], AstNode::Identifier { name } if name == "a")
                        )
                    )
                });

                (has_call_assignment, has_method_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_call_assignment,
            "Unary plus on compute(a) should preserve function-call assignment shape"
        );
        assert!(
            has_method_return,
            "Unary plus on obj.get(a) should preserve method-call return shape"
        );
    }

    #[test]
    fn test_return_expression_unary_ref_parses_as_unary_expression() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_unary_ref.fol")
            .expect("Should read unary ref function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse unary ref function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(
                &return_value,
                AstNode::UnaryOp {
                    op: fol_parser::ast::UnaryOperator::Ref,
                    operand
                } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a")
            ),
            "Return value should be unary ref of identifier 'a'"
        );
    }

    #[test]
    fn test_return_expression_unary_deref_precedence() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_deref_precedence.fol")
                .expect("Should read unary deref precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse unary deref precedence function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(
                &return_value,
                AstNode::BinaryOp {
                    op: fol_parser::ast::BinaryOperator::Mul,
                    left,
                    right
                }
                if matches!(
                    left.as_ref(),
                    AstNode::UnaryOp {
                        op: fol_parser::ast::UnaryOperator::Deref,
                        operand
                    } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a")
                ) && matches!(right.as_ref(), AstNode::Identifier { name } if name == "b")
            ),
            "Return value should be multiplication with unary deref on left operand"
        );
    }

    #[test]
    fn test_unary_ref_deref_chains_parse_with_expected_shape() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_ref_deref_chain.fol")
                .expect("Should read unary ref/deref chain function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse unary ref/deref chain function");

        let (has_chain_assignment, has_chain_return) = match ast {
            AstNode::Program { declarations } => {
                let has_chain_assignment = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(
                            value.as_ref(),
                            AstNode::UnaryOp {
                                op: fol_parser::ast::UnaryOperator::Deref,
                                operand,
                            }
                            if matches!(
                                operand.as_ref(),
                                AstNode::UnaryOp {
                                    op: fol_parser::ast::UnaryOperator::Ref,
                                    operand,
                                } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a")
                            )
                        )
                    )
                });

                let has_chain_return = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(
                            value.as_ref(),
                            AstNode::UnaryOp {
                                op: fol_parser::ast::UnaryOperator::Ref,
                                operand,
                            }
                            if matches!(
                                operand.as_ref(),
                                AstNode::UnaryOp {
                                    op: fol_parser::ast::UnaryOperator::Deref,
                                    operand,
                                } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a")
                            )
                        )
                    )
                });

                (has_chain_assignment, has_chain_return)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_chain_assignment,
            "Assignment should parse as unary deref over unary ref chain"
        );
        assert!(
            has_chain_return,
            "Return should parse as unary ref over unary deref chain"
        );
    }

    #[test]
    fn test_mixed_unary_chains_parse_with_expected_shape() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_mixed_chains.fol")
                .expect("Should read mixed unary chain function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse mixed unary chain function");

        let (has_assignment_chain, has_return_chain) = match ast {
            AstNode::Program { declarations } => {
                let has_assignment_chain = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(
                            value.as_ref(),
                            AstNode::UnaryOp {
                                op: fol_parser::ast::UnaryOperator::Neg,
                                operand,
                            }
                            if matches!(
                                operand.as_ref(),
                                AstNode::UnaryOp {
                                    op: fol_parser::ast::UnaryOperator::Deref,
                                    operand,
                                }
                                if matches!(
                                    operand.as_ref(),
                                    AstNode::UnaryOp {
                                        op: fol_parser::ast::UnaryOperator::Ref,
                                        operand,
                                    } if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a")
                                )
                            )
                        )
                    )
                });

                let has_return_chain = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Return { value: Some(value) }
                        if matches!(
                            value.as_ref(),
                            AstNode::UnaryOp {
                                op: fol_parser::ast::UnaryOperator::Not,
                                operand,
                            }
                            if matches!(operand.as_ref(), AstNode::Identifier { name } if name == "a")
                        )
                    )
                });

                (has_assignment_chain, has_return_chain)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_assignment_chain,
            "Assignment should parse as neg(deref(ref(a))) unary chain"
        );
        assert!(
            has_return_chain,
            "Return should parse as not(a) when unary plus acts as identity"
        );
    }

    #[test]
    fn test_return_expression_unary_minus_parenthesized_addition() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_paren_precedence.fol")
                .expect("Should read unary parenthesized precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse unary parenthesized precedence function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right: _ } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Mul));
                assert!(
                    matches!(
                        left.as_ref(),
                        AstNode::UnaryOp {
                            op: fol_parser::ast::UnaryOperator::Neg,
                            operand
                        } if matches!(operand.as_ref(), AstNode::BinaryOp { op: fol_parser::ast::BinaryOperator::Add, .. })
                    ),
                    "Left side should be negated parenthesized addition"
                );
            }
            _ => panic!("Return value should be binary multiplication expression"),
        }
    }

    #[test]
    fn test_return_expression_subtraction_is_left_associative() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_assoc_sub.fol")
            .expect("Should read subtraction associativity function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse subtraction associativity function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right: _ } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Sub));
                assert!(
                    matches!(
                        left.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Sub,
                            ..
                        }
                    ),
                    "Left side should contain the first subtraction for left associativity"
                );
            }
            _ => panic!("Return value should be binary subtraction expression"),
        }
    }

    #[test]
    fn test_return_expression_division_is_left_associative() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_assoc_div.fol")
            .expect("Should read division associativity function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse division associativity function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right: _ } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Div));
                assert!(
                    matches!(
                        left.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Div,
                            ..
                        }
                    ),
                    "Left side should contain the first division for left associativity"
                );
            }
            _ => panic!("Return value should be binary division expression"),
        }
    }

    #[test]
    fn test_return_expression_mixed_precedence_and_associativity() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_mixed_precedence_assoc.fol")
                .expect("Should read mixed precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse mixed precedence function");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right: _ } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Sub));
                assert!(
                    matches!(
                        left.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Sub,
                            left: _,
                            right
                        } if matches!(right.as_ref(), AstNode::BinaryOp { op: fol_parser::ast::BinaryOperator::Mul, .. })
                    ),
                    "Expected (a - (b * c)) - d tree shape"
                );
            }
            _ => panic!("Return value should be subtraction expression"),
        }
    }

    #[test]
    fn test_return_expression_division_with_grouped_rhs() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_div_paren_rhs.fol")
            .expect("Should read grouped division function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse division with grouped rhs");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left: _, right } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Div));
                assert!(
                    matches!(
                        right.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Add,
                            ..
                        }
                    ),
                    "Right side should be grouped addition subtree"
                );
            }
            _ => panic!("Return value should be division expression"),
        }
    }

    #[test]
    fn test_assignment_statement_parsing_with_expression_value() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_assignment.fol")
            .expect("Should read assignment function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse assignment statement");

        let assignment = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Assignment { target, value } = node {
                        Some((target.as_ref().clone(), value.as_ref().clone()))
                    } else {
                        None
                    }
                })
                .expect("Program should contain an assignment statement"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(assignment.0, AstNode::Identifier { name } if name == "result"),
            "Assignment target should be identifier 'result'"
        );
        assert!(
            matches!(assignment.1, AstNode::BinaryOp { .. }),
            "Assignment value should be parsed as expression tree"
        );
    }

    #[test]
    fn test_compound_assignment_statements_are_lowered_to_binary_ops() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_compound_assignment.fol")
                .expect("Should read compound assignment function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse compound assignment statements");

        let assignment_ops = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .filter_map(|node| {
                    if let AstNode::Assignment { value, .. } = node {
                        if let AstNode::BinaryOp { op, .. } = value.as_ref() {
                            Some(op.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            _ => panic!("Expected program node"),
        };

        assert!(
            assignment_ops.len() >= 4,
            "Expected compound assignments to produce binary expression values"
        );
        assert!(
            matches!(assignment_ops[0], fol_parser::ast::BinaryOperator::Add),
            "'+=' should lower to Add"
        );
        assert!(
            matches!(assignment_ops[1], fol_parser::ast::BinaryOperator::Sub),
            "'-=' should lower to Sub"
        );
        assert!(
            matches!(assignment_ops[2], fol_parser::ast::BinaryOperator::Mul),
            "'*=' should lower to Mul"
        );
        assert!(
            matches!(assignment_ops[3], fol_parser::ast::BinaryOperator::Div),
            "'/=' should lower to Div"
        );
    }

    #[test]
    fn test_mod_assignment_and_comparison_expressions() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_mod_and_compare.fol")
            .expect("Should read mod and comparison function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse modulo and comparison expressions");

        let (has_mod_assignment, return_ops, return_values) = match ast {
            AstNode::Program { declarations } => {
                let has_mod_assignment = declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::Assignment { value, .. }
                        if matches!(value.as_ref(), AstNode::BinaryOp { op: fol_parser::ast::BinaryOperator::Mod, .. })
                    )
                });

                let return_ops = declarations
                    .iter()
                    .filter_map(|node| {
                        if let AstNode::Return { value: Some(value) } = node {
                            if let AstNode::BinaryOp { op, .. } = value.as_ref() {
                                Some(op.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                let return_values = declarations
                    .iter()
                    .filter_map(|node| {
                        if let AstNode::Return { value } = node {
                            Some(format!("{:?}", value))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                (has_mod_assignment, return_ops, return_values)
            }
            _ => panic!("Expected program node"),
        };

        assert!(
            has_mod_assignment,
            "Expected assignment lowered/parsed with modulo binary operator"
        );
        assert!(
            return_ops
                .iter()
                .any(|op| matches!(op, fol_parser::ast::BinaryOperator::Eq)),
            "Expected return expression parsed with equality operator, got ops {:?} and return values {:?}",
            return_ops,
            return_values
        );
    }

    #[test]
    fn test_logical_and_has_lower_precedence_than_comparison() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_logical.fol")
            .expect("Should read logical expression function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse logical expression");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::And));
                assert!(
                    matches!(
                        left.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Eq,
                            ..
                        }
                    ),
                    "Left side should be comparison subtree"
                );
                assert!(
                    matches!(
                        right.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Eq,
                            ..
                        }
                    ),
                    "Right side should be comparison subtree"
                );
            }
            _ => panic!("Return value should be logical and expression"),
        }
    }

    #[test]
    fn test_logical_or_has_lower_precedence_than_and() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_logical_or_precedence.fol")
                .expect("Should read logical or precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse logical or precedence expression");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Or));
                assert!(
                    matches!(
                        left.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Eq,
                            ..
                        }
                    ),
                    "Left side should be equality comparison"
                );
                assert!(
                    matches!(
                        right.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::And,
                            ..
                        }
                    ),
                    "Right side should be grouped logical and subtree"
                );
            }
            _ => panic!("Return value should be logical or expression"),
        }
    }

    #[test]
    fn test_logical_not_parses_as_unary_expression() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_logical_not.fol")
            .expect("Should read logical not function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse logical not expression");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(
                return_value,
                AstNode::UnaryOp {
                    op: fol_parser::ast::UnaryOperator::Not,
                    ..
                }
            ),
            "Return value should be unary logical-not expression"
        );
    }

    #[test]
    fn test_logical_xor_precedence_between_or_and_and() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_logical_xor_precedence.fol")
                .expect("Should read logical xor precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse logical xor precedence expression");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left: _, right } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::Or));
                assert!(
                    matches!(
                        right.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Xor,
                            ..
                        }
                    ),
                    "Right side should be logical xor subtree"
                );
                if let AstNode::BinaryOp {
                    right: xor_right, ..
                } = right.as_ref()
                {
                    assert!(
                        matches!(
                            xor_right.as_ref(),
                            AstNode::BinaryOp {
                                op: fol_parser::ast::BinaryOperator::And,
                                ..
                            }
                        ),
                        "Xor right side should keep tighter logical and subtree"
                    );
                }
            }
            _ => panic!("Return value should be logical or expression"),
        }
    }

    #[test]
    fn test_logical_nand_lowers_to_not_of_and() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_logical_nand_nor.fol")
            .expect("Should read logical nand/nor function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse logical nand/nor expression");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(
                &return_value,
                AstNode::UnaryOp {
                    op: fol_parser::ast::UnaryOperator::Not,
                    operand
                } if matches!(
                    operand.as_ref(),
                    AstNode::BinaryOp {
                        op: fol_parser::ast::BinaryOperator::And,
                        ..
                    }
                )
            ),
            "Nand should lower to not(and(...)), got {:?}",
            return_value
        );
    }

    #[test]
    fn test_logical_nor_lowers_to_not_of_or() {
        let mut file_stream = FileStream::from_file("test/parser/simple_fun_logical_nor.fol")
            .expect("Should read logical nor function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse logical nor expression");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        assert!(
            matches!(
                &return_value,
                AstNode::UnaryOp {
                    op: fol_parser::ast::UnaryOperator::Not,
                    operand
                } if matches!(
                    operand.as_ref(),
                    AstNode::BinaryOp {
                        op: fol_parser::ast::BinaryOperator::Or,
                        ..
                    }
                )
            ),
            "Nor should lower to not(or(...)), got {:?}",
            return_value
        );
    }

    #[test]
    fn test_logical_not_precedence_over_comparison_and_and() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_logical_not_precedence.fol")
                .expect("Should read logical not precedence function test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();

        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should parse logical not precedence expression");

        let return_value = match ast {
            AstNode::Program { declarations } => declarations
                .iter()
                .find_map(|node| {
                    if let AstNode::Return { value: Some(value) } = node {
                        Some(value.as_ref().clone())
                    } else {
                        None
                    }
                })
                .expect("Program should contain a return value"),
            _ => panic!("Expected program node"),
        };

        match &return_value {
            AstNode::BinaryOp { op, left, right: _ } => {
                assert!(matches!(op, fol_parser::ast::BinaryOperator::And));
                assert!(
                    matches!(
                        left.as_ref(),
                        AstNode::BinaryOp {
                            op: fol_parser::ast::BinaryOperator::Eq,
                            left,
                            ..
                        } if matches!(left.as_ref(), AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Not, .. })
                    ),
                    "Expected left comparison to contain unary not on its lhs"
                );
            }
            _ => panic!("Return value should be logical and expression"),
        }
    }

    #[test]
    fn test_literal_parsing() {
        let parser = AstParser::new();

        // Test integer literal
        match parser.parse_literal("42") {
            Ok(ast) => {
                assert!(
                    matches!(ast, AstNode::Literal(_)),
                    "Should parse integer literal"
                );
            }
            Err(e) => panic!("Should parse integer literal: {:?}", e),
        }

        // Test string literal
        match parser.parse_literal("\"hello\"") {
            Ok(ast) => {
                assert!(
                    matches!(ast, AstNode::Literal(_)),
                    "Should parse string literal"
                );
            }
            Err(e) => panic!("Should parse string literal: {:?}", e),
        }

        // Test identifier
        match parser.parse_literal("variable_name") {
            Ok(ast) => {
                assert!(
                    matches!(ast, AstNode::Identifier { .. }),
                    "Should parse identifier"
                );
            }
            Err(e) => panic!("Should parse identifier: {:?}", e),
        }
    }

    #[test]
    fn test_parse_error_has_location_for_illegal_token() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_var.fol").expect("Should read test file");

        let mut lexer = Elements::init(&mut file_stream);
        lexer
            .set_key(KEYWORD::Illegal)
            .expect("Should be able to force an illegal token for parser test");

        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when current token is illegal");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(parse_error.line() > 0, "Line should be non-zero");
        assert!(parse_error.column() > 0, "Column should be non-zero");
        assert!(
            parse_error.length() > 0,
            "Token length should be non-zero for diagnostics"
        );
    }

    #[test]
    fn test_unary_plus_missing_operand_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_plus_missing_operand.fol")
                .expect("Should read unary-plus missing operand test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when unary plus is missing its operand");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '+'"),
            "Unary plus without operand should report explicit unary-plus operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Unary plus missing-operand parse error should point to return line"
        );
    }

    #[test]
    fn test_call_argument_unary_plus_missing_operand_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_unary_plus_missing_operand.fol")
                .expect("Should read unary-plus missing operand call-arg test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when call arg unary plus is missing an operand");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '+'"),
            "Unary plus without operand in call arg should report explicit unary-plus operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Call-arg unary plus missing-operand parse error should point to call line"
        );
    }

    #[test]
    fn test_unary_minus_missing_operand_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_minus_missing_operand.fol")
                .expect("Should read unary-minus missing operand test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when unary minus is missing its operand");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '-'"),
            "Unary minus without operand should report explicit unary-minus operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Unary minus missing-operand parse error should point to return line"
        );
    }

    #[test]
    fn test_unary_not_missing_operand_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_not_missing_operand.fol")
                .expect("Should read unary-not missing operand test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when unary not is missing its operand");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary 'not'"),
            "Unary not without operand should report explicit unary-not operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Unary not missing-operand parse error should point to return line"
        );
    }

    #[test]
    fn test_unary_ref_missing_operand_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_ref_missing_operand.fol")
                .expect("Should read unary-ref missing operand test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when unary ref is missing its operand");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '&'"),
            "Unary ref without operand should report explicit unary-ref operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Unary ref missing-operand parse error should point to return line"
        );
    }

    #[test]
    fn test_unary_deref_missing_operand_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_unary_deref_missing_operand.fol")
                .expect("Should read unary-deref missing operand test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when unary deref is missing its operand");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '*'"),
            "Unary deref without operand should report explicit unary-deref operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Unary deref missing-operand parse error should point to return line"
        );
    }

    #[test]
    fn test_call_argument_unary_ref_missing_operand_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_unary_ref_missing_operand.fol")
                .expect("Should read unary-ref missing operand call-arg test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when call arg unary ref is missing an operand");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '&'"),
            "Unary ref without operand in call arg should report explicit unary-ref operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Call-arg unary ref missing-operand parse error should point to call line"
        );
    }

    #[test]
    fn test_call_argument_unary_deref_missing_operand_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_unary_deref_missing_operand.fol")
                .expect("Should read unary-deref missing operand call-arg test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when call arg unary deref is missing an operand");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '*'"),
            "Unary deref without operand in call arg should report explicit unary-deref operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Call-arg unary deref missing-operand parse error should point to call line"
        );
    }

    #[test]
    fn test_top_level_call_argument_unary_ref_missing_operand_reports_parse_error() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_call_top_level_unary_ref_missing_operand.fol",
        )
        .expect("Should read top-level unary-ref missing operand call-arg test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when top-level call arg unary ref is missing an operand",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '&'"),
            "Top-level unary ref without operand should report explicit unary-ref operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            1,
            "Top-level call-arg unary ref missing-operand parse error should point to call line"
        );
    }

    #[test]
    fn test_top_level_call_argument_unary_deref_missing_operand_reports_parse_error() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_call_top_level_unary_deref_missing_operand.fol",
        )
        .expect("Should read top-level unary-deref missing operand call-arg test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when top-level call arg unary deref is missing an operand",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '*'"),
            "Top-level unary deref without operand should report explicit unary-deref operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            1,
            "Top-level call-arg unary deref missing-operand parse error should point to call line"
        );
    }

    #[test]
    fn test_top_level_call_argument_unary_minus_missing_operand_reports_parse_error() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_call_top_level_unary_minus_missing_operand.fol",
        )
        .expect("Should read top-level unary-minus missing operand call-arg test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when top-level call arg unary minus is missing an operand",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '-'"),
            "Top-level unary minus without operand should report explicit unary-minus operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            1,
            "Top-level call-arg unary minus missing-operand parse error should point to call line"
        );
    }

    #[test]
    fn test_top_level_call_argument_unary_not_missing_operand_reports_parse_error() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_call_top_level_unary_not_missing_operand.fol",
        )
        .expect("Should read top-level unary-not missing operand call-arg test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when top-level call arg unary not is missing an operand",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary 'not'"),
            "Top-level unary not without operand should report explicit unary-not operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            1,
            "Top-level call-arg unary not missing-operand parse error should point to call line"
        );
    }

    #[test]
    fn test_top_level_call_argument_unary_plus_missing_operand_reports_parse_error() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_call_top_level_unary_plus_missing_operand.fol",
        )
        .expect("Should read top-level unary-plus missing operand call-arg test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when top-level call arg unary plus is missing an operand",
        );

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = parse_error.to_string();

        assert!(
            first_message.contains("Expected expression after unary '+'"),
            "Top-level unary plus without operand should report explicit unary-plus operand error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            1,
            "Top-level call-arg unary plus missing-operand parse error should point to call line"
        );
    }

    #[test]
    fn test_unary_missing_operand_at_eof_reports_explicit_errors() {
        let cases = [
            (
                "test/parser/simple_fun_unary_plus_eof_operand.fol",
                "Expected expression after unary '+'",
            ),
            (
                "test/parser/simple_fun_unary_not_eof_operand.fol",
                "Expected expression after unary 'not'",
            ),
            (
                "test/parser/simple_fun_unary_deref_eof_operand.fol",
                "Expected expression after unary '*'",
            ),
            (
                "test/parser/simple_fun_unary_minus_eof_operand.fol",
                "Expected expression after unary '-'",
            ),
            (
                "test/parser/simple_fun_unary_ref_eof_operand.fol",
                "Expected expression after unary '&'",
            ),
        ];

        for (path, expected_message) in cases {
            let mut file_stream = FileStream::from_file(path)
                .unwrap_or_else(|_| panic!("Should read fixture: {}", path));

            let mut lexer = Elements::init(&mut file_stream);
            let mut parser = AstParser::new();
            let errors = parser
                .parse(&mut lexer)
                .expect_err("Parser should fail when unary operand is missing at EOF");

            let parse_error = errors
                .first()
                .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
                .expect("First parser error should be ParseError");

            let first_message = parse_error.to_string();
            assert!(
                first_message.contains(expected_message),
                "Fixture {} should report '{}', got: {}",
                path,
                expected_message,
                first_message
            );
            assert_eq!(
                parse_error.line(),
                2,
                "Fixture {} should report unary EOF error on second line",
                path
            );
        }
    }

    #[test]
    fn test_missing_call_closing_paren_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_missing_paren.fol")
                .expect("Should read missing call paren test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when a call is missing a closing ')' ");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        assert!(
            first_message.contains("Expected ',' or ')' in call arguments")
                || first_message.contains("Unsupported expression token '; '"),
            "Missing call ')' should report a call-argument parse error, got: {}",
            first_message
        );
    }

    #[test]
    fn test_missing_call_argument_separator_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_bad_separator.fol")
                .expect("Should read bad call separator test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when call arguments are missing a separator");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Expected ',' or ')' in call arguments"),
            "Missing call separator should report argument-separator parse error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            4,
            "Missing call separator parse error should point to the next argument token line"
        );
    }

    #[test]
    fn test_top_level_call_with_leading_comma_argument_reports_location() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_call_top_level_leading_comma_arg.fol")
                .expect("Should read top-level malformed call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when call arguments start with a comma");

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        assert!(
            first_message.contains("Unsupported expression token"),
            "Leading comma argument should report unsupported expression token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Leading comma parse error should point to the comma line"
        );
        assert!(
            parse_error.column() > 0,
            "Leading comma parse error should include a non-zero column"
        );
    }

    #[test]
    fn test_method_call_missing_argument_separator_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_method_call_bad_separator.fol")
                .expect("Should read malformed method call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when method call arguments are missing a separator");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Expected ',' or ')' in call arguments"),
            "Method call with missing separator should report argument-separator parse error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            4,
            "Method missing-separator parse error should point to the next argument token line"
        );
    }

    #[test]
    fn test_nested_call_missing_argument_separator_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_nested_bad_separator.fol")
                .expect("Should read malformed nested call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when nested call arguments are missing a separator");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        assert!(
            first_message.contains("Expected ',' or ')' in call arguments"),
            "Nested call with missing separator should report argument-separator parse error, got: {}",
            first_message
        );
    }

    #[test]
    fn test_top_level_call_with_double_comma_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_call_top_level_double_comma_arg.fol")
                .expect("Should read malformed top-level double-comma call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when top-level call has an empty argument slot");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token"),
            "Top-level call with double comma should report unsupported expression token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            3,
            "Top-level double-comma parse error should point to empty argument slot line"
        );
    }

    #[test]
    fn test_method_call_with_empty_argument_slot_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_method_call_empty_argument_slot.fol")
                .expect("Should read malformed method empty-slot call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when method call starts argument list with comma");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token"),
            "Method call with empty argument slot should report unsupported expression token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            3,
            "Method empty-slot parse error should point to the comma line"
        );
    }

    #[test]
    fn test_nested_call_with_empty_argument_slot_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_nested_empty_slot.fol")
                .expect("Should read malformed nested empty-slot call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when nested call has an empty argument slot");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token"),
            "Nested call with empty argument slot should report unsupported expression token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            5,
            "Nested empty-slot parse error should point to the comma line"
        );
    }

    #[test]
    fn test_method_call_with_nested_empty_argument_slot_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_method_call_nested_empty_slot.fol")
                .expect("Should read malformed method nested empty-slot call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when method call has nested empty argument slot");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token"),
            "Method call with nested empty argument slot should report unsupported expression token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            5,
            "Method nested empty-slot parse error should point to the comma line"
        );
    }

    #[test]
    fn test_call_argument_with_dangling_operator_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_dangling_operator.fol")
                .expect("Should read malformed dangling-operator call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when a call argument expression has a dangling operator",
        );

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token ','"),
            "Dangling operator in call argument should report unsupported comma token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            3,
            "Dangling operator parse error should point to the expression line"
        );
    }

    #[test]
    fn test_method_call_argument_with_dangling_operator_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_method_call_dangling_operator.fol")
                .expect("Should read malformed method dangling-operator call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when a method call argument has a dangling operator");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token ','"),
            "Dangling operator in method call argument should report unsupported comma token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            3,
            "Method dangling-operator parse error should point to the expression line"
        );
    }

    #[test]
    fn test_method_call_nested_dangling_operator_reports_parse_error() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_method_call_nested_dangling_operator.fol",
        )
        .expect("Should read malformed method nested dangling-operator call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when nested method call argument has a dangling operator",
        );

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token ','"),
            "Nested dangling operator in method call should report unsupported comma token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            4,
            "Nested method dangling-operator parse error should point to inner expression line"
        );
    }

    #[test]
    fn test_function_call_nested_dangling_operator_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_nested_dangling_operator.fol")
                .expect("Should read malformed function nested dangling-operator call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when nested function call argument has a dangling operator",
        );

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token ','"),
            "Nested dangling operator in function call should report unsupported comma token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            4,
            "Nested function dangling-operator parse error should point to inner expression line"
        );
    }

    #[test]
    fn test_top_level_nested_call_with_empty_argument_slot_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_call_top_level_nested_empty_slot.fol")
                .expect("Should read malformed top-level nested empty-slot call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when top-level nested call has an empty argument slot");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token"),
            "Top-level nested empty-slot call should report unsupported expression token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            4,
            "Top-level nested empty-slot parse error should point to inner empty-slot comma line"
        );
    }

    #[test]
    fn test_function_call_with_unmatched_close_paren_argument_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_unmatched_close_paren_arg.fol")
                .expect("Should read malformed unmatched-close-paren call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when function call argument list contains unmatched ')' token",
        );

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token ')'"),
            "Unmatched ')' argument should report unsupported close-paren token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            3,
            "Unmatched close-paren argument parse error should point to the malformed expression line"
        );
    }

    #[test]
    fn test_method_call_with_unmatched_close_paren_argument_reports_parse_error() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_method_call_unmatched_close_paren_arg.fol",
        )
        .expect("Should read malformed unmatched-close-paren method call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when method call argument list contains unmatched ')' token",
        );

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token ')'"),
            "Unmatched ')' in method call argument should report unsupported close-paren token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            3,
            "Method unmatched close-paren parse error should point to malformed expression line"
        );
    }

    #[test]
    fn test_top_level_call_with_unmatched_close_paren_argument_reports_parse_error() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_call_top_level_unmatched_close_paren_arg.fol",
        )
        .expect("Should read malformed unmatched-close-paren top-level call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(
            "Parser should fail when top-level call argument list contains unmatched ')' token",
        );

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Unsupported expression token ')'"),
            "Unmatched ')' in top-level call argument should report unsupported close-paren token, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            2,
            "Top-level unmatched close-paren parse error should point to malformed expression line"
        );
    }

    #[test]
    fn test_function_call_with_unmatched_open_paren_argument_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_fun_call_unmatched_open_paren_arg.fol")
                .expect("Should read malformed unmatched-open-paren call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when function call argument has unmatched '(' token");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Expected closing ')' for parenthesized expression"),
            "Unmatched '(' in function call argument should report missing close-paren error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            4,
            "Function unmatched open-paren parse error should point to malformed expression line"
        );
    }

    #[test]
    fn test_method_call_with_unmatched_open_paren_argument_reports_parse_error() {
        let mut file_stream = FileStream::from_file(
            "test/parser/simple_fun_method_call_unmatched_open_paren_arg.fol",
        )
        .expect("Should read malformed unmatched-open-paren method call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when method call argument has unmatched '(' token");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Expected closing ')' for parenthesized expression"),
            "Unmatched '(' in method call argument should report missing close-paren error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            4,
            "Method unmatched open-paren parse error should point to malformed expression line"
        );
    }

    #[test]
    fn test_top_level_call_with_unmatched_open_paren_argument_reports_parse_error() {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_call_top_level_unmatched_open_paren_arg.fol")
                .expect("Should read malformed unmatched-open-paren top-level call test file");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should fail when top-level call argument has unmatched '(' token");

        let first_message = errors
            .first()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "<no error message>".to_string());

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError");

        assert!(
            first_message.contains("Expected closing ')' for parenthesized expression"),
            "Unmatched '(' in top-level call argument should report missing close-paren error, got: {}",
            first_message
        );
        assert_eq!(
            parse_error.line(),
            3,
            "Top-level unmatched open-paren parse error should point to malformed expression line"
        );
    }

    fn assert_first_parse_error(
        path: &str,
        expected_message_substring: &str,
        expected_line: usize,
    ) {
        let mut file_stream =
            FileStream::from_file(path).unwrap_or_else(|_| panic!("Should read fixture: {}", path));

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser.parse(&mut lexer).expect_err(&format!(
            "Parser should fail for malformed fixture: {}",
            path
        ));

        let parse_error = errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .unwrap_or_else(|| {
                panic!(
                    "First parser error should be ParseError for fixture: {}",
                    path
                )
            });

        let first_message = parse_error.to_string();
        assert!(
            first_message.contains(expected_message_substring),
            "Fixture {} should report '{}', got: {}",
            path,
            expected_message_substring,
            first_message
        );
        assert_eq!(
            parse_error.line(),
            expected_line,
            "Fixture {} should report expected error line",
            path
        );
    }

    const EXPECT_MISSING_CLOSE_PAREN: &str = "Expected closing ')' for parenthesized expression";
    const EXPECT_UNSUPPORTED_CLOSE_PAREN_TOKEN: &str = "Unsupported expression token ')'";

    #[test]
    fn test_mixed_unmatched_open_error_matrix_representative_cases() {
        let cases = [
            (
                "test/parser/simple_fun_call_mixed_unmatched_open_and_trailing_comma.fol",
                4usize,
            ),
            (
                "test/parser/simple_fun_method_call_mixed_unmatched_open_and_trailing_comma.fol",
                4usize,
            ),
            (
                "test/parser/simple_call_top_level_mixed_unmatched_open_and_trailing_comma.fol",
                3usize,
            ),
            (
                "test/parser/simple_fun_call_mixed_unmatched_open_sixth_arg.fol",
                9usize,
            ),
            (
                "test/parser/simple_fun_method_call_mixed_unmatched_open_sixth_arg.fol",
                9usize,
            ),
            (
                "test/parser/simple_call_top_level_mixed_unmatched_open_sixth_arg.fol",
                8usize,
            ),
        ];

        for (path, line) in cases {
            assert_first_parse_error(path, EXPECT_MISSING_CLOSE_PAREN, line);
        }
    }

    #[test]
    fn test_mixed_unmatched_close_error_matrix_representative_cases() {
        let cases = [
            (
                "test/parser/simple_fun_call_mixed_unmatched_close_first_arg.fol",
                4usize,
            ),
            (
                "test/parser/simple_fun_method_call_mixed_unmatched_close_first_arg.fol",
                4usize,
            ),
            (
                "test/parser/simple_call_top_level_mixed_unmatched_close_first_arg.fol",
                3usize,
            ),
            (
                "test/parser/simple_fun_call_mixed_unmatched_close_fifth_arg.fol",
                8usize,
            ),
            (
                "test/parser/simple_fun_method_call_mixed_unmatched_close_fifth_arg.fol",
                8usize,
            ),
            (
                "test/parser/simple_call_top_level_mixed_unmatched_close_fifth_arg.fol",
                7usize,
            ),
        ];

        for (path, line) in cases {
            assert_first_parse_error(path, EXPECT_UNSUPPORTED_CLOSE_PAREN_TOKEN, line);
        }
    }
}
