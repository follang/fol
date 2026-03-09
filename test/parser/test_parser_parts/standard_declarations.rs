use super::*;
use fol_parser::ast::{DeclOption, StandardKind, VarOption};

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
                    AstNode::StdDecl { name, kind: StandardKind::Protocol, body, .. }
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
fn test_protocol_standard_accepts_default_function_implementations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_protocol_default_fun.fol")
        .expect("Should read protocol default-function standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse default function implementations in protocol standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Protocol, body, .. }
                    if name == "geometry"
                        && body.len() == 2
                        && matches!(&body[0], AstNode::FunDecl { name, body, .. } if name == "area" && !body.is_empty())
                        && matches!(&body[1], AstNode::FunDecl { name, body, .. } if name == "perim" && body.is_empty())
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_protocol_standard_accepts_alias_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_protocol_alias.fol")
        .expect("Should read protocol alias-member standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alias members in protocol standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Protocol, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::AliasDecl { name, .. } if name == "Area"))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, return_type: Some(FolType::Named { name: ret }), .. } if name == "area" && ret == "Area"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_protocol_standard_accepts_type_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_protocol_type.fol")
        .expect("Should read protocol type-member standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse type members in protocol standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Protocol, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::TypeDecl { name, .. } if name == "Area"))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, return_type: Some(FolType::Named { name: ret }), .. } if name == "area" && ret == "Area"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_protocol_standard_accepts_constant_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_protocol_const.fol")
        .expect("Should read protocol constant-member standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse constant members in protocol standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Protocol, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, options, .. } if name == "epsilon" && options.contains(&VarOption::Immutable)))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, .. } if name == "area"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_standard_routine_members_accept_capture_lists() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_protocol_captures.fol")
        .expect("Should read standard routine capture fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse capture lists on standard routine members");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, kind: StandardKind::Protocol, body, .. }
                if name == "geometry"
                    && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, captures, .. } if name == "area" && captures == &vec!["unit".to_string()]))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_protocol_standard_accepts_default_procedure_implementations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_protocol_default_pro.fol")
        .expect("Should read protocol default-procedure standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse default procedure implementations in protocol standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, kind: StandardKind::Protocol, body, .. }
                if name == "geometry"
                    && body.iter().any(|stmt| matches!(stmt, AstNode::ProDecl { name, body, .. } if name == "reset" && !body.is_empty()))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_protocol_standard_accepts_default_logical_implementations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_protocol_default_log.fol")
        .expect("Should read protocol default-logical standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse default logical implementations in protocol standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, kind: StandardKind::Protocol, body, .. }
                if name == "geometry"
                    && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, body, .. } if name == "valid" && !body.is_empty()))
            )));
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
                    AstNode::StdDecl { name, kind: StandardKind::Blueprint, body, .. }
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
fn test_blueprint_standard_accepts_routine_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_blueprint_methods.fol")
        .expect("Should read blueprint routine-member standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse routine members in blueprint standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Blueprint, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, .. } if name == "color"))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, body, .. } if name == "describe" && !body.is_empty()))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::ProDecl { name, body, .. } if name == "reset" && body.is_empty()))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_blueprint_standard_accepts_alias_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_blueprint_alias.fol")
        .expect("Should read blueprint alias-member standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alias members in blueprint standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Blueprint, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::AliasDecl { name, .. } if name == "Color"))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, type_hint: Some(FolType::Named { name: typ }), .. } if name == "color" && typ == "Color"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_blueprint_standard_accepts_type_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_blueprint_type.fol")
        .expect("Should read blueprint type-member standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse type members in blueprint standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Blueprint, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::TypeDecl { name, .. } if name == "Color"))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, type_hint: Some(FolType::Named { name: typ }), .. } if name == "color" && typ == "Color"))
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
                    AstNode::StdDecl { name, kind: StandardKind::Extended, body, .. }
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
fn test_extended_standard_accepts_alias_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_extended_alias.fol")
        .expect("Should read extended alias-member standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alias members in extended standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Extended, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::AliasDecl { name, .. } if name == "Color"))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, type_hint: Some(FolType::Named { name: typ }), .. } if name == "color" && typ == "Color"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_extended_standard_accepts_type_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_extended_type.fol")
        .expect("Should read extended type-member standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse type members in extended standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Extended, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::TypeDecl { name, .. } if name == "Color"))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, type_hint: Some(FolType::Named { name: typ }), .. } if name == "color" && typ == "Color"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_extended_standard_accepts_default_function_implementations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_extended_default_fun.fol")
        .expect("Should read extended default-function standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse default function implementations in extended standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, kind: StandardKind::Extended, body, .. }
                if name == "geometry"
                    && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, body, .. } if name == "area" && !body.is_empty()))
                    && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, .. } if name == "color"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_extended_standard_accepts_default_procedure_implementations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_extended_default_pro.fol")
        .expect("Should read extended default-procedure standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse default procedure implementations in extended standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, kind: StandardKind::Extended, body, .. }
                if name == "geometry"
                    && body.iter().any(|stmt| matches!(stmt, AstNode::ProDecl { name, body, .. } if name == "reset" && !body.is_empty()))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_extended_standard_accepts_default_logical_implementations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_extended_default_log.fol")
        .expect("Should read extended default-logical standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse default logical implementations in extended standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, kind: StandardKind::Extended, body, .. }
                if name == "geometry"
                    && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, body, .. } if name == "valid" && !body.is_empty()))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_standard_declaration_visibility_options_parse() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_visibility_options.fol")
        .expect("Should read visibility-option standard test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse standard visibility options");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, options, .. }
                if name == "geometry" && options == &vec![DeclOption::Export]
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_standard_declaration_rejects_conflicting_visibility_options() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_conflicting_options.fol")
        .expect("Should read conflicting standard option test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject conflicting standard options");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Conflicting standard visibility options"),
        "Conflicting std options should report a targeted error, got: {}",
        parse_error
    );
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
fn test_protocol_standard_accepts_kind_options() {
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
                    AstNode::StdDecl {
                        name,
                        kind: StandardKind::Protocol,
                        kind_options,
                        ..
                    }
                    if name == "geometry" && kind_options == &vec![DeclOption::Export]
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
                    AstNode::StdDecl {
                        name,
                        kind: StandardKind::Blueprint,
                        kind_options,
                        ..
                    }
                    if name == "geometry" && kind_options == &vec![DeclOption::Hidden]
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_blueprint_standard_accepts_const_fields() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_blueprint_const_field.fol")
        .expect("Should read const blueprint standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse const fields in blueprint standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Blueprint, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, options, .. } if name == "theme" && options.contains(&VarOption::Immutable)))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_blueprint_standard_accepts_field_alternatives() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_blueprint_field_alternatives.fol")
            .expect("Should read blueprint field alternative fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse binding alternatives in blueprint standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Blueprint, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, options, .. } if name == "theme" && options.contains(&VarOption::Export)))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, options, .. } if name == "status" && options.contains(&VarOption::Hidden)))
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
                    AstNode::StdDecl {
                        name,
                        kind: StandardKind::Extended,
                        kind_options,
                        ..
                    }
                    if name == "geometry" && kind_options == &vec![DeclOption::Normal]
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_extended_standard_accepts_const_fields() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_extended_const_field.fol")
        .expect("Should read const extended standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse const members in extended standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Extended, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, options, .. } if name == "theme" && options.contains(&VarOption::Immutable)))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, .. } if name == "area"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_extended_standard_accepts_field_alternatives() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_extended_field_alternatives.fol")
            .expect("Should read extended field alternative fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse binding alternatives in extended standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::StdDecl { name, kind: StandardKind::Extended, body, .. }
                    if name == "geometry"
                        && body.iter().any(|stmt| matches!(stmt, AstNode::FunDecl { name, .. } if name == "area"))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, options, .. } if name == "theme" && options.contains(&VarOption::Export)))
                        && body.iter().any(|stmt| matches!(stmt, AstNode::VarDecl { name, options, .. } if name == "status" && options.contains(&VarOption::Hidden)))
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
    assert_eq!(
        parse_error.column(),
        9,
        "Duplicate protocol signature should point to the duplicate routine name"
    );
}

