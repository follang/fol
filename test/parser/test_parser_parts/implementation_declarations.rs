use super::*;
use fol_parser::ast::DeclOption;

#[test]
fn test_basic_implementation_declaration_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_basic.fol")
        .expect("Should read implementation declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse basic implementation declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ImpDecl { name, target, body, generics, .. }
                    if name == "Self"
                        && generics.is_empty()
                        && matches!(target, FolType::Named { name } if name == "ID")
                        && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, .. } if name == "ready"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_declaration_parsing_inside_function_bodies() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_imp_basic.fol")
        .expect("Should read function-body implementation declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse implementation declarations inside function bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(stmt, AstNode::ImpDecl { name, .. } if name == "Local"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_declaration_accepts_empty_option_brackets() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_empty_options.fol")
        .expect("Should read empty-option implementation declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept imp[] declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::ImpDecl { name, .. } if name == "Self")
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_declaration_accepts_empty_marker_form() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_marker.fol")
        .expect("Should read marker-form implementation declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept marker-form implementation declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ImpDecl { name, target, body, generics, .. }
                if name == "Self"
                    && generics.is_empty()
                    && matches!(target, FolType::Named { name } if name == "ID")
                    && body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_declaration_accepts_empty_option_marker_form() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_empty_options_marker.fol")
        .expect("Should read empty-option marker implementation fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept imp[] marker declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ImpDecl { name, target, body, generics, .. }
                if name == "Self"
                    && generics.is_empty()
                    && matches!(target, FolType::Named { name } if name == "ID")
                    && body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_declaration_rejects_unknown_options() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_unknown_option.fol")
        .expect("Should read malformed imp option test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject non-empty imp option lists");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Unknown implementation option"),
        "Malformed imp option list should report unsupported options, got: {}",
        parse_error
    );
}

#[test]
fn test_implementation_declaration_supports_generic_headers() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_generics.fol")
        .expect("Should read generic implementation declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse implementation generic headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ImpDecl { name, generics, target, .. }
                    if name == "Self"
                        && generics.len() == 2
                        && generics[0].name == "T"
                        && generics[1].name == "U"
                        && matches!(target, FolType::Named { name } if name == "Pair[T,U]")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_declaration_visibility_options_parse() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_imp_visibility_options.fol")
            .expect("Should read visibility-option implementation test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse implementation visibility options");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ImpDecl { name, options, .. }
                if name == "Self" && options == &vec![DeclOption::Export]
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_declaration_rejects_conflicting_visibility_options() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_imp_conflicting_options.fol")
            .expect("Should read conflicting imp option test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject conflicting implementation options");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Conflicting implementation visibility options"),
        "Conflicting imp options should report a targeted error, got: {}",
        parse_error
    );
}

#[test]
fn test_implementation_generic_header_missing_separator_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_imp_generics_missing_separator.fol")
            .expect("Should read malformed imp generic header test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject malformed imp generic headers");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Expected ','; ';', or ')' after generic parameter")
            || parse_error
                .to_string()
                .contains("Expected ',', ';', or ')' after generic parameter"),
        "Malformed imp generic header should report separator error, got: {}",
        parse_error
    );
}

#[test]
fn test_implementation_declaration_accepts_quoted_names() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_quoted_name.fol")
        .expect("Should read quoted-name implementation declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse quoted implementation names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ImpDecl { name, target, body, .. }
                    if name == "math"
                        && matches!(target, FolType::Named { name } if name == "Number")
                        && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, .. } if name == "run"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_declaration_supports_bracketed_generic_constraints() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_bracketed_generics.fol")
        .expect("Should read bracketed generic implementation fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse bracketed implementation generic constraints");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ImpDecl { name, generics, .. }
                    if name == "Self"
                        && generics.len() == 2
                        && generics[0].name == "T"
                        && generics[1].name == "U"
                        && generics[0].constraints.len() == 1
                        && generics[1].constraints.len() == 1
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
