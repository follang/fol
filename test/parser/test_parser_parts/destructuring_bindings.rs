use super::*;

#[test]
fn test_destructuring_binding_supports_leading_rest_pattern() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_var_destructure_leading_rest.fol")
            .expect("Should read leading-rest destructuring fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept leading-rest destructuring bindings");

    assert!(matches!(
        ast,
        AstNode::Program { declarations }
            if declarations.iter().any(|node| matches!(
                node,
                AstNode::DestructureDecl {
                    pattern: BindingPattern::Sequence(parts),
                    ..
                } if matches!(
                    parts.as_slice(),
                    [BindingPattern::Name(start), BindingPattern::Rest(rest)]
                        if start == "start" && rest == "_"
                )
            ))
    ));
}

#[test]
fn test_destructuring_binding_supports_trailing_rest_pattern() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_var_destructure_trailing_rest.fol")
            .expect("Should read trailing-rest destructuring fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept trailing-rest destructuring bindings");

    assert!(matches!(
        ast,
        AstNode::Program { declarations }
            if declarations.iter().any(|node| matches!(
                node,
                AstNode::DestructureDecl {
                    pattern: BindingPattern::Sequence(parts),
                    ..
                } if matches!(
                    parts.as_slice(),
                    [BindingPattern::Rest(rest), BindingPattern::Name(end)]
                        if rest == "_" && end == "end"
                )
            ))
    ));
}

#[test]
fn test_destructuring_binding_supports_nested_patterns() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_var_destructure_nested.fol")
            .expect("Should read nested destructuring fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();

    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept nested destructuring bindings");

    assert!(matches!(
        ast,
        AstNode::Program { declarations }
            if declarations.iter().any(|node| matches!(
                node,
                AstNode::DestructureDecl {
                    pattern: BindingPattern::Sequence(parts),
                    ..
                } if matches!(
                    parts.as_slice(),
                    [
                        BindingPattern::Name(start),
                        BindingPattern::Rest(rest),
                        BindingPattern::Sequence(nested),
                    ]
                    if start == "start"
                        && rest == "_"
                        && matches!(
                            nested.as_slice(),
                            [BindingPattern::Name(letter), BindingPattern::Rest(inner_rest)]
                                if letter == "last_word_first_letter" && inner_rest == "_"
                        )
                )
            ))
    ));
}
