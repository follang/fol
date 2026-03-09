use super::*;

#[test]
fn test_function_declaration_supports_flow_body() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_flow_body.fol")
        .expect("Should read function flow-body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse function declarations with flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, params, return_type: Some(FolType::Int { .. }), body, inquiries, .. }
                if name == "add"
                    && params.is_empty()
                    && inquiries.is_empty()
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_procedure_declaration_supports_flow_body() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pro_flow_body.fol")
        .expect("Should read procedure flow-body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse procedure declarations with flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl { name, params, return_type: Some(FolType::Int { .. }), body, inquiries, .. }
                if name == "main"
                    && params.is_empty()
                    && inquiries.is_empty()
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_logical_declaration_supports_flow_body() {
    let mut file_stream = FileStream::from_file("test/parser/simple_log_flow_body.fol")
        .expect("Should read logical flow-body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse logical declarations with flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, params, return_type: Some(FolType::Bool), body, inquiries, .. }
                if name == "ready"
                    && params.is_empty()
                    && inquiries.is_empty()
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_function_declaration_supports_parameterized_flow_body() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_flow_body_params.fol")
        .expect("Should read parameterized function flow-body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse parameterized function flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, params, return_type: Some(FolType::Int { .. }), body, .. }
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
fn test_procedure_declaration_supports_parameterized_flow_body() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pro_flow_body_params.fol")
        .expect("Should read parameterized procedure flow-body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse parameterized procedure flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl { name, params, return_type: Some(FolType::Int { .. }), body, .. }
                if name == "main"
                    && params.len() == 2
                    && params[0].name == "left"
                    && params[1].name == "right"
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_logical_declaration_supports_parameterized_flow_body() {
    let mut file_stream = FileStream::from_file("test/parser/simple_log_flow_body_params.fol")
        .expect("Should read parameterized logical flow-body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse parameterized logical flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, params, return_type: Some(FolType::Bool), body, .. }
                if name == "ready"
                    && params.len() == 2
                    && params[0].name == "left"
                    && params[1].name == "right"
                    && !body.is_empty()
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_function_declaration_supports_flow_body_inquiries() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_flow_body_inquiry.fol")
        .expect("Should read function flow-body inquiry fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve inquiries on function flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, inquiries, .. }
                if name == "add"
                    && inquiries.len() == 1
                    && matches!(&inquiries[0], AstNode::Inquiry { target, body } if target == "self" && !body.is_empty())
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_procedure_declaration_supports_flow_body_inquiries() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pro_flow_body_inquiry.fol")
        .expect("Should read procedure flow-body inquiry fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve inquiries on procedure flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::ProDecl { name, inquiries, .. }
                if name == "main"
                    && inquiries.len() == 1
                    && matches!(&inquiries[0], AstNode::Inquiry { target, body } if target == "this" && !body.is_empty())
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_logical_declaration_supports_flow_body_inquiries() {
    let mut file_stream = FileStream::from_file("test/parser/simple_log_flow_body_inquiry.fol")
        .expect("Should read logical flow-body inquiry fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve inquiries on logical flow bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { name, inquiries, .. }
                if name == "ready"
                    && inquiries.len() == 1
                    && matches!(&inquiries[0], AstNode::Inquiry { target, body } if target == "self" && !body.is_empty())
            )));
        }
        _ => panic!("Expected program node"),
    }
}
