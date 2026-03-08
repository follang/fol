use super::*;

#[test]
fn test_qualified_quoted_type_references_parse() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_qualified_quoted_type_refs.fol")
            .expect("Should read qualified quoted type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted qualified type references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::AliasDecl { target: FolType::Named { name }, .. } if name == "core::Target")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { params, return_type: Some(FolType::Named { name }), .. }
                    if name == "errs::Output"
                        && matches!(params.as_slice(), [Parameter { param_type: FolType::Named { name }, .. }] if name == "pkg::Input")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_single_quoted_type_references_parse() {
    let mut file_stream = FileStream::from_file("test/parser/simple_single_quoted_type_refs.fol")
        .expect("Should read single-quoted type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept single-quoted type references");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(node, AstNode::AliasDecl { target: FolType::Named { name }, .. } if name == "Target")
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::FunDecl { params, return_type: Some(FolType::Named { name }), .. }
                    if name == "errs::Output"
                        && matches!(params.as_slice(), [Parameter { param_type: FolType::Named { name }, .. }] if name == "pkg::Input")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_quoted_type_references_compose_inside_nested_type_forms() {
    let mut file_stream = FileStream::from_file("test/parser/simple_quoted_nested_type_refs.fol")
        .expect("Should read nested quoted type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted names inside nested type forms");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Alias {
                            target: FolType::Vector { element_type }
                        },
                        ..
                    } if name == "Boxed"
                        && matches!(element_type.as_ref(), FolType::Named { name } if name == "Item")
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Alias {
                            target: FolType::Map { key_type, value_type }
                        },
                        ..
                    } if name == "Mapping"
                        && matches!(key_type.as_ref(), FolType::Named { name } if name == "Key")
                        && matches!(value_type.as_ref(), FolType::Named { name } if name == "pkg::Value")
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Alias {
                            target: FolType::Optional { inner }
                        },
                        ..
                    } if name == "Maybe"
                        && matches!(inner.as_ref(), FolType::Named { name } if name == "Inner")
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_quoted_type_references_compose_inside_array_and_matrix_types() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_quoted_array_matrix_type_refs.fol")
            .expect("Should read quoted array/matrix type-reference fixture");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should accept quoted names inside array and matrix types");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Alias {
                            target: FolType::Array { element_type, size }
                        },
                        ..
                    } if name == "Buffer"
                        && matches!(element_type.as_ref(), FolType::Named { name } if name == "Byte")
                        && *size == Some(16)
                )
            }));
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        name,
                        type_def: TypeDefinition::Alias {
                            target: FolType::Matrix { element_type, dimensions }
                        },
                        ..
                    } if name == "Grid"
                        && matches!(element_type.as_ref(), FolType::Named { name } if name == "pkg::Cell")
                        && dimensions.as_slice() == [4, 8]
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}
