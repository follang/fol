use super::*;

#[test]
fn test_function_calls_support_unpack_arguments() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_call_unpack_args.fol")
        .expect("Should read unpack call-argument fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept call-site unpack arguments");

    let has_unpack_call = match ast {
        AstNode::Program { declarations } => declarations.iter().any(|node| {
            matches!(
                node,
                AstNode::Assignment { value, .. }
                if matches!(
                    value.as_ref(),
                    AstNode::FunctionCall { name, args }
                    if name == "calc"
                        && args.len() == 2
                        && matches!(&args[0], AstNode::Literal(Literal::Boolean(true)))
                        && matches!(&args[1], AstNode::Unpack { .. })
                )
            )
        }),
        _ => panic!("Expected program node"),
    };

    assert!(
        has_unpack_call,
        "Function call should preserve call-site unpack arguments structurally"
    );
}
