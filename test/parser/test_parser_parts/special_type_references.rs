use super::*;

#[test]
fn test_never_type_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_never_type_refs.fol")
        .expect("Should read never type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower nev[] references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Never
                    },
                    ..
                } if name == "Impossible"
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::Never),
                    ..
                }
                if name == "stop"
                    && matches!(params.as_slice(),
                        [Parameter { param_type: FolType::Never, .. }]
                    )
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::VarDecl {
                    name,
                    type_hint: Some(FolType::Never),
                    ..
                } if name == "doom"
            )));
        }
        _ => panic!("Expected program node"),
    }
}
