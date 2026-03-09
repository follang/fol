use super::*;

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

#[test]
fn test_loc_type_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_source_loc_types.fol")
        .expect("Should read loc source-kind type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower loc source-kind types");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::UseDecl {
                    name,
                    path_type: FolType::Location { name: kind_name },
                    ..
                } if name == "local" && kind_name.is_empty()
            )));
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Location { name: kind_name }
                    },
                    ..
                } if name == "LocalPath" && kind_name == "disk"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_std_type_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_source_std_types.fol")
        .expect("Should read std source-kind type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower std source-kind types");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::UseDecl {
                    name,
                    path_type: FolType::Standard { name: kind_name },
                    ..
                } if name == "fmt" && kind_name.is_empty()
            )));
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Standard { name: kind_name }
                    },
                    ..
                } if name == "StdPkg" && kind_name == "pkg"
            )));
        }
        _ => panic!("Expected program node"),
    }
}
