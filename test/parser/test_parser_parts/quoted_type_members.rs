use super::*;

#[test]
fn test_record_type_definition_accepts_quoted_fields() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_record_quoted_fields.fol")
        .expect("Should read quoted record-field fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted record field names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Record { fields },
                        ..
                    } if name == "Data" && fields.contains_key("id") && fields.contains_key("label")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_entry_type_definition_accepts_quoted_variants() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_entry_quoted_variants.fol")
            .expect("Should read quoted entry-variant fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted entry variant names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Entry { variants },
                        ..
                    } if name == "Result" && variants.contains_key("ok") && variants.contains_key("err")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_single_quoted_type_members_parse() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_single_quoted_type_members.fol")
            .expect("Should read single-quoted type-member fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept single-quoted type members");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Record { fields },
                        ..
                    } if name == "Data" && fields.contains_key("id") && fields.contains_key("label")
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Entry { variants },
                        ..
                    } if name == "Result" && variants.contains_key("ok") && variants.contains_key("err")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
