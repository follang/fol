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

#[test]
fn test_record_type_retains_multiple_contract_headers() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_record_multiple_contracts.fol")
            .expect("Should read multiple record-contract fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should retain multiple record contracts");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl { name, contracts, type_def: TypeDefinition::Record { .. }, .. }
                if name == "Shape"
                    && matches!(contracts.as_slice(),
                        [FolType::Named { name: first }, FolType::Named { name: second }]
                        if first == "geo" && second == "draw")
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_entry_type_retains_contract_headers() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_entry_contracts.fol")
        .expect("Should read entry contract fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should retain entry contracts from unconstrained type headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl { name, contracts, type_def: TypeDefinition::Entry { .. }, .. }
                if name == "Status"
                    && matches!(contracts.as_slice(), [FolType::Named { name }] if name == "display")
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_record_type_contracts_follow_generic_separator_rules() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_record_contracts_semicolon.fol")
            .expect("Should read semicolon-separated record contract fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should retain semicolon-separated record contracts");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl { name, contracts, type_def: TypeDefinition::Record { .. }, .. }
                if name == "Shape"
                    && matches!(contracts.as_slice(),
                        [FolType::Named { name: first }, FolType::Named { name: second }]
                        if first == "geo" && second == "draw")
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_record_type_contracts_accept_keyword_and_quoted_names() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_named_contracts.fol")
        .expect("Should read named contract fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should retain keyword and quoted contract names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl { name, contracts, type_def: TypeDefinition::Record { .. }, .. }
                if name == "Box"
                    && matches!(contracts.as_slice(),
                        [FolType::Named { name: first }, FolType::Named { name: second }]
                        if first == "get" && second == "item")
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_constrained_type_generics_do_not_become_contracts() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_constrained_record_generics.fol")
            .expect("Should read constrained record-generic fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should keep constrained type generics separate from contracts");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl { name, generics, contracts, type_def: TypeDefinition::Record { .. }, .. }
                if name == "Box" && generics.len() == 1 && generics[0].name == "T" && contracts.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alias_headers_do_not_synthesize_contracts() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_alias_unconstrained_header.fol")
            .expect("Should read alias unconstrained-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should not synthesize contracts for alias type declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl { name, generics, contracts, type_def: TypeDefinition::Alias { .. }, .. }
                if name == "Alias" && generics.len() == 1 && generics[0].name == "Box" && contracts.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}
