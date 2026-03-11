use super::*;

#[test]
fn test_type_entry_missing_variant_name_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_entry_missing_variant_name.fol")
            .expect("Should read malformed type entry test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail when an entry variant name is missing");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected entry variant name"),
        "Malformed type entry should report missing variant name, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        2,
        "Type entry parse error should point to the malformed variant line"
    );
}

#[test]
fn test_duplicate_entry_variant_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_entry_duplicate_variant.fol")
            .expect("Should read duplicate type entry test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate entry variants");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Duplicate entry variant 'BLUE'"),
        "Duplicate entry variant should report the variant name, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        3,
        "Type entry duplicate parse error should point to the duplicate variant line"
    );
    assert_eq!(
        parse_error.column(),
        9,
        "Type entry duplicate parse error should point to the duplicate variant name"
    );
}

#[test]
fn test_canonical_duplicate_entry_variant_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_entry_duplicate_variant_canonical.fol")
            .expect("Should read canonical duplicate type entry test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject canonical duplicate entry variants");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Duplicate entry variant 'BlueValue'"),
        "Canonical duplicate entry variant should report the later spelling, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        3,
        "Canonical duplicate entry parse error should point to the duplicate variant line"
    );
}

#[test]
fn test_type_entry_marker_unknown_option_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_entry_marker_unknown_option.fol")
            .expect("Should read malformed entry marker option test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail when ent marker uses an unknown option");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Unknown entry type marker option"),
        "Malformed ent marker option should report unknown option, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        1,
        "Entry marker option parse error should point to the declaration line"
    );
}

#[test]
fn test_type_entry_definition_supports_lab_variants() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_entry_lab_variants.fol")
        .expect("Should read lab entry variant test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse entry definitions with lab variants");

    match ast {
        AstNode::Program { declarations } => {
            assert!(declarations.iter().any(|node| {
                matches!(
                    node,
                    AstNode::TypeDecl {
                        type_def: TypeDefinition::Entry { variants, .. },
                        ..
                    }
                    if matches!(variants.get("None"), Some(Some(FolType::None)))
                        && matches!(variants.get("Some"), Some(Some(FolType::Named { name })) if name == "str")
                        && matches!(variants.get("Many"), Some(Some(FolType::Int { size: None, signed: true })))
                )
            }));
        }
        _ => panic!("Expected program node"),
    }
}

#[test]
fn test_type_entry_definition_reports_missing_lab_variant_name() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_entry_missing_variant_label.fol")
            .expect("Should read malformed lab entry variant test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject lab variants without names");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected entry variant name"),
        "Malformed lab variant should report the missing name, got: {}",
        first_message
    );
}

#[test]
fn test_type_record_marker_missing_assign_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_record_marker_missing_assign.fol")
            .expect("Should read malformed type record marker test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail when rec marker is not followed by '='");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected '=' after record type marker"),
        "Malformed rec marker should report missing '=', got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        1,
        "Record marker parse error should point to the declaration line"
    );
}

#[test]
fn test_type_record_marker_unknown_option_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_record_marker_unknown_option.fol")
            .expect("Should read malformed record marker option test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail when rec marker uses an unknown option");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Unknown record type marker option"),
        "Malformed rec marker option should report unknown option, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        1,
        "Record marker option parse error should point to the declaration line"
    );
}

#[test]
fn test_type_generic_header_missing_separator_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_generics_missing_separator.fol")
            .expect("Should read malformed type generics test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail when type generic items are missing a separator");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected ',', ';', or ')' after generic parameter"),
        "Malformed type generic header should report missing separator, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        1,
        "Type generic header parse error should point to the declaration line"
    );
}

#[test]
fn test_type_record_missing_close_reports_parse_error() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_record_missing_close.fol")
        .expect("Should read malformed type record test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail when type record is missing closing '}'");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected '}' to close type record definition"),
        "Malformed type record should report missing close brace, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        3,
        "Type record parse error should point to the end of the declaration"
    );
}

#[test]
fn test_duplicate_record_method_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_record_duplicate_method.fol")
            .expect("Should read duplicate record method test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject duplicate record methods");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Duplicate type member 'getBrand#0'"),
        "Duplicate record method should report the member key, got: {}",
        parse_error
    );
}

