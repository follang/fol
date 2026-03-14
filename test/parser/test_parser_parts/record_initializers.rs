use super::*;

#[test]
fn test_named_record_initializer_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_record_init_named.fol")
        .expect("Should read named record initializer fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse named record initializers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { value: Some(value), .. }
                        if matches!(value.as_ref(), AstNode::RecordInit { fields, .. } if fields.len() == 4)
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_nested_record_initializer_parsing() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_record_init_nested.fol")
        .expect("Should read nested record initializer fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse nested record initializers");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::VarDecl { value: Some(value), .. }
                        if matches!(value.as_ref(), AstNode::RecordInit { fields, .. }
                            if fields.iter().any(|field|
                                field.name == "MonthlySalary"
                                    && matches!(field.value, AstNode::RecordInit { .. })))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
