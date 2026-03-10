use super::*;

#[test]
fn test_channel_endpoint_access_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_channel_endpoint_access.fol")
            .expect("Should read channel endpoint access fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse channel endpoint access");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { value: Some(value), .. }
                        if matches!(value.as_ref(), AstNode::ChannelAccess { endpoint, .. }
                            if matches!(endpoint, fol_parser::ast::ChannelEndpoint::Rx))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_channel_endpoint_access_can_chain_index() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_channel_endpoint_index.fol")
            .expect("Should read chained channel endpoint access fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse channel endpoint access with chained index");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { value: Some(value), .. }
                        if matches!(value.as_ref(), AstNode::IndexAccess { container, .. }
                            if matches!(container.as_ref(), AstNode::ChannelAccess { endpoint, .. }
                                if matches!(endpoint, fol_parser::ast::ChannelEndpoint::Rx)))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
