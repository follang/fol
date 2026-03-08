use super::*;

#[test]
fn test_test_type_reference_lowering() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_test_block_type.fol")
        .expect("Should read tst type lowering test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower tst[...] type references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Alias {
                            target: FolType::Test { name: Some(label), access }
                        },
                        ..
                    }
                    if name == "TestBlock" && label == "unit" && access == &vec!["shko".to_string()]
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_named_test_block_definition_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_test_block.fol")
        .expect("Should read named tst def test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse tst block definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::DefDecl {
                        name,
                        def_type: FolType::Test { name: Some(label), access },
                        body,
                        ..
                    }
                    if name == "test1"
                        && label == "sometest"
                        && access == &vec!["shko".to_string()]
                        && body.is_empty()
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_access_only_test_block_definition_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_def_access_only_test_block.fol")
            .expect("Should read access-only tst def test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse access-only tst block definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::DefDecl {
                        name,
                        def_type: FolType::Test { name: None, access },
                        ..
                    }
                    if name == "some unit testing" && access == &vec!["shko".to_string()]
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_test_block_rejects_quoted_access_argument() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_def_bad_test_block_access.fol")
            .expect("Should read malformed tst def test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject quoted tst access arguments");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error
            .to_string()
            .contains("Quoted tst[...] arguments are only allowed for the optional test label"),
        "Expected quoted tst access diagnostic, got: {}",
        parse_error
    );
}
