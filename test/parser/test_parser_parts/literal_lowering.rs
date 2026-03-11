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
