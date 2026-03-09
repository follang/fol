use super::*;

#[test]
fn test_self_and_this_inquiry_targets_lower_structurally() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_inquiry_target_self_this.fol")
            .expect("Should read self/this inquiry target fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should keep self/this inquiry targets structurally");

    match ast {
        AstNode::Program { declarations } => {
            let Some(AstNode::FunDecl { inquiries, .. }) =
                declarations.iter().find(|node| matches!(node, AstNode::FunDecl { .. }))
            else {
                panic!("Expected a function declaration");
            };

            assert!(matches!(
                inquiries.as_slice(),
                [
                    AstNode::Inquiry {
                        target: InquiryTarget::SelfValue,
                        ..
                    },
                    AstNode::Inquiry {
                        target: InquiryTarget::ThisValue,
                        ..
                    }
                ]
            ));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_named_and_quoted_inquiry_targets_lower_structurally() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_inquiry_target_named_quoted.fol")
            .expect("Should read named/quoted inquiry target fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should keep named and quoted inquiry targets structurally");

    match ast {
        AstNode::Program { declarations } => {
            let Some(AstNode::FunDecl { inquiries, .. }) =
                declarations.iter().find(|node| matches!(node, AstNode::FunDecl { .. }))
            else {
                panic!("Expected a function declaration");
            };

            assert!(matches!(
                inquiries.as_slice(),
                [
                    AstNode::Inquiry {
                        target: InquiryTarget::Named(first),
                        ..
                    },
                    AstNode::Inquiry {
                        target: InquiryTarget::Quoted(second),
                        ..
                    }
                ] if first == "cache" && second == "sink"
            ));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_qualified_inquiry_targets_lower_structurally() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_inquiry_target_qualified.fol")
            .expect("Should read qualified inquiry target fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should keep qualified inquiry targets structurally");

    match ast {
        AstNode::Program { declarations } => {
            let Some(AstNode::FunDecl { inquiries, .. }) =
                declarations.iter().find(|node| matches!(node, AstNode::FunDecl { .. }))
            else {
                panic!("Expected a function declaration");
            };

            assert!(matches!(
                inquiries.as_slice(),
                [
                    AstNode::Inquiry {
                        target: InquiryTarget::Qualified(first),
                        ..
                    },
                    AstNode::Inquiry {
                        target: InquiryTarget::Qualified(second),
                        ..
                    }
                ]
                if first == &vec!["pkg".to_string(), "cache".to_string()]
                    && second == &vec!["sys".to_string(), "sink".to_string()]
            ));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_named_and_quoted_duplicate_inquiry_targets_are_rejected() {
    let message = {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_inquiry_target_duplicate_named_quoted.fol")
                .expect("Should read duplicate named/quoted inquiry target fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject duplicate named/quoted inquiry targets");

        errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError")
            .to_string()
    };

    assert!(
        message.contains("Duplicate inquiry clause for 'cache'"),
        "Expected duplicate inquiry diagnostic, got: {message}",
    );
}

#[test]
fn test_qualified_duplicate_inquiry_targets_are_rejected() {
    let message = {
        let mut file_stream =
            FileStream::from_file("test/parser/simple_inquiry_target_duplicate_qualified.fol")
                .expect("Should read duplicate qualified inquiry target fixture");

        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let errors = parser
            .parse(&mut lexer)
            .expect_err("Parser should reject duplicate qualified inquiry targets");

        errors
            .first()
            .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
            .expect("First parser error should be ParseError")
            .to_string()
    };

    assert!(
        message.contains("Duplicate inquiry clause for 'pkg::cache'"),
        "Expected duplicate inquiry diagnostic, got: {message}",
    );
}

#[test]
fn test_semicolon_separated_inquiry_targets_keep_structural_variants() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_fun_inquiry_target_semicolons.fol")
            .expect("Should read semicolon inquiry target fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should keep semicolon-separated inquiry targets structurally");

    match ast {
        AstNode::Program { declarations } => {
            let Some(AstNode::FunDecl { inquiries, .. }) =
                declarations.iter().find(|node| matches!(node, AstNode::FunDecl { .. }))
            else {
                panic!("Expected a function declaration");
            };

            assert!(matches!(
                inquiries.as_slice(),
                [
                    AstNode::Inquiry {
                        target: InquiryTarget::SelfValue,
                        ..
                    },
                    AstNode::Inquiry {
                        target: InquiryTarget::ThisValue,
                        ..
                    }
                ]
            ));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_named_routine_inquiries_keep_named_quoted_and_qualified_targets() {
    for (path, expected) in [
        (
            "test/parser/simple_fun_inquiry_named_target.fol",
            InquiryTarget::Named("cache".to_string()),
        ),
        (
            "test/parser/simple_fun_inquiry_quoted_target.fol",
            InquiryTarget::Quoted("cache".to_string()),
        ),
        (
            "test/parser/simple_fun_inquiry_qualified_target.fol",
            InquiryTarget::Qualified(vec!["pkg".to_string(), "cache".to_string()]),
        ),
    ] {
        let mut file_stream = FileStream::from_file(path).expect("Should read inquiry fixture");
        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should preserve inquiry target structure");

        match ast {
            AstNode::Program { declarations } => {
                let Some(AstNode::FunDecl { inquiries, .. }) =
                    declarations.iter().find(|node| matches!(node, AstNode::FunDecl { .. }))
                else {
                    panic!("Expected a function declaration");
                };

                assert!(
                    matches!(&inquiries[0], AstNode::Inquiry { target, .. } if target == &expected),
                    "Expected first inquiry target {:?} for fixture {}",
                    expected,
                    path
                );
            }
            _ => panic!("Expected program node"),
        }
    }
}

#[test]
fn test_flow_bodied_routines_keep_structural_inquiry_targets() {
    for (path, expected) in [
        (
            "test/parser/simple_fun_flow_body_inquiry.fol",
            InquiryTarget::SelfValue,
        ),
        (
            "test/parser/simple_pro_flow_body_inquiry.fol",
            InquiryTarget::ThisValue,
        ),
        (
            "test/parser/simple_log_flow_body_inquiry.fol",
            InquiryTarget::SelfValue,
        ),
    ] {
        let mut file_stream = FileStream::from_file(path).expect("Should read flow inquiry fixture");
        let mut lexer = Elements::init(&mut file_stream);
        let mut parser = AstParser::new();
        let ast = parser
            .parse(&mut lexer)
            .expect("Parser should preserve flow-body inquiry target structure");

        match ast {
            AstNode::Program { declarations } => {
                let inquiry_target = declarations.iter().find_map(|node| match node {
                    AstNode::FunDecl { inquiries, .. }
                    | AstNode::ProDecl { inquiries, .. } => inquiries.first(),
                    _ => None,
                });

                let Some(AstNode::Inquiry { target, .. }) = inquiry_target else {
                    panic!("Expected an inquiry clause");
                };

                assert_eq!(target, &expected, "Unexpected inquiry target for fixture {path}");
            }
            _ => panic!("Expected program node"),
        }
    }
}
