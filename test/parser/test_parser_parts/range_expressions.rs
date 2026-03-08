use super::*;

#[test]
fn test_triple_dot_ranges_parse_as_non_inclusive() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_triple_dot_ranges.fol")
            .expect("Should read triple-dot range fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept triple-dot ranges");

    match ast {
        AstNode::Program { declarations } => {
            let ranges: Vec<&AstNode> = declarations
                .iter()
                .filter_map(|node| match node {
                    AstNode::Assignment { value, .. } => Some(value.as_ref()),
                    AstNode::Return { value: Some(value) } => Some(value.as_ref()),
                    _ => None,
                })
                .collect();

            assert!(ranges.iter().any(|node| {
                matches!(
                    node,
                    AstNode::Range {
                        start: Some(_),
                        end: None,
                        inclusive: false
                    }
                )
            }));
            assert!(ranges.iter().any(|node| {
                matches!(
                    node,
                    AstNode::Range {
                        start: None,
                        end: Some(_),
                        inclusive: false
                    }
                )
            }));
            assert!(ranges.iter().any(|node| {
                matches!(
                    node,
                    AstNode::Range {
                        start: Some(_),
                        end: Some(_),
                        inclusive: false
                    }
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
