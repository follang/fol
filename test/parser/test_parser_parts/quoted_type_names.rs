use super::*;

#[test]
fn test_type_declaration_accepts_quoted_names() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_quoted_name.fol")
        .expect("Should read quoted type-name fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted type names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Alias { target: FolType::Named { name: target } },
                        ..
                    } if name == "Result" && target == "pkg::Value"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
