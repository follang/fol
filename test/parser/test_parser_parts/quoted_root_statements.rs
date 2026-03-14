use super::*;

#[test]
fn test_top_level_quoted_call_and_assignment_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_top_level_quoted_call_and_assignment.fol")
            .expect("Should read top-level quoted root statement fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted root calls and assignments at top level");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::FunctionCall { name, args, .. } if name == "notify" && args.is_empty())
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::Assignment { target, value }
                    if matches!(target.as_ref(), AstNode::Identifier { name, .. } if name == "slot")
                        && matches!(value.as_ref(), AstNode::Identifier { name, .. } if name == "ready")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_function_body_quoted_call_and_assignment_parsing() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_quoted_call_and_assignment.fol")
            .expect("Should read function-body quoted root statement fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted root calls and assignments in function bodies");

    match ast {
        AstNode::Program { declarations } => {
            let body = declarations
                .iter()
                .find_map(|node| match node {
                    AstNode::FunDecl { body, .. } => Some(body),
                    _ => None,
                })
                .expect("Program should include function body");
            assert!(body.iter().any(|node| {
                matches!(node, AstNode::FunctionCall { name, args, .. } if name == "check" && args.is_empty())
            }));
            assert!(body.iter().any(|node| {
                matches!(
                    node,
                    AstNode::Assignment { target, value }
                    if matches!(target.as_ref(), AstNode::Identifier { name, .. } if name == "state")
                        && matches!(value.as_ref(), AstNode::Identifier { name, .. } if name == "ready")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
