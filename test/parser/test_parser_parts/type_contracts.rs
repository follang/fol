use super::*;

#[test]
fn test_record_type_retains_standard_contract_headers() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_generics_unconstrained.fol")
            .expect("Should read unconstrained type-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should retain record contracts from unconstrained type headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl { name, contracts, type_def: TypeDefinition::Record { .. }, .. }
                if name == "Rect"
                    && matches!(contracts.as_slice(), [FolType::Named { name }] if name == "geo")
            )));
        }
        _ => panic!("Expected program node"),
    }
}
