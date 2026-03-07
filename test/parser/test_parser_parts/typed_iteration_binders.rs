use super::*;

#[test]
fn test_top_level_each_iteration_supports_typed_silent_binder() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_each_typed_silent_binder.fol")
            .expect("Should read top-level typed silent each-loop test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse top-level each with typed silent binder");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::Loop { condition, body }
                    if matches!(
                        condition.as_ref(),
                        fol_parser::ast::LoopCondition::Iteration {
                            var,
                            type_hint: Some(FolType::Named { name }),
                            condition: Some(_),
                            ..
                        } if var == "_" && name == "str"
                    ) && body.iter().any(|stmt| matches!(stmt, AstNode::Yield { .. }))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_top_level_loop_iteration_supports_typed_silent_binder() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_loop_typed_silent_binder.fol")
            .expect("Should read top-level typed silent loop test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse top-level loop with typed silent binder");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::Loop { condition, body }
                    if matches!(
                        condition.as_ref(),
                        fol_parser::ast::LoopCondition::Iteration {
                            var,
                            type_hint: Some(FolType::Named { name }),
                            condition: Some(_),
                            ..
                        } if var == "_" && name == "str"
                    ) && body.iter().any(|stmt| matches!(stmt, AstNode::Yield { .. }))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_top_level_for_iteration_supports_typed_silent_binder() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_for_typed_silent_binder.fol")
            .expect("Should read top-level typed silent for-loop test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse top-level for with typed silent binder");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::Loop { condition, body }
                    if matches!(
                        condition.as_ref(),
                        fol_parser::ast::LoopCondition::Iteration {
                            var,
                            type_hint: Some(FolType::Named { name }),
                            condition: Some(_),
                            ..
                        } if var == "_" && name == "str"
                    ) && body.iter().any(|stmt| matches!(stmt, AstNode::Yield { .. }))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
