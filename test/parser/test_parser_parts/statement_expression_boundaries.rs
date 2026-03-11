use super::*;

fn parse_program_declarations(path: &str) -> Vec<AstNode> {
    let mut file_stream = FileStream::from_file(path).expect("Should read parser boundary fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    match parser
        .parse(&mut lexer)
        .expect("Parser should accept boundary fixture")
    {
        AstNode::Program { declarations } => declarations,
        other => panic!("Expected program node, got {other:?}"),
    }
}

#[test]
fn test_top_level_grouped_callee_lowers_as_invoke_not_function_call() {
    let declarations = parse_program_declarations("test/parser/simple_invoke_top_level.fol");

    assert_eq!(
        declarations.len(),
        1,
        "Top-level grouped callee should lower to a single root node"
    );

    assert!(matches!(
        declarations.first(),
        Some(AstNode::Invoke { callee, args })
            if args.len() == 1
                && matches!(callee.as_ref(), AstNode::FunctionCall { name, args } if name == "factory" && args.is_empty())
                && matches!(&args[0], AstNode::Identifier { name } if name == "value")
    ));
}
