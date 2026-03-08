use super::*;

#[test]
fn test_path_type_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_source_path_types.fol")
        .expect("Should read path source-kind type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower path source-kind types");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::UseDecl {
                    name,
                    path_type: FolType::Path { name: kind_name },
                    ..
                } if name == "math" && kind_name.is_empty()
            )));
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Path { name: kind_name }
                    },
                    ..
                } if name == "PkgPath" && kind_name == "core"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_url_type_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_source_url_types.fol")
        .expect("Should read url source-kind type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower url source-kind types");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::UseDecl {
                    name,
                    path_type: FolType::Url { name: kind_name },
                    ..
                } if name == "remote" && kind_name.is_empty()
            )));
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Url { name: kind_name }
                    },
                    ..
                } if name == "RemoteUrl" && kind_name == "https"
            )));
        }
        _ => panic!("Expected program node"),
    }
}
