use super::*;

#[test]
fn test_quoted_type_references_parse_in_receiver_and_error_positions() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_quoted_receiver_and_error_types.fol")
            .expect("Should read quoted receiver/error-type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted type references in receiver and error positions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl {
                        name,
                        return_type: Some(FolType::Named { name: ret }),
                        error_type: Some(FolType::Named { name: err }),
                        ..
                    } if name == "run" && ret == "Output" && err == "errs::Failure"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_single_quoted_type_references_parse_in_receiver_and_error_positions() {
    let mut file_stream = FileStream::from_file(
        "test/parser/simple_single_quoted_receiver_and_error_types.fol",
    )
    .expect("Should read single-quoted receiver/error-type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept single-quoted type refs in receiver and error positions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl {
                        name,
                        return_type: Some(FolType::Named { name: ret }),
                        error_type: Some(FolType::Named { name: err }),
                        ..
                    } if name == "run" && ret == "Output" && err == "errs::Failure"
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