#[test]
fn test_record_field_and_alias_name_conflict_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_record_field_alias_conflict.fol")
            .expect("Should read record field/alias conflict test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject record field and alias name conflicts");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Duplicate type member 'host'"),
        "Record field/alias conflict should report duplicate type member, got: {}",
        parse_error
    );
}

#[test]
fn test_entry_variant_and_type_name_conflict_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_entry_variant_type_conflict.fol")
            .expect("Should read entry variant/type conflict test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject entry variant and nested type name conflicts");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    assert!(
        parse_error.to_string().contains("Duplicate type member 'Ok'"),
        "Entry variant/type conflict should report duplicate type member, got: {}",
        parse_error
    );
}

#[test]
fn test_type_alias_parsing_supports_special_boxed_types() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_special_types.fol")
        .expect("Should read special type alias test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse special boxed typ aliases");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def: TypeDefinition::Alias {
                                target: FolType::Optional { inner }
                            },
                            ..
                        }
                        if name == "MaybeText"
                            && matches!(inner.as_ref(), FolType::Named { name } if name == "str")
                    )
                }),
                "Type alias should lower opt[...] to FolType::Optional"
            );
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias {
                                    target: FolType::Multiple { types }
                                },
                                ..
                            }
                            if name == "Pair"
                                && types.len() == 2
                                && matches!(types.first(), Some(FolType::Int { size: None, signed: true }))
                                && matches!(types.get(1), Some(FolType::Named { name }) if name == "str")
                        )
                    }),
                    "Type alias should lower mul[...] to FolType::Multiple"
                );
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def: TypeDefinition::Alias {
                                target: FolType::Pointer { target }
                            },
                            ..
                        }
                        if name == "CounterPtr"
                            && matches!(target.as_ref(), FolType::Int { size: None, signed: true })
                    )
                }),
                "Type alias should lower ptr[...] to FolType::Pointer"
            );
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def: TypeDefinition::Alias {
                                target: FolType::Error { inner: Some(inner) }
                            },
                            ..
                        }
                        if name == "Failure"
                            && matches!(inner.as_ref(), FolType::Named { name } if name == "str")
                    )
                }),
                "Type alias should lower err[T] to FolType::Error with payload"
            );
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def: TypeDefinition::Alias {
                                target: FolType::Error { inner: None }
                            },
                            ..
                        }
                        if name == "BareFailure"
                    )
                }),
                "Type alias should lower err[] to FolType::Error without payload"
            );
        }
        _ => panic!("Should return Program node"),
    }
}

#[test]
fn test_special_boxed_type_bad_arity_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_special_types_bad_arity.fol")
            .expect("Should read malformed special type alias test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail when opt[...] has the wrong number of arguments");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected exactly one type argument for opt[...]"),
        "Malformed opt type should report bad arity, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        1,
        "Special type bad-arity parse error should point to the declaration line"
    );
}

#[test]
fn test_type_alias_parsing_supports_container_types() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_container_types.fol")
        .expect("Should read container type alias test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse container typ aliases");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias {
                                    target: FolType::Vector { element_type }
                                },
                                ..
                            }
                            if name == "Names"
                                && matches!(element_type.as_ref(), FolType::Named { name } if name == "str")
                        )
                    }),
                    "Type alias should lower vec[...] to FolType::Vector"
                );
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias {
                                    target: FolType::Sequence { element_type }
                                },
                                ..
                            }
                            if name == "Queue"
                                && matches!(element_type.as_ref(), FolType::Named { name } if name == "pkg::Task")
                        )
                    }),
                    "Type alias should lower seq[...] to FolType::Sequence"
                );
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def: TypeDefinition::Alias {
                                target: FolType::Set { types }
                            },
                            ..
                        }
                        if name == "Palette"
                            && types.len() == 2
                    )
                }),
                "Type alias should lower set[...] to FolType::Set"
            );
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias {
                                    target: FolType::Map { key_type, value_type }
                                },
                                ..
                            }
                            if name == "Lookup"
                                && matches!(key_type.as_ref(), FolType::Named { name } if name == "str")
                                && matches!(
                                    value_type.as_ref(),
                                    FolType::Vector { element_type }
                                    if matches!(element_type.as_ref(), FolType::Named { name } if name == "pkg::Output")
                                )
                        )
                    }),
                    "Type alias should lower map[...] to FolType::Map"
                );
        }
        _ => panic!("Should return Program node"),
    }
}

