use super::*;

#[test]
fn test_macro_definition_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_macro.fol")
        .expect("Should read macro definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse macro definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::DefDecl {
                    name,
                    params,
                    def_type: FolType::Named { name: def_kind },
                    body,
                    ..
                }
                if name == "$"
                    && def_kind == "mac"
                    && params.len() == 1
                    && params[0].name == "a"
                    && matches!(params[0].param_type, FolType::Any)
                    && body.len() == 1
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_definition_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_def_alternative.fol")
        .expect("Should read alternative definition test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative definitions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::DefDecl {
                    name,
                    params,
                    def_type: FolType::Named { name: def_kind },
                    body,
                    ..
                }
                if name == "+var"
                    && params.is_empty()
                    && def_kind == "alt"
                    && body.len() == 1
            )));
        }
        _ => panic!("Expected program node"),
    }
}
