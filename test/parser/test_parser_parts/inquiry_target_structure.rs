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
