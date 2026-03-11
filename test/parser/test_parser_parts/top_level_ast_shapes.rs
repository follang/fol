use super::*;

fn parse_program_declarations(path: &str) -> Vec<AstNode> {
    let mut file_stream = FileStream::from_file(path).expect("Should read parser fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should produce a program AST for the fixture");

    match ast {
        AstNode::Program { declarations } => declarations,
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_top_level_fun_currently_leaks_body_before_declaration() {
    let declarations = parse_program_declarations("test/parser/simple_fun.fol");

    assert_eq!(
        declarations.len(),
        2,
        "Top-level function fixture should currently lower to leaked body plus declaration"
    );
    assert!(
        matches!(&declarations[0], AstNode::Return { .. }),
        "Current root contamination places the function body return before the declaration"
    );
    assert!(
        matches!(
            &declarations[1],
            AstNode::FunDecl { name, body, .. } if name == "add" && body.len() == 1
        ),
        "Second node should remain the authoritative function declaration"
    );
}

#[test]
fn test_top_level_pro_currently_leaks_body_before_declaration() {
    let declarations = parse_program_declarations("test/parser/simple_pro.fol");

    assert_eq!(
        declarations.len(),
        3,
        "Top-level procedure fixture should currently lower to leaked body statements plus declaration"
    );
    assert!(
        matches!(&declarations[0], AstNode::Assignment { .. }),
        "Current root contamination places the procedure body assignment before the declaration"
    );
    assert!(
        matches!(&declarations[1], AstNode::Return { .. }),
        "Current root contamination also leaks the procedure return before the declaration"
    );
    assert!(
        matches!(
            &declarations[2],
            AstNode::ProDecl { name, body, .. } if name == "update" && body.len() == 2
        ),
        "Final node should remain the authoritative procedure declaration"
    );
}
