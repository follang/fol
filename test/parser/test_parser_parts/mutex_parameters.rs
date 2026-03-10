use super::*;

#[test]
fn test_mutex_parameter_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_mutex_params.fol")
        .expect("Should read mutex parameter fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse mutex parameters");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { params, .. }
                        if params.len() == 2
                            && params[1].name == "meshes"
                            && params[1].is_mutex
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
