use super::*;

#[test]
fn test_never_type_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_never_type_refs.fol")
        .expect("Should read never type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower nev[] references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Never
                    },
                    ..
                } if name == "Impossible"
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::Never),
                    ..
                }
                if name == "stop"
                    && matches!(params.as_slice(),
                        [Parameter { param_type: FolType::Never, .. }]
                    )
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::VarDecl {
                    name,
                    type_hint: Some(FolType::Never),
                    ..
                } if name == "doom"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_bare_never_type_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_bare_never_type_refs.fol")
        .expect("Should read bare never type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower bare nev references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Never
                    },
                    ..
                } if name == "Bottom"
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    return_type: Some(FolType::Never),
                    ..
                } if name == "crash"
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_bracketed_any_and_none_type_references_lower_structurally() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_bracketed_any_none_type_refs.fol")
            .expect("Should read bracketed any/none type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower bracketed any[] and none[] references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias { target: FolType::Any },
                    ..
                } if name == "Dynamic"
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias { target: FolType::None },
                    ..
                } if name == "Empty"
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::None),
                    ..
                }
                if name == "finish"
                    && matches!(params.as_slice(),
                        [Parameter { param_type: FolType::Any, .. }]
                    )
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_union_type_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_union_type_refs.fol")
        .expect("Should read union type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower uni[...] references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Union { types }
                    },
                    ..
                }
                if name == "DynamicNumber"
                    && matches!(types.as_slice(),
                        [FolType::Int { .. }, FolType::Float { .. }, FolType::Bool]
                    )
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::Union { types }),
                    ..
                }
                if name == "choose"
                    && matches!(
                        params.as_slice(),
                        [Parameter { param_type: FolType::Union { types: input_types }, .. }]
                        if matches!(input_types.as_slice(),
                            [FolType::Named { name: left }, FolType::Char { .. }]
                            if left == "str"
                        )
                    )
                    && matches!(types.as_slice(),
                        [FolType::Named { name: left }, FolType::Char { .. }]
                        if left == "str"
                    )
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_optional_type_shorthand_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_optional_type_shorthand.fol")
        .expect("Should read optional shorthand type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower ?T shorthand references");

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
                } if name == "MaybeCount"
                    && matches!(inner.as_ref(), FolType::Int { .. })
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::Optional { inner }),
                    ..
                } if name == "lookup"
                    && matches!(
                        params.as_slice(),
                        [Parameter { param_type: FolType::Optional { inner: param_inner }, .. }]
                        if matches!(param_inner.as_ref(), FolType::Named { name } if name == "str")
                    )
                    && matches!(inner.as_ref(), FolType::Named { name } if name == "str")
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_never_type_shorthand_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_never_type_shorthand.fol")
        .expect("Should read never shorthand type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower !T shorthand references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Never
                    },
                    ..
                } if name == "ImpossibleNumber"
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::FunDecl {
                    name,
                    params,
                    return_type: Some(FolType::Never),
                    ..
                } if name == "crashIfMissing"
                    && matches!(params.as_slice(), [Parameter { param_type: FolType::Never, .. }])
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_type_limit_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_type_limit_refs.fol")
        .expect("Should read type-limit reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower limited type references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Limited { base, limits }
                    },
                    ..
                } if name == "Byte"
                    && matches!(base.as_ref(), FolType::Int { .. })
                    && matches!(limits.as_slice(), [AstNode::FunctionCall { name, .. }] if name == "range")
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::VarDecl {
                    name,
                    type_hint: Some(FolType::Limited { base, limits }),
                    ..
                } if name == "username"
                    && matches!(
                        base.as_ref(),
                        FolType::Named { .. } | FolType::Sequence { .. } | FolType::Vector { .. }
                    )
                    && matches!(limits.as_slice(), [AstNode::FunctionCall { name, .. }] if name == "regex")
            )));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_channel_type_references_lower_structurally() {
    let mut file_stream = FileStream::from_file("test/parser/simple_channel_type_refs.fol")
        .expect("Should read channel type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower chn[...] references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::VarDecl {
                    name,
                    type_hint: Some(FolType::Channel { element_type }),
                    ..
                } if name == "channel"
                    && matches!(element_type.as_ref(), FolType::Named { name } if name == "str")
            )));

            assert!(declarations.iter().any(|node| matches!(
                node,
                AstNode::TypeDecl {
                    name,
                    type_def: TypeDefinition::Alias {
                        target: FolType::Channel { element_type }
                    },
                    ..
                } if name == "StringChannel"
                    && matches!(element_type.as_ref(), FolType::Named { name } if name == "str")
            )));
        }
        _ => panic!("Expected program node"),
    }
}
