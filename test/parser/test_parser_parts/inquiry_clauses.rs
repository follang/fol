use super::*;

#[test]
fn test_function_inquiry_clause_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_inquiry_clause.fol")
        .expect("Should read function inquiry clause test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse inquiry clauses attached to functions");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { name, inquiries, .. }
                    if name == "sum"
                        && inquiries.len() == 6
                        && matches!(&inquiries[0], AstNode::FunctionCall { .. })
                        && matches!(&inquiries[1], AstNode::Literal(Literal::Integer(0)))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_procedure_inquiry_clause_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_pro_inquiry_clause.fol")
        .expect("Should read procedure inquiry clause test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse inquiry clauses attached to procedures");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::ProDecl { name, inquiries, .. }
                    if name == "store"
                        && inquiries.len() == 4
                        && matches!(&inquiries[0], AstNode::FunctionCall { .. })
                        && matches!(&inquiries[1], AstNode::Literal(Literal::Integer(1)))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
