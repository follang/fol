use super::*;

#[test]
fn test_multi_pattern_access_expression_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_pattern_access_expr.fol")
        .expect("Should read pattern access fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse multi-pattern access expressions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::Return { value: Some(value) }
                        if matches!(value.as_ref(), AstNode::PatternAccess { patterns, .. } if patterns.len() == 2)
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
