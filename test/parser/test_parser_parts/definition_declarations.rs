use super::*;

#[test]
fn test_top_level_def_module_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_module.fol")
        .expect("Should read module definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse module definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::DefDecl {
                            name,
                            def_type: FolType::Module { name: module_name },
                            body,
                        }
                        if name == "argo"
                            && module_name == "init"
                            && body.iter().any(|stmt| matches!(
                                stmt,
                                AstNode::VarDecl { name, .. } if name == "name"
                            ))
                            && body.iter().any(|stmt| matches!(
                                stmt,
                                AstNode::FunDecl { name, .. } if name == "add"
                            ))
                    )
                }),
                "Program should include parsed module definition"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_top_level_def_block_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_block.fol")
        .expect("Should read block definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse block definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::DefDecl {
                            name,
                            def_type: FolType::Block { name: block_name },
                            body,
                        }
                        if name == "marker"
                            && block_name.is_empty()
                            && body.iter().any(|stmt| matches!(
                                stmt,
                                AstNode::VarDecl { name, .. } if name == "count"
                            ))
                    )
                }),
                "Program should include parsed block definition"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_top_level_def_module_parsing_with_trailing_semicolon() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_module_with_semi.fol")
        .expect("Should read semicolon-terminated module definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse semicolon-terminated module definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::DefDecl {
                            name,
                            def_type: FolType::Module { name: module_name },
                            body,
                        }
                        if name == "argo"
                            && module_name == "init"
                            && body.iter().any(|stmt| matches!(
                                stmt,
                                AstNode::VarDecl { name, .. } if name == "name"
                            ))
                    )
                }),
                "Program should include parsed semicolon-terminated module definition"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_def_with_quoted_name_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_quoted_name.fol")
        .expect("Should read quoted-name definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse quoted definition names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::DefDecl {
                            name,
                            def_type: FolType::Block { name: block_name },
                            body,
                        }
                        if name == "startup block"
                            && block_name.is_empty()
                            && body.iter().any(|stmt| matches!(
                                stmt,
                                AstNode::VarDecl { name, .. } if name == "ready"
                            ))
                    )
                }),
                "Program should include parsed quoted-name block definition"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_def_block_marker_without_body_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_block_marker.fol")
        .expect("Should read block-marker definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse block marker definitions without bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::DefDecl {
                            name,
                            def_type: FolType::Block { name: block_name },
                            body,
                        }
                        if name == "mark" && block_name.is_empty() && body.is_empty()
                    )
                }),
                "Program should include parsed empty-body block marker definition"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_def_block_marker_without_body_parsing_with_trailing_semicolon() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_def_block_marker_with_semi.fol")
            .expect("Should read block-marker-with-semicolon test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse block marker definitions with trailing semicolons");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::DefDecl {
                            name,
                            def_type: FolType::Block { name: block_name },
                            body,
                        }
                        if name == "mark" && block_name.is_empty() && body.is_empty()
                    )
                }),
                "Program should include parsed semicolon-terminated empty-body block marker definition"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_def_invalid_type_reports_parse_error() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_invalid_type.fol")
        .expect("Should read malformed def fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject definitions with non block/module types");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains(
            "Definition declarations currently support only mod[...] or blk[...] types"
        ),
        "Invalid definition type should report the supported def target kinds, got: {}",
        first_message
    );
}

#[test]
fn test_def_module_without_body_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_def_module_missing_body.fol")
            .expect("Should read module-missing-body def fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject module definitions without bodies");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected '=' before definition body"),
        "Module definitions without bodies should keep requiring '=', got: {}",
        first_message
    );
}
