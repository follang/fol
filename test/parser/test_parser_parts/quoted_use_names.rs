use super::*;

#[test]
fn test_use_declaration_accepts_quoted_name() {
    let mut file_stream = FileStream::from_file("test/parser/simple_use_quoted_name.fol")
        .expect("Should read quoted-name use declaration fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted use names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::UseDecl { name, path, path_type, .. }
                    if name == "warn"
                        && path == "std/warn"
                        && matches!(path_type, FolType::Module { .. })
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_accepts_multiple_quoted_names() {
    let mut file_stream = FileStream::from_file("test/parser/simple_use_multi_quoted_names.fol")
        .expect("Should read multi quoted-name use declaration fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept multiple quoted use names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "warn" && path == "std/warn")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "trace" && path == "std/trace")
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_normalizes_single_quoted_paths() {
    let mut file_stream = FileStream::from_file("test/parser/simple_use_single_quoted_path.fol")
        .expect("Should read single-quoted use path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should normalize single-quoted use paths");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "warn" && path == "std/warn")
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_use_declaration_accepts_multiple_single_quoted_names() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_use_multi_single_quoted_names.fol")
            .expect("Should read multi single-quoted use declaration fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept multiple single-quoted use names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "warn" && path == "std/warn")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path, .. } if name == "trace" && path == "std/trace")
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_quoted_type_references_parse_in_use_declarations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_quoted_use_type_refs.fol")
        .expect("Should read quoted use-type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted type references in use declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path_type: FolType::Named { name: hint }, .. } if name == "warn" && hint == "Module")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::UseDecl { name, path_type: FolType::Named { name: hint }, .. } if name == "trace" && hint == "pkg::Module")
            }));
        }
        _ => panic!("Expected program node"),
    }
}
