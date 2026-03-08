use super::*;

#[test]
fn test_top_level_segment_declaration_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_seg_module.fol")
        .expect("Should read segment declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse top-level segment declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::SegDecl { name, seg_type, body }
                    if name == "core"
                        && matches!(seg_type, FolType::Module { .. })
                        && body.iter().any(|stmt| matches!(stmt, AstNode::DefDecl { name, .. } if name == "helper"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_segment_declaration_parsing_inside_function_bodies() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_seg_module.fol")
        .expect("Should read function-body segment declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse segment declarations inside function bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(stmt, AstNode::SegDecl { name, .. } if name == "local"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_segment_declaration_rejects_non_module_types() {
    let mut file_stream = FileStream::from_file("test/parser/simple_seg_invalid_type.fol")
        .expect("Should read malformed segment declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject segments with non-module types");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Segment declarations require module types"),
        "Malformed segment declaration should report invalid type, got: {}",
        first_message
    );
}

#[test]
fn test_segment_declaration_accepts_keyword_names() {
    let mut file_stream = FileStream::from_file("test/parser/simple_seg_keyword_name.fol")
        .expect("Should read keyword-name segment declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse builtin-token segment names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::SegDecl { name, seg_type, .. }
                    if name == "get" && matches!(seg_type, FolType::Module { .. })
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_segment_declaration_accepts_quoted_names() {
    let mut file_stream = FileStream::from_file("test/parser/simple_seg_quoted_name.fol")
        .expect("Should read quoted-name segment declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse quoted segment names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::SegDecl { name, seg_type, body }
                    if name == "core"
                        && matches!(seg_type, FolType::Module { .. })
                        && body.iter().any(|stmt| matches!(stmt, AstNode::DefDecl { name, .. } if name == "helper"))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_segment_declaration_accepts_empty_option_brackets() {
    let mut file_stream = FileStream::from_file("test/parser/simple_seg_empty_options.fol")
        .expect("Should read empty-options segment declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept empty segment option brackets");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::SegDecl { name, seg_type, body }
                if name == "core" && matches!(seg_type, FolType::Module { .. }) && body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_segment_declaration_rejects_non_empty_option_brackets() {
    let mut file_stream = FileStream::from_file("test/parser/simple_seg_unknown_option.fol")
        .expect("Should read invalid segment option test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject non-empty segment options");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let message = parse_error.to_string();
    assert!(
        message.contains("Segment options currently support only empty brackets"),
        "Non-empty segment option brackets should be rejected, got: {}",
        message
    );
}