#[test]
fn test_blueprint_standard_rejects_duplicate_fields() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_blueprint_duplicate_field.fol")
            .expect("Should read duplicate blueprint standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate blueprint fields");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Duplicate standard member 'color'"),
        "Expected duplicate blueprint member error, got: {}",
        parse_error
    );
    assert_eq!(
        parse_error.column(),
        9,
        "Duplicate blueprint field should point to the duplicate field name"
    );
}

#[test]
fn test_extended_standard_rejects_duplicate_members() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_extended_duplicate_member.fol")
            .expect("Should read duplicate extended standard fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate extended members");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Duplicate standard member 'area#0'"),
        "Expected duplicate extended member error, got: {}",
        parse_error
    );
    assert_eq!(
        parse_error.column(),
        9,
        "Duplicate extended member should point to the duplicate member name"
    );
}

#[test]
fn test_standard_rejects_unknown_declaration_options() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_unknown_options.fol")
            .expect("Should read malformed std[] fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject unknown std declaration options");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Unknown standard option"),
        "Expected std option error, got: {}",
        parse_error
    );
}

#[test]
fn test_standard_rejects_unknown_kind_options() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_std_unknown_kind_options.fol")
            .expect("Should read malformed standard kind fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject unknown standard kind options");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Unknown protocol standard kind option"),
        "Expected standard kind option error, got: {}",
        parse_error
    );
}

#[test]
fn test_extended_standards_accept_grouped_type_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_std_grouped_types.fol")
        .expect("Should read grouped standard-type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should flatten grouped type members inside standards");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::StdDecl { name, body, .. }
                if name == "Shapes"
                    && body.iter().any(|member| matches!(
                        member,
                        AstNode::TypeDecl { name, type_def: TypeDefinition::Record { .. }, .. }
                        if name == "Inner"
                    ))
                    && body.iter().any(|member| matches!(
                        member,
                        AstNode::TypeDecl {
                            name,
                            type_def: TypeDefinition::Alias { target: FolType::Named { name: target } },
                            ..
                        } if name == "Label" && target == "str"
                    ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}
