use super::*;

#[test]
fn test_routine_variadic_parameter_lowers_to_sequence_type() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_variadic_param.fol")
            .expect("Should read variadic parameter fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept variadic parameters");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { name, params, .. }
                    if name == "calc"
                        && params.len() == 2
                        && matches!(
                            params[1].param_type,
                            FolType::Sequence { ref element_type }
                                if matches!(element_type.as_ref(), FolType::Int { size: None, signed: true })
                        )
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
