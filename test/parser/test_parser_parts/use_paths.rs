use super::*;

#[test]
fn test_use_declaration_accepts_direct_quoted_paths() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_use_direct_quoted_path.fol")
            .expect("Should read direct quoted use-path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept direct quoted use paths");

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
fn test_use_declaration_accepts_multiple_direct_quoted_paths() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_use_multi_direct_quoted_paths.fol")
            .expect("Should read multi direct quoted use-path fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept multiple direct quoted use paths");

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
