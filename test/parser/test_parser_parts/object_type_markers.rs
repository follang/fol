use super::*;

#[test]
fn test_object_type_marker_lowers_to_record_definition() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_object_marker.fol")
        .expect("Should read object type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept object type marker");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Record { fields, .. },
                    ..
                } if name == "File"
                    && matches!(fields.get("name"), Some(FolType::Named { name, .. }) if name == "str")
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_object_type_marker_accepts_empty_marker_options() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_object_marker_empty_options.fol")
            .expect("Should read object marker options fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept empty object marker options");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Record { fields, .. },
                    ..
                } if name == "Handle"
                    && matches!(fields.get("active"), Some(FolType::Bool))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_object_type_marker_accepts_empty_body_form() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_object_marker_empty_body.fol")
            .expect("Should read empty object marker fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept empty object marker forms");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Record { fields, members, .. },
                    ..
                } if name == "User" && fields.is_empty() && members.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}
