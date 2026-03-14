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
fn test_alternative_function_header_without_params_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_alt_header_no_params_flow.fol")
            .expect("Should read alternative function-header flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied alternative function headers without params");

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
fn test_alternative_procedure_header_without_params_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_alt_header_no_params_flow.fol")
            .expect("Should read alternative procedure-header flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied alternative procedure headers without params");

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
                AstNode::LogDecl {
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

#[test]
fn test_alternative_logical_header_without_params_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_log_alt_header_no_params_flow.fol")
            .expect("Should read alternative logical-header flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied alternative logical headers without params");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::LogDecl {
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

#[test]
fn test_alternative_function_header_with_params() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_alt_header_params.fol")
        .expect("Should read alternative parameterized function-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative function headers with params");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::Int { .. }),
                    ..
                }
                if name == "add"
                    && params.len() == 2
                    && params[0].name == "a"
                    && params[1].name == "b"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_function_header_with_params_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_alt_header_params_flow.fol")
            .expect("Should read alternative parameterized function-header flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied alternative function headers with params");

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
                if name == "add"
                    && params.len() == 2
                    && params[0].name == "a"
                    && params[1].name == "b"
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_procedure_header_with_params() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pro_alt_header_params.fol")
        .expect("Should read alternative parameterized procedure-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative procedure headers with params");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl {
                    name,
                    params,
                    return_type: Some(FolType::Int { .. }),
                    ..
                }
                if name == "main"
                    && params.len() == 1
                    && params[0].name == "value"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_procedure_header_with_params_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_alt_header_params_flow.fol")
            .expect("Should read alternative parameterized procedure-header flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied alternative procedure headers with params");

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
                if name == "main"
                    && params.len() == 1
                    && params[0].name == "value"
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_logical_header_with_params() {
    let mut file_stream = FileStream::from_file("test/parser/simple_log_alt_header_params.fol")
        .expect("Should read alternative parameterized logical-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative logical headers with params");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::LogDecl {
                    name,
                    params,
                    return_type: Some(FolType::Bool),
                    ..
                }
                if name == "ready"
                    && params.len() == 1
                    && params[0].name == "value"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_logical_header_with_params_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_log_alt_header_params_flow.fol")
            .expect("Should read alternative parameterized logical-header flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied alternative logical headers with params");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::LogDecl {
                    name,
                    params,
                    return_type: Some(FolType::Bool),
                    body,
                    ..
                }
                if name == "ready"
                    && params.len() == 1
                    && params[0].name == "value"
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_function_header_with_captures() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_alt_header_capture.fol")
        .expect("Should read alternative function-header capture fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse captures on alternative function headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, captures, params, .. }
                if name == "add"
                    && params.len() == 1
                    && captures == &vec!["n".to_string()]
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_function_header_with_captures_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_alt_header_capture_flow.fol")
            .expect("Should read alternative function-header capture flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse captures on flow-bodied alternative function headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    captures,
                    params,
                    body,
                    ..
                }
                if name == "add"
                    && params.len() == 1
                    && captures == &vec!["n".to_string()]
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_procedure_header_with_captures() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pro_alt_header_capture.fol")
        .expect("Should read alternative procedure-header capture fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse captures on alternative procedure headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl { name, captures, params, .. }
                if name == "main"
                    && params.len() == 1
                    && captures == &vec!["sink".to_string()]
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_procedure_header_with_captures_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_alt_header_capture_flow.fol")
            .expect("Should read alternative procedure-header capture flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse captures on flow-bodied alternative procedure headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl {
                    name,
                    captures,
                    params,
                    body,
                    ..
                }
                if name == "main"
                    && params.len() == 1
                    && captures == &vec!["sink".to_string()]
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_logical_header_with_captures() {
    let mut file_stream = FileStream::from_file("test/parser/simple_log_alt_header_capture.fol")
        .expect("Should read alternative logical-header capture fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse captures on alternative logical headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::LogDecl { name, captures, params, .. }
                if name == "ready"
                    && params.len() == 1
                    && captures == &vec!["state".to_string()]
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_logical_header_with_captures_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_log_alt_header_capture_flow.fol")
            .expect("Should read alternative logical-header capture flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse captures on flow-bodied alternative logical headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::LogDecl {
                    name,
                    captures,
                    params,
                    body,
                    ..
                }
                if name == "ready"
                    && params.len() == 1
                    && captures == &vec!["state".to_string()]
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_headers_accept_semicolon_capture_separators() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_alt_header_capture_semicolon.fol")
            .expect("Should read semicolon alternative-header capture fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse semicolon-separated captures on alternative headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, captures, params, .. }
                if name == "add"
                    && params.len() == 1
                    && captures == &vec!["n".to_string()]
            )));
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl { name, captures, params, .. }
                if name == "main"
                    && params.len() == 1
                    && captures == &vec!["sink".to_string()]
            )));
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::LogDecl { name, captures, params, .. }
                if name == "ready"
                    && params.len() == 1
                    && captures == &vec!["state".to_string()]
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_function_header_with_generics() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_alt_header_generics.fol")
        .expect("Should read alternative generic function-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative function headers with generics");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    generics,
                    params,
                    return_type: Some(FolType::Named { name: type_name , ..}),
                    ..
                }
                if name == "id"
                    && generics.len() == 1
                    && generics[0].name == "T"
                    && params.len() == 1
                    && type_name == "T"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_function_header_with_generics_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_alt_header_generics_flow.fol")
            .expect("Should read alternative generic function-header flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied alternative function headers with generics");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    generics,
                    params,
                    return_type: Some(FolType::Named { name: type_name , ..}),
                    body,
                    ..
                }
                if name == "id"
                    && generics.len() == 1
                    && generics[0].name == "T"
                    && params.len() == 1
                    && type_name == "T"
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_procedure_header_with_generics() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pro_alt_header_generics.fol")
        .expect("Should read alternative generic procedure-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative procedure headers with generics");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl {
                    name,
                    generics,
                    params,
                    return_type: Some(FolType::Named { name: type_name , ..}),
                    ..
                }
                if name == "wrap"
                    && generics.len() == 1
                    && generics[0].name == "T"
                    && params.len() == 1
                    && type_name == "T"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_procedure_header_with_generics_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_alt_header_generics_flow.fol")
            .expect("Should read alternative generic procedure-header flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied alternative procedure headers with generics");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl {
                    name,
                    generics,
                    params,
                    return_type: Some(FolType::Named { name: type_name , ..}),
                    body,
                    ..
                }
                if name == "wrap"
                    && generics.len() == 1
                    && generics[0].name == "T"
                    && params.len() == 1
                    && type_name == "T"
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_logical_header_with_generics() {
    let mut file_stream = FileStream::from_file("test/parser/simple_log_alt_header_generics.fol")
        .expect("Should read alternative generic logical-header fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse alternative logical headers with generics");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::LogDecl {
                    name,
                    generics,
                    params,
                    return_type: Some(FolType::Named { name: type_name , ..}),
                    ..
                }
                if name == "decide"
                    && generics.len() == 1
                    && generics[0].name == "T"
                    && params.len() == 1
                    && type_name == "T"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_logical_header_with_generics_supports_flow_body() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_log_alt_header_generics_flow.fol")
            .expect("Should read alternative generic logical-header flow fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse flow-bodied alternative logical headers with generics");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::LogDecl {
                    name,
                    generics,
                    params,
                    return_type: Some(FolType::Named { name: type_name , ..}),
                    body,
                    ..
                }
                if name == "decide"
                    && generics.len() == 1
                    && generics[0].name == "T"
                    && params.len() == 1
                    && type_name == "T"
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_function_header_with_flow_body_inquiries() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_alt_header_flow_inquiry.fol")
            .expect("Should read alternative function-header flow inquiry fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve inquiries on flow-bodied alternative function headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, inquiries, .. }
                if name == "add"
                    && inquiries.len() == 1
                    && matches!(&inquiries[0], AstNode::Inquiry { target, body } if inquiry_target_is(target, "self") && !body.is_empty())
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_procedure_header_with_flow_body_inquiries() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_pro_alt_header_flow_inquiry.fol")
            .expect("Should read alternative procedure-header flow inquiry fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve inquiries on flow-bodied alternative procedure headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl { name, inquiries, .. }
                if name == "main"
                    && inquiries.len() == 1
                    && matches!(&inquiries[0], AstNode::Inquiry { target, body } if inquiry_target_is(target, "this") && !body.is_empty())
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_alternative_logical_header_with_flow_body_inquiries() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_log_alt_header_flow_inquiry.fol")
            .expect("Should read alternative logical-header flow inquiry fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve inquiries on flow-bodied alternative logical headers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::LogDecl { name, inquiries, .. }
                if name == "ready"
                    && inquiries.len() == 1
                    && matches!(&inquiries[0], AstNode::Inquiry { target, body } if inquiry_target_is(target, "self") && !body.is_empty())
            )));
        }
        _ => panic!("Expected program node"),
    }
}
