use super::*;
use fol_parser::ast::StandardKind;

#[test]
fn test_protocol_standard_declaration_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_protocol.fol")
        .expect("Should read protocol standard test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse protocol standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Protocol, body }
                    if name == "geometry"
                        && body.len() == 2
                        && matches!(&body[0], AstNode::FunDecl { name, body, .. } if name == "area" && body.is_empty())
                        && matches!(&body[1], AstNode::FunDecl { name, body, .. } if name == "perim" && body.is_empty())
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_blueprint_standard_declaration_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_blueprint.fol")
        .expect("Should read blueprint standard test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse blueprint standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Blueprint, body }
                    if name == "geometry"
                        && body.len() == 2
                        && matches!(&body[0], AstNode::VarDecl { name, .. } if name == "color")
                        && matches!(&body[1], AstNode::VarDecl { name, .. } if name == "size")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_extended_standard_declaration_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_extended.fol")
        .expect("Should read extended standard test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse extended standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Extended, body }
                    if name == "geometry"
                        && body.len() == 2
                        && matches!(&body[0], AstNode::FunDecl { name, .. } if name == "area")
                        && matches!(&body[1], AstNode::VarDecl { name, .. } if name == "color")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_standard_declaration_parsing_inside_function_bodies() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_std_protocol.fol")
        .expect("Should read nested standard test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse standards inside function bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { name, body, .. }
                    if name == "build"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::StdDecl { name, kind: StandardKind::Protocol, .. } if name == "geometry"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_standard_declaration_accepts_empty_options() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_empty_options.fol")
        .expect("Should read std[] test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse std declarations with empty options");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::StdDecl { name, .. } if name == "geometry")
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_protocol_standard_accepts_empty_kind_brackets() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_protocol_kind_options.fol")
            .expect("Should read pro[] standard test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse pro[] standard declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Protocol, .. }
                    if name == "geometry"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_blueprint_standard_accepts_empty_kind_brackets() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_blueprint_kind_options.fol")
            .expect("Should read blu[] standard test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse blu[] standard declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Blueprint, .. }
                    if name == "geometry"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_extended_standard_accepts_empty_kind_brackets() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_extended_kind_options.fol")
            .expect("Should read ext[] standard test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse ext[] standard declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Extended, .. }
                    if name == "geometry"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_protocol_standard_rejects_duplicate_signatures() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_protocol_duplicate_signature.fol")
            .expect("Should read duplicate protocol standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate protocol signatures");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Duplicate standard member 'area#0'"),
        "Expected duplicate standard signature error, got: {}",
        parse_error
    );
}
