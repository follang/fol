use super::*;

#[test]
fn test_routine_declarations_accept_keyword_named_parameters() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_keyword_named_params.fol")
        .expect("Should read keyword-named parameter fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept keyword-named routine parameters");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { params, .. }
                    if params.len() == 2 && params[0].name == "get" && params[1].name == "std"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_function_type_references_accept_keyword_named_parameters() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_function_type_keyword_params.fol")
            .expect("Should read keyword-named function-type parameter fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept keyword-named parameters in function types");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { params, .. }
                    if matches!(
                        params.as_slice(),
                        [Parameter {
                            param_type: FolType::Function { params, .. },
                            ..
                        }] if matches!(params.as_slice(), [FolType::Int { .. }, FolType::Named { name }] if name == "str")
                    )
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