#[test]
fn test_container_type_bad_arity_reports_parse_error() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_container_types_bad_arity.fol")
            .expect("Should read malformed container type alias test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should fail when map[...] has the wrong number of arguments");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Expected exactly two type arguments for map[...]"),
        "Malformed map type should report bad arity, got: {}",
        first_message
    );
    assert_eq!(
        parse_error.line(),
        1,
        "Container type bad-arity parse error should point to the declaration line"
    );
}

#[test]
fn test_type_alias_parsing_supports_module_and_block_types() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_module_block_types.fol")
        .expect("Should read module/block type alias test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse module and block typ aliases");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias { target: FolType::Module { name: module_name } },
                                ..
                            }
                            if name == "StdModule" && module_name == "std"
                        )
                    }),
                    "Type alias should lower mod[std] to FolType::Module"
                );
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias { target: FolType::Module { name: module_name } },
                                ..
                            }
                            if name == "InitModule" && module_name.is_empty()
                        )
                    }),
                    "Type alias should lower mod[] to an empty FolType::Module name"
                );
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias { target: FolType::Block { name: block_name } },
                                ..
                            }
                            if name == "JumpBlock" && block_name == "label"
                        )
                    }),
                    "Type alias should lower blk[label] to FolType::Block"
                );
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias { target: FolType::Block { name: block_name } },
                                ..
                            }
                            if name == "EmptyBlock" && block_name.is_empty()
                        )
                    }),
                    "Type alias should lower blk[] to an empty FolType::Block name"
                );
        }
        _ => panic!("Should return Program node"),
    }
}

#[test]
fn test_type_alias_parsing_supports_bare_module_and_block_types() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_bare_module_block_types.fol")
            .expect("Should read bare module/block type alias test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse bare module and block typ aliases");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias { target: FolType::Module { name: module_name } },
                                ..
                            }
                            if name == "BareModule" && module_name.is_empty()
                        )
                    }),
                    "Type alias should lower bare mod to FolType::Module"
                );
            assert!(
                    declarations.iter().any(|node| {
                        matches!(
                            node,
                            AstNode::TypeDecl {
                                name,
                                type_def: TypeDefinition::Alias { target: FolType::Block { name: block_name } },
                                ..
                            }
                            if name == "BareBlock" && block_name.is_empty()
                        )
                    }),
                    "Type alias should lower bare blk to FolType::Block"
                );
        }
        _ => panic!("Should return Program node"),
    }
}

#[test]
fn test_type_alias_parsing_supports_any_and_none_types() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_any_none.fol")
        .expect("Should read any/none type alias test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse any/non typ aliases");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def: TypeDefinition::Alias { target: FolType::Any },
                            ..
                        }
                        if name == "Anything"
                    )
                }),
                "Type alias should lower any to FolType::Any"
            );
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def: TypeDefinition::Alias { target: FolType::None },
                            ..
                        }
                        if name == "Nothing"
                    )
                }),
                "Type alias should lower non to FolType::None"
            );
        }
        _ => panic!("Should return Program node"),
    }
}

