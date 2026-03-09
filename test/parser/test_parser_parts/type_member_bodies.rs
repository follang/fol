use super::*;

#[test]
fn test_record_type_accepts_routine_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_record_methods.fol")
        .expect("Should read record routine-member fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept routine members in record type bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Record { members, .. },
                    ..
                }
                if name == "Computer"
                    && members.iter().any(|member| matches!(member, AstNode::FunDecl { name, .. } if name == "getBrand"))
                    && members.iter().any(|member| matches!(member, AstNode::ProDecl { name, .. } if name == "reset"))
                    && members.iter().any(|member| matches!(member, AstNode::FunDecl { name, .. } if name == "ready"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_entry_type_accepts_routine_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_entry_methods.fol")
        .expect("Should read entry routine-member fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept routine members in entry type bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Entry { members, .. },
                    ..
                }
                if name == "Status"
                    && members.iter().any(|member| matches!(member, AstNode::FunDecl { name, .. } if name == "label"))
                    && members.iter().any(|member| matches!(member, AstNode::ProDecl { name, .. } if name == "reset"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_record_type_accepts_alias_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_record_alias_member.fol")
        .expect("Should read record alias-member fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept alias members in record type bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Record { members, .. },
                    ..
                }
                if name == "Distance"
                    && members.iter().any(|member| matches!(member, AstNode::AliasDecl { name, target: FolType::Named { name: target } } if name == "Unit" && target == "str"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_record_type_accepts_nested_type_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_record_type_member.fol")
        .expect("Should read record type-member fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept nested type members in record type bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Record { members, .. },
                    ..
                }
                if name == "Distance"
                    && members.iter().any(|member| matches!(member, AstNode::TypeDecl { name, type_def: TypeDefinition::Alias { target: FolType::Named { name: target } }, .. } if name == "Unit" && target == "str"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_entry_type_accepts_alias_members() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_entry_alias_member.fol")
        .expect("Should read entry alias-member fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept alias members in entry type bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Entry { members, .. },
                    ..
                }
                if name == "Status"
                    && members.iter().any(|member| matches!(member, AstNode::AliasDecl { name, target: FolType::Named { name: target } } if name == "Label" && target == "str"))
            )));
        }
        _ => panic!("Expected program node"),
    }
}
