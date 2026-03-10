use super::*;

#[test]
fn test_book_keyword_call_example_parses() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_book_keyword_call_example.fol")
            .expect("Should read book keyword call example");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept the book keyword call example");

    let has_keyword_call = match ast {
        AstNode::Program { declarations } => declarations.iter().any(|node| {
            matches!(
                node,
                AstNode::ProDecl { body, .. }
                if body.iter().any(|stmt| matches!(
                    stmt,
                    AstNode::FunctionCall { name, args }
                    if name == "calc"
                        && args.len() == 3
                        && matches!(&args[0], AstNode::NamedArgument { name, .. } if name == "el3")
                        && matches!(&args[1], AstNode::NamedArgument { name, .. } if name == "el2")
                        && matches!(&args[2], AstNode::NamedArgument { name, .. } if name == "el1")
                ))
            )
        }),
        _ => panic!("Expected program node"),
    };

    assert!(
        has_keyword_call,
        "Book keyword call example should preserve named call arguments"
    );
}

#[test]
fn test_book_mixed_keyword_call_example_parses() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_book_mixed_keyword_call_example.fol")
            .expect("Should read book mixed keyword call example");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept the book mixed keyword call example");

    let has_mixed_keyword_call = match ast {
        AstNode::Program { declarations } => declarations.iter().any(|node| {
            matches!(
                node,
                AstNode::ProDecl { body, .. }
                if body.iter().any(|stmt| matches!(
                    stmt,
                    AstNode::FunctionCall { name, args }
                    if name == "calc"
                        && args.len() == 5
                        && !matches!(&args[0], AstNode::NamedArgument { .. })
                        && !matches!(&args[1], AstNode::NamedArgument { .. })
                        && matches!(&args[2], AstNode::NamedArgument { name, .. } if name == "el5")
                        && matches!(&args[3], AstNode::NamedArgument { name, .. } if name == "el4")
                        && matches!(&args[4], AstNode::NamedArgument { name, .. } if name == "el3")
                ))
            )
        }),
        _ => panic!("Expected program node"),
    };

    assert!(
        has_mixed_keyword_call,
        "Book mixed keyword call example should preserve positional and named call arguments"
    );
}

#[test]
fn test_book_higher_order_function_parameter_example_parses() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_book_higher_order_param_example.fol")
            .expect("Should read book higher-order parameter example");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept the book higher-order parameter example");

    let has_higher_order_param = match ast {
        AstNode::Program { declarations } => declarations.iter().any(|node| {
            matches!(
                node,
                AstNode::FunDecl { name, params, .. }
                if name == "add1"
                    && params.len() == 1
                    && matches!(params[0].param_type, FolType::Function { .. })
            )
        }),
        _ => panic!("Expected program node"),
    };

    assert!(
        has_higher_order_param,
        "Book higher-order parameter example should keep the function-type parameter"
    );
}