#[test]
fn test_type_alias_parsing_lowers_scalar_types() {
    let mut file_stream = FileStream::from_file("test/parser/simple_typ_scalar_types.fol")
        .expect("Should read scalar type alias test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower scalar type aliases");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def:
                                TypeDefinition::Alias {
                                    target: FolType::Int {
                                        size: Some(IntSize::I32),
                                        signed: false,
                                    }
                                },
                            ..
                        } if name == "UnsignedWord"
                    )
                }),
                "int[u32] alias should lower to unsigned I32"
            );
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def:
                                TypeDefinition::Alias {
                                    target: FolType::Int {
                                        size: Some(IntSize::I64),
                                        signed: true,
                                    }
                                },
                            ..
                        } if name == "SignedWord"
                    )
                }),
                "int[64] alias should lower to signed I64"
            );
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def:
                                TypeDefinition::Alias {
                                    target: FolType::Float {
                                        size: Some(FloatSize::F32)
                                    }
                                },
                            ..
                        } if name == "FloatSmall" || name == "BareF32"
                    )
                }),
                "flt[32] and bare f32 aliases should lower to F32"
            );
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def:
                                TypeDefinition::Alias {
                                    target: FolType::Char {
                                        encoding: CharEncoding::Utf16
                                    }
                                },
                            ..
                        } if name == "WideRune"
                    )
                }),
                "chr[utf16] alias should lower to Utf16 chars"
            );
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::TypeDecl {
                            name,
                            type_def:
                                TypeDefinition::Alias {
                                    target: FolType::Int {
                                        size: Some(IntSize::I64),
                                        signed: false,
                                    }
                                },
                            ..
                        } if name == "BareU64"
                    )
                }),
                "bare u64 alias should lower to unsigned I64"
            );
        }
        _ => panic!("Should return Program node"),
    }
}

#[test]
fn test_routine_signatures_support_scalar_types() {
    let mut file_stream = FileStream::from_file("test/parser/simple_fun_scalar_types.fol")
        .expect("Should read scalar-typed routine test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should lower scalar types in routine signatures");

    match ast {
        AstNode::Program { declarations } => {
            let function = declarations.iter().find_map(|node| match node {
                AstNode::FunDecl {
                    name,
                    params,
                    return_type,
                    error_type,
                    ..
                } if name == "convert" => Some((params, return_type, error_type)),
                _ => None,
            });

            let (params, return_type, error_type) =
                function.expect("Program should contain convert function");

            assert!(matches!(
                params.first(),
                Some(Parameter {
                    param_type: FolType::Int {
                        size: Some(IntSize::I16),
                        signed: false,
                    },
                    ..
                })
            ));
            assert!(matches!(
                params.get(1),
                Some(Parameter {
                    param_type: FolType::Float {
                        size: Some(FloatSize::F32),
                    },
                    ..
                })
            ));
            assert!(matches!(
                params.get(2),
                Some(Parameter {
                    param_type: FolType::Char {
                        encoding: CharEncoding::Utf32,
                    },
                    ..
                })
            ));
            assert!(matches!(
                return_type,
                Some(FolType::Int {
                    size: Some(IntSize::I64),
                    signed: true,
                })
            ));
            assert!(matches!(error_type, Some(FolType::Bool)));
        }
        _ => panic!("Should return Program node"),
    }
}

#[test]
fn test_type_alias_parsing_rejects_unknown_scalar_type_option() {
    let mut file_stream =
        FileStream::from_file("test/parser/simple_typ_scalar_type_bad_option.fol")
            .expect("Should read malformed scalar type alias test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse(&mut lexer)
        .expect_err("Parser should reject unknown scalar type options");

    let parse_error = errors
        .first()
        .and_then(|e| e.as_ref().as_any().downcast_ref::<ParseError>())
        .expect("First parser error should be ParseError");

    let first_message = parse_error.to_string();
    assert!(
        first_message.contains("Unknown integer type option 'wat'"),
        "Malformed scalar type alias should report unknown option, got: {}",
        first_message
    );
}

#[test]
fn test_use_declarations_support_module_types() {
    let mut file_stream = FileStream::from_file("test/parser/simple_use_mod_type_lowering.fol")
        .expect("Should read module-typed use declaration test file");

    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let ast = parser
        .parse(&mut lexer)
        .expect("Parser should parse use declarations with module path types");

    match ast {
        AstNode::Program { declarations } => {
            assert!(
                declarations.iter().any(|node| {
                    matches!(
                        node,
                        AstNode::UseDecl {
                            name,
                            path_type: FolType::Module { name: module_name },
                            path,
                            ..
                        }
                        if name == "file" && module_name == "std" && path == "std::fs::File"
                    )
                }),
                "Use declarations should lower mod[...] path types to FolType::Module"
            );
        }
        _ => panic!("Should return Program node"),
    }
}
