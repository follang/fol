use super::*;

#[test]
fn test_shared_binding_value_expands_to_multiple_declarations() {
    let mut file_stream = FileStream::from_file("test/parser/simple_var_shared_value_multi.fol")
        .expect("Should read shared-value multi-binding fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept shared-value multi-bindings");

    match ast {
        AstNode::Program { declarations } => {
            let vars: Vec<_> = declarations
                .iter()
                .filter_map(|node| {
                    if let AstNode::VarDecl {
                        name,
                        type_hint,
                        value,
                        options,
                    } = node
                    {
                        Some((name, type_hint, value, options))
                    } else {
                        None
                    }
                })
                .collect();

            assert_eq!(
                vars.len(),
                2,
                "Multi-binding should expand into two VarDecl nodes"
            );
            assert!(vars.iter().any(|(name, _, _, _)| *name == "left"));
            assert!(vars.iter().any(|(name, _, _, _)| *name == "right"));
            assert!(vars.iter().all(|(_, type_hint, _, _)| {
                matches!(
                    type_hint,
                    Some(FolType::Int {
                        size: None,
                        signed: true
                    })
                )
            }));
            assert!(vars.iter().all(|(_, _, value, _)| value.is_some()));
            assert!(vars.iter().all(|(_, _, _, options)| {
                options.contains(&fol_parser::ast::VarOption::Mutable)
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_parallel_binding_values_expand_in_order() {
    let mut file_stream = FileStream::from_file("test/parser/simple_var_parallel_multi.fol")
        .expect("Should read parallel-value multi-binding fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept parallel-value multi-bindings");

    match ast {
        AstNode::Program { declarations } => {
            let vars: Vec<_> = declarations
                .iter()
                .filter_map(|node| {
                    if let AstNode::VarDecl {
                        name,
                        value: Some(value),
                        ..
                    } = node
                    {
                        Some((name.clone(), value.as_ref().clone()))
                    } else {
                        None
                    }
                })
                .collect();

            assert_eq!(
                vars.len(),
                2,
                "Parallel multi-binding should expand into two nodes"
            );
            assert!(matches!(
                vars.as_slice(),
                [(left, AstNode::Literal(Literal::Integer(1))), (right, AstNode::Literal(Literal::Integer(2)))]
                if left == "left" && right == "right"
            ));
        }
        _ => panic!("Expected program node"),
    }
}
