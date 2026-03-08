use super::*;

#[test]
fn test_root_quoted_type_references_parse_across_declarations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_root_quoted_type_refs.fol")
        .expect("Should read root quoted type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted root type references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::AliasDecl { target: FolType::Named { name }, .. } if name == "Target")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        type_def: TypeDefinition::Alias { target: FolType::Named { name } },
                        ..
                    } if name == "Payload"
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { params, return_type: Some(FolType::Named { name }), .. }
                    if name == "Output"
                        && matches!(params.as_slice(), [Parameter { param_type: FolType::Named { name }, .. }] if name == "Input")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
