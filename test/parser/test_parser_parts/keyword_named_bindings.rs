use super::*;

#[test]
fn test_keyword_named_bindings_parse() {
    let mut file_stream = FileStream::from_file("test/parser/simple_binding_keyword_names.fol")
        .expect("Should read keyword-named binding test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept builtin-token binding names");

    match ast {
        AstNode::Program { declarations } => {
            let names: Vec<_> = declarations
                .iter()
                .filter_map(|node| {
                    if let AstNode::VarDecl { name, .. } = node {
                        Some(name.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            assert_eq!(names, vec!["get", "log", "seg"]);
        }
        _ => panic!("Expected program node"),
    }
}
