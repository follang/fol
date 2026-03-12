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

#[test]
fn test_body_leading_comments_are_preserved_as_ast_nodes() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_body_comments.fol")
        .expect("Should read body comments fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve leading body comments in routine bodies");

    match ast {
        AstNode::Program { declarations } => {
            let body = only_root_routine_body_nodes(&declarations);

            assert!(matches!(
                body[0],
                AstNode::Comment {
                    kind: CommentKind::Doc,
                    ref raw,
                } if raw == "`[doc] body docs`"
            ));
            assert!(matches!(
                body[1],
                AstNode::Comment {
                    kind: CommentKind::Backtick,
                    ref raw,
                } if raw == "`body note`"
            ));
            assert!(matches!(
                body[2],
                AstNode::VarDecl {
                    ref name,
                    value: Some(ref value),
                    ..
                } if name == "alpha"
                    && matches!(value.as_ref(), AstNode::Literal(Literal::Integer(1)))
            ));
            assert!(matches!(
                body[3],
                AstNode::Return {
                    value: Some(ref value),
                } if matches!(value.as_ref(), AstNode::Identifier { name } if name == "alpha")
            ));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_root_comments_between_declarations_are_preserved() {
    let mut file_stream = FileStream::from_file("test/parser/simple_root_adjacent_comments.fol")
        .expect("Should read root adjacency comments fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve comments between root declarations");

    match ast {
        AstNode::Program { declarations } => {
            assert!(matches!(
                &declarations[0],
                AstNode::Comment {
                    kind: CommentKind::Doc,
                    raw,
                } if raw == "`[doc] alpha docs`"
            ));
            assert!(matches!(
                &declarations[1],
                AstNode::VarDecl {
                    name,
                    value: Some(value),
                    ..
                } if name == "alpha"
                    && matches!(value.as_ref(), AstNode::Literal(Literal::Integer(1)))
            ));
            assert!(matches!(
                &declarations[2],
                AstNode::Comment {
                    kind: CommentKind::Doc,
                    raw,
                } if raw == "`[doc] beta docs`"
            ));
            assert!(matches!(
                &declarations[3],
                AstNode::VarDecl {
                    name,
                    value: Some(value),
                    ..
                } if name == "beta"
                    && matches!(value.as_ref(), AstNode::Literal(Literal::Integer(2)))
            ));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_body_comments_between_statements_are_preserved() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_adjacent_comments.fol")
        .expect("Should read body adjacency comments fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should preserve comments between body statements");

    match ast {
        AstNode::Program { declarations } => {
            let body = only_root_routine_body_nodes(&declarations);

            assert!(matches!(
                body[0],
                AstNode::VarDecl {
                    ref name,
                    value: Some(ref value),
                    ..
                } if name == "alpha"
                    && matches!(value.as_ref(), AstNode::Literal(Literal::Integer(1)))
            ));
            assert!(matches!(
                body[1],
                AstNode::Comment {
                    kind: CommentKind::Doc,
                    ref raw,
                } if raw == "`[doc] beta docs`"
            ));
            assert!(matches!(
                body[2],
                AstNode::VarDecl {
                    ref name,
                    value: Some(ref value),
                    ..
                } if name == "beta"
                    && matches!(value.as_ref(), AstNode::Literal(Literal::Integer(2)))
            ));
            assert!(matches!(
                body[3],
                AstNode::Return {
                    value: Some(ref value),
                } if matches!(value.as_ref(), AstNode::Identifier { name } if name == "beta")
            ));
        }
        _ => panic!("Expected program node"),
    }
}
