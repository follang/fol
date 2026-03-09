use super::*;

#[test]
fn test_alternative_function_header_without_params() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_alt_header_no_params.fol")
        .expect("Should read alternative function-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative function headers without params");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::Int { .. }),
                    body,
                    ..
                }
                if name == "add" && params.is_empty() && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_procedure_header_without_params() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pro_alt_header_no_params.fol")
        .expect("Should read alternative procedure-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative procedure headers without params");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl {
                    name,
                    params,
                    return_type: Some(FolType::Int { .. }),
                    body,
                    ..
                }
                if name == "main" && params.is_empty() && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_logical_header_without_params() {
    let mut file_stream = FileStream::from_file("test/parser/simple_log_alt_header_no_params.fol")
        .expect("Should read alternative logical-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative logical headers without params");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::Bool),
                    body,
                    ..
                }
                if name == "ready" && params.is_empty() && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}
