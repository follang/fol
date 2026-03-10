use super::*;

#[test]
fn test_book_generic_object_type_example() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_book_generic_object_type.fol")
            .expect("Should read generic object type example");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept the book generic object type example");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    generics,
                    contracts,
                    type_def: TypeDefinition::Record { fields, members, .. },
                    ..
                }
                if name == "container"
                    && generics.len() == 2
                    && contracts.is_empty()
                    && matches!(fields.get("items"), Some(FolType::Sequence { .. }))
                    && members.iter().any(|member| matches!(member, AstNode::FunDecl { name, .. } if name == "getsize"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}
