use super::*;

#[test]
fn test_root_comments_are_preserved_as_ast_nodes() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_root_comments.fol").expect("Should read root comments fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve leading root comments in the AST");

    match ast {
        AstNode::Program { declarations } => {
            assert_eq!(
                declarations.len(),
                3,
                "Expected two preserved root comments followed by the variable declaration"
            );

            assert!(matches!(
                &declarations[0],
                AstNode::Comment {
                    kind: CommentKind::Doc,
                    raw,
                } if raw == "`[doc] module docs`"
            ));
            assert!(matches!(
                &declarations[1],
                AstNode::Comment {
                    kind: CommentKind::Backtick,
                    raw,
                } if raw == "`ordinary root note`"
            ));
            assert!(matches!(
                &declarations[2],
                AstNode::VarDecl {
                    name,
                    value: Some(value),
                    ..
                } if name == "alpha"
                    && matches!(value.as_ref(), AstNode::Literal(Literal::Integer(1)))
            ));
        }
        _ => panic!("Expected program node"),
    }
}
