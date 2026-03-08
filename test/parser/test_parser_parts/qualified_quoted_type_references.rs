use super::*;

#[test]
fn test_qualified_quoted_type_references_parse() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_qualified_quoted_type_refs.fol")
            .expect("Should read qualified quoted type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted qualified type references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::AliasDecl { target: FolType::Named { name }, .. } if name == "core::Target")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { params, return_type: Some(FolType::Named { name }), .. }
                    if name == "errs::Output"
                        && matches!(params.as_slice(), [Parameter { param_type: FolType::Named { name }, .. }] if name == "pkg::Input")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
