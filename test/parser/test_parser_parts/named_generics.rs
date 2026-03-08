use super::*;

#[test]
fn test_routine_generic_headers_accept_keyword_and_quoted_names() {
    let mut file_stream = FileStream::from_file("test/parser/simple_routine_named_generics.fol")
        .expect("Should read named routine generics fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept keyword and quoted routine generic names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { generics, .. }
                    if generics.len() == 2 && generics[0].name == "get" && generics[1].name == "item"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_type_generic_headers_accept_keyword_and_quoted_names() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_named_generics.fol")
        .expect("Should read named type generics fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept keyword and quoted type generic names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl { generics, .. }
                    if generics.len() == 2 && generics[0].name == "get" && generics[1].name == "item"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_implementation_generic_headers_accept_keyword_and_quoted_names() {
    let mut file_stream = FileStream::from_file("test/parser/simple_imp_named_generics.fol")
        .expect("Should read named implementation generics fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept keyword and quoted implementation generic names");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ImpDecl { generics, .. }
                    if generics.len() == 2 && generics[0].name == "get" && generics[1].name == "item"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
