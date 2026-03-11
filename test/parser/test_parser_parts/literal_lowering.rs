use super::*;

#[test]
fn test_top_level_string_and_character_literals_lower_cleanly() {
    let mut file_stream = FileStream::from_file("test/parser/simple_literal_strings.fol")
        .expect("Should read literal lowering fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower top-level string-like literals");

    match ast {
        AstNode::Program { declarations } => {
            assert_eq!(
                declarations,
                vec![
                    AstNode::Literal(Literal::String("hello".to_string())),
                    AstNode::Literal(Literal::Character('c')),
                    AstNode::Literal(Literal::String("xy".to_string())),
                ],
                "Quoted literals should lower to clean AST values without wrapper quotes"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_top_level_integer_literals_lower_to_exact_values() {
    let mut file_stream = FileStream::from_file("test/parser/simple_literal_numbers.fol")
        .expect("Should read numeric literal lowering fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower top-level numeric literals");

    match ast {
        AstNode::Program { declarations } => {
            assert_eq!(
                declarations,
                vec![
                    AstNode::Literal(Literal::Integer(42)),
                    AstNode::Literal(Literal::Integer(1_000)),
                    AstNode::Literal(Literal::Integer(26)),
                    AstNode::Literal(Literal::Integer(15)),
                    AstNode::Literal(Literal::Integer(10)),
                ],
                "Numeric literals should preserve their integer value across supported families"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parse_literal_supports_float_payloads() {
    let parser = AstParser::new();
    let literal = parser
        .parse_literal("3.5")
        .expect("Direct parser literal lowering should support floats");

    assert_eq!(
        literal,
        AstNode::Literal(Literal::Float(3.5)),
        "Float payloads should lower to Literal::Float"
    );
}

#[test]
fn test_top_level_boolean_and_nil_literals_lower_cleanly() {
    let mut file_stream = FileStream::from_file("test/parser/simple_literal_logic.fol")
        .expect("Should read logical literal lowering fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower top-level boolean and nil literals");

    match ast {
        AstNode::Program { declarations } => {
            assert_eq!(
                declarations,
                vec![
                    AstNode::Literal(Literal::Boolean(true)),
                    AstNode::Literal(Literal::Boolean(false)),
                    AstNode::Literal(Literal::Nil),
                ],
                "Top-level logical literals should lower to concrete AST literals"
            );
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_top_level_float_literal_lowers_cleanly() {
    let mut file_stream = FileStream::from_file("test/parser/simple_literal_float.fol")
        .expect("Should read float literal lowering fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower top-level float literal");

    match ast {
        AstNode::Program { declarations } => {
            assert_eq!(
                declarations,
                vec![AstNode::Literal(Literal::Float(3.14))],
                "Top-level float literal should lower to Literal::Float"
            );
        }
        _ => panic!("Expected program node"),
    }
}
