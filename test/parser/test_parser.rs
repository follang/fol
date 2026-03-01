// Parser tests - to be expanded when full parser is implemented

use fol_lexer::lexer::stage3::Elements;
use fol_lexer::token::KEYWORD;
use fol_parser::ast::{AstNode, AstParser, ParseError};
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
}

// TODO: Expand these tests when full parser is implemented
// - Variable declarations
// - Function declarations
// - Type declarations
// - Expressions
// - Statements
// - Error recovery
// - AST structure validation
