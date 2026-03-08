use super::*;
use fol_parser::ast::WhenCase;

#[test]
fn test_when_flow_bodies_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_when_flow_body.fol")
        .expect("Should read when flow body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse when flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::When { cases, .. }
                        if matches!(cases.as_slice(),
                            [
                                WhenCase::Is { body, .. },
                                WhenCase::Has { body: second_body, .. }
                            ]
                            if matches!(body.as_slice(), [AstNode::Literal(Literal::Integer(0))])
                                && matches!(second_body.as_slice(), [AstNode::Identifier { name }] if name == "value")
                        )
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_if_flow_bodies_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_if_flow_body.fol")
        .expect("Should read if flow body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse if/else flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::When { cases, default, .. }
                        if matches!(cases.as_slice(),
                            [WhenCase::Case { body, .. }]
                            if matches!(body.as_slice(), [AstNode::Identifier { name }] if name == "value")
                        )
                        && matches!(default, Some(default_body)
                            if matches!(default_body.as_slice(), [AstNode::Literal(Literal::Integer(0))]))
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
