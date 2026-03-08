use super::*;

#[test]
fn test_slice_assignment_target_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_slice_assignment_target.fol")
            .expect("Should read slice assignment target fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse slice assignment targets");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::Assignment { target, .. }
                        if matches!(
                            target.as_ref(),
                            AstNode::SliceAccess { start, end, reverse, .. }
                            if !reverse && start.is_some() && end.is_some()
                        )
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_reverse_slice_assignment_target_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_reverse_slice_assignment_target.fol")
            .expect("Should read reverse slice assignment target fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse reverse slice assignment targets");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { body, .. }
                    if body.iter().any(|stmt| matches!(
                        stmt,
                        AstNode::Assignment { target, .. }
                        if matches!(
                            target.as_ref(),
                            AstNode::SliceAccess { start, end, reverse, .. }
                            if *reverse && start.is_none() && end.is_some()
                        )
                    ))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
