use super::*;

fn parse_package_errors(path: &str) -> Vec<ParseError> {
    let mut file_stream =
        FileStream::from_file(path).expect("Should read parser package transition fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse_package(&mut lexer)
        .expect_err("Book-aligned package parsing should reject script-style file roots");

    errors
        .iter()
        .map(|error| {
            error
                .as_ref()
                .as_any()
                .downcast_ref::<ParseError>()
                .cloned()
                .expect("Package transition errors should be ParseError values")
        })
        .collect()
}

#[test]
fn test_parse_is_a_compatibility_shim_over_parse_script_package() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_top_level_keyword_call_and_assignment.fol")
            .expect("Should read mixed-root transition fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Legacy parse() should still accept mixed root forms");

    let parsed = parse_script_package_from_file(
        "test/parser/simple_top_level_keyword_call_and_assignment.fol",
    );

    match ast {
        AstNode::Program { declarations } => {
            let flattened = parsed.source_units[0]
                .items
                .iter()
                .map(|item| item.node.clone())
                .collect::<Vec<_>>();

            assert_eq!(
                declarations, flattened,
                "Legacy parse() should flatten the mixed-surface script package result"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_package_rejects_script_roots_while_parse_accepts_them() {
    let mut file_stream = FileStream::from_file("test/parser/simple_call_top_level.fol")
        .expect("Should read top-level call transition fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Legacy parse() should continue to accept top-level script roots");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::FunctionCall { name, args, .. } if name == "run" && args.len() == 2)
            }));
        }
        _ => panic!("Expected program node"),
    }

    let errors = parse_package_errors("test/parser/simple_call_top_level.fol");

    assert_eq!(
        errors.len(),
        1,
        "Book-aligned parse_package() should reject the same file root that parse() still accepts"
    );
    assert!(
        errors[0]
            .to_string()
            .contains("Executable calls are not allowed at file root"),
        "Expected explicit file-root call rejection, got: {}",
        errors[0]
    );
}
