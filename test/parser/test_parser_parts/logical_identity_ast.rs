use super::*;

#[test]
fn test_log_decl_get_type_defaults_to_bool() {
    let node = AstNode::LogDecl {
        options: Vec::new(),
        generics: Vec::new(),
        name: "dating".to_string(),
        receiver_type: None,
        captures: Vec::new(),
        params: Vec::new(),
        return_type: None,
        error_type: None,
        body: Vec::new(),
        inquiries: Vec::new(),
    };

    assert_eq!(
        node.get_type(),
        Some(FolType::Bool),
        "LogDecl without an explicit return type should still report bool as its routine type"
    );
}

#[test]
fn test_log_decl_children_include_body_and_inquiries() {
    let body_stmt = AstNode::Return {
        value: Some(Box::new(AstNode::Literal(Literal::Boolean(true)))),
    };
    let inquiry = AstNode::Inquiry {
        target: InquiryTarget::Named("ready".to_string()),
        body: vec![AstNode::Literal(Literal::Boolean(true))],
    };
    let node = AstNode::LogDecl {
        options: Vec::new(),
        generics: Vec::new(),
        name: "dating".to_string(),
        receiver_type: None,
        captures: Vec::new(),
        params: Vec::new(),
        return_type: Some(FolType::Bool),
        error_type: None,
        body: vec![body_stmt.clone()],
        inquiries: vec![inquiry.clone()],
    };

    assert_eq!(
        node.children(),
        vec![&body_stmt, &inquiry],
        "LogDecl tree traversal should expose both body statements and inquiry clauses"
    );
}
