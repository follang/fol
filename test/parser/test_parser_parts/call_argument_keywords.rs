use super::*;

#[test]
fn test_function_calls_support_keyword_arguments() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_keyword_call_args.fol")
        .expect("Should read keyword call-argument fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept keyword call arguments");

    let has_keyword_call = match ast {
        AstNode::Program { declarations } => declarations.iter().any(|node| {
            matches!(
                node,
                AstNode::Assignment { value, .. }
                if matches!(
                    value.as_ref(),
                    AstNode::FunctionCall { name, args }
                    if name == "calc"
                        && args.len() == 3
                        && matches!(&args[0], AstNode::NamedArgument { name, .. } if name == "el3")
                        && matches!(&args[1], AstNode::NamedArgument { name, .. } if name == "el2")
                        && matches!(&args[2], AstNode::NamedArgument { name, .. } if name == "el1")
                )
            )
        }),
        _ => panic!("Expected program node"),
    };

    assert!(
        has_keyword_call,
        "Function call should preserve keyword arguments structurally"
    );
}
