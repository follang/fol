use super::*;

#[test]
fn test_aliases_and_bindings_accept_semicolon_type_arguments() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_type_args_semicolon_alias_binding.fol")
            .expect("Should read semicolon type-argument alias/binding fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse semicolon-separated type arguments in aliases and bindings");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Sequence { element_type }
                    },
                    ..
                }
                if name == "Numbers"
                    && matches!(element_type.as_ref(), FolType::Int { .. })
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::VarDecl {
                    name,
                    type_hint: Some(FolType::Sequence { element_type }),
                    ..
                }
                if name == "items"
                    && matches!(element_type.as_ref(), FolType::Int { .. })
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl { body, .. }
                if body.iter().any(|stmt| matches!(
                    stmt,
                    AstNode::VarDecl {
                        name,
                        type_hint: Some(FolType::Map { key_type, value_type }),
                        ..
                    }
                    if name == "cache"
                        && matches!(key_type.as_ref(), FolType::Named { name } if name == "str")
                        && matches!(
                            value_type.as_ref(),
                            FolType::Vector { element_type }
                            if matches!(element_type.as_ref(), FolType::Int { .. })
                        )
                ))
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_routines_and_use_declarations_accept_semicolon_type_arguments() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_type_args_semicolon_routine_use.fol")
            .expect("Should read semicolon type-argument routine/use fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse semicolon-separated type arguments in routines and uses");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::UseDecl {
                    name,
                    path_type: FolType::Module { name: module_name },
                    ..
                }
                if name == "loader" && module_name == "std"
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::Map { key_type, value_type }),
                    ..
                }
                if name == "read"
                    && params.len() == 1
                    && matches!(
                        params[0].param_type,
                        FolType::Sequence { ref element_type }
                        if matches!(element_type.as_ref(), FolType::Named { name } if name == "pkg::Input")
                    )
                    && matches!(key_type.as_ref(), FolType::Named { name } if name == "str")
                    && matches!(
                        value_type.as_ref(),
                        FolType::Vector { element_type }
                        if matches!(element_type.as_ref(), FolType::Named { name } if name == "pkg::Output")
                    )
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_type_bodies_accept_semicolon_type_arguments() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_type_args_semicolon_type_bodies.fol")
            .expect("Should read semicolon type-argument type-body fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse semicolon-separated type arguments in type bodies");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Record { fields, .. },
                    ..
                }
                if name == "Config"
                    && matches!(
                        fields.get("values"),
                        Some(FolType::Sequence { element_type })
                        if matches!(element_type.as_ref(), FolType::Int { .. })
                    )
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Entry { variants, .. },
                    ..
                }
                if name == "Event"
                    && matches!(
                        variants.get("Data"),
                        Some(Some(FolType::Map { key_type, value_type }))
                        if matches!(key_type.as_ref(), FolType::Named { name } if name == "str")
                            && matches!(
                                value_type.as_ref(),
                                FolType::Vector { element_type }
                                if matches!(element_type.as_ref(), FolType::Int { .. })
                            )
                    )
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_special_type_forms_accept_semicolon_type_arguments() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_type_args_semicolon_specials.fol")
            .expect("Should read semicolon type-argument special-type fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse semicolon-separated type arguments in special forms");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Optional { inner }
                    },
                    ..
                }
                if name == "MaybePath"
                    && matches!(inner.as_ref(), FolType::Path { name } if name == "std")
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Multiple { types }
                    },
                    ..
                }
                if name == "Paths"
                    && matches!(types.as_slice(),
                        [FolType::Path { name: left }, FolType::Url { name: right }]
                        if left == "std" && right == "web")
            )));

        }
        _ => panic!("Expected program node"),
    }
}
