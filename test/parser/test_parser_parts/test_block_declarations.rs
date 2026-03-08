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
