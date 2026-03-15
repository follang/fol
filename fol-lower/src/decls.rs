use crate::{
    LoweredFieldLayout, LoweredPackage, LoweredTypeDecl, LoweredTypeDeclKind,
    LoweredVariantLayout, LoweringError, LoweringErrorKind, LoweringResult,
};
use fol_parser::ast::{AstNode, TypeDefinition};
use fol_resolver::{SourceUnitId, SymbolId, SymbolKind};
use fol_typecheck::CheckedType;

pub fn lower_routine_signatures(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &mut LoweredPackage,
) -> LoweringResult<()> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package.program.resolved().syntax().source_units.iter().enumerate() {
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            let Some(name) = routine_name(&item.node) else {
                continue;
            };

            match find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Routine,
                name,
            ) {
                Some(symbol_id) => match lower_symbol_signature(typed_package, lowered_package, symbol_id) {
                    Ok(signature_type) => {
                        lowered_package.routine_signatures.insert(symbol_id, signature_type);
                    }
                    Err(error) => errors.push(error),
                },
                None => errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("top-level routine '{name}' does not retain a resolved symbol"),
                )),
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn lower_alias_declarations(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &mut LoweredPackage,
) -> LoweringResult<()> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package.program.resolved().syntax().source_units.iter().enumerate() {
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            let AstNode::AliasDecl { name, .. } = &item.node else {
                continue;
            };

            match find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Alias,
                name,
            ) {
                Some(symbol_id) => match lower_symbol_signature(typed_package, lowered_package, symbol_id) {
                    Ok(target_type) => {
                        lowered_package.type_decls.insert(
                            symbol_id,
                            LoweredTypeDecl {
                                symbol_id,
                                source_unit_id,
                                name: name.clone(),
                                runtime_type: target_type,
                                kind: LoweredTypeDeclKind::Alias { target_type },
                            },
                        );
                    }
                    Err(error) => errors.push(error),
                },
                None => errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("alias '{name}' does not retain a resolved symbol"),
                )),
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn lower_record_declarations(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &mut LoweredPackage,
) -> LoweringResult<()> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package.program.resolved().syntax().source_units.iter().enumerate() {
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            let AstNode::TypeDecl {
                name,
                type_def: TypeDefinition::Record { .. },
                ..
            } = &item.node
            else {
                continue;
            };

            match find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Type,
                name,
            ) {
                Some(symbol_id) => match lower_record_decl(
                    typed_package,
                    lowered_package,
                    symbol_id,
                    source_unit_id,
                    name,
                ) {
                    Ok(type_decl) => {
                        lowered_package.type_decls.insert(symbol_id, type_decl);
                    }
                    Err(error) => errors.push(error),
                },
                None => errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("record type '{name}' does not retain a resolved symbol"),
                )),
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn lower_entry_declarations(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &mut LoweredPackage,
) -> LoweringResult<()> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package.program.resolved().syntax().source_units.iter().enumerate() {
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            let AstNode::TypeDecl {
                name,
                type_def: TypeDefinition::Entry { .. },
                ..
            } = &item.node
            else {
                continue;
            };

            match find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Type,
                name,
            ) {
                Some(symbol_id) => match lower_entry_decl(
                    typed_package,
                    lowered_package,
                    symbol_id,
                    source_unit_id,
                    name,
                ) {
                    Ok(type_decl) => {
                        lowered_package.type_decls.insert(symbol_id, type_decl);
                    }
                    Err(error) => errors.push(error),
                },
                None => errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("entry type '{name}' does not retain a resolved symbol"),
                )),
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn lower_symbol_signature(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &LoweredPackage,
    symbol_id: SymbolId,
) -> Result<crate::LoweredTypeId, LoweringError> {
    let checked_signature = typed_package
        .program
        .typed_symbol(symbol_id)
        .and_then(|typed_symbol| typed_symbol.declared_type)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("routine symbol {} does not retain a typed signature", symbol_id.0),
            )
        })?;

    lowered_package
        .checked_type_map
        .get(&checked_signature)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "routine symbol {} does not map to a lowering-owned signature type",
                    symbol_id.0
                ),
            )
        })
}

fn lower_record_decl(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &LoweredPackage,
    symbol_id: SymbolId,
    source_unit_id: SourceUnitId,
    name: &str,
) -> Result<LoweredTypeDecl, LoweringError> {
    let checked_type = typed_package
        .program
        .typed_symbol(symbol_id)
        .and_then(|typed_symbol| typed_symbol.declared_type)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("record symbol {} does not retain a typed runtime shape", symbol_id.0),
            )
        })?;
    let runtime_type = lowered_package
        .checked_type_map
        .get(&checked_type)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("record symbol {} does not map to a lowered runtime type", symbol_id.0),
            )
        })?;
    let CheckedType::Record { fields } = typed_package
        .program
        .type_table()
        .get(checked_type)
        .cloned()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("record symbol {} lost its typed runtime definition", symbol_id.0),
            )
        })?
    else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("record symbol {} no longer lowers to a record shape", symbol_id.0),
        ));
    };
    let mut lowered_fields = Vec::new();
    for (field_name, field_type) in fields {
        let lowered_field_type = lowered_package.checked_type_map.get(&field_type).copied().ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "record field '{field_name}' on symbol {} does not map to a lowered type",
                    symbol_id.0
                ),
            )
        })?;
        lowered_fields.push(LoweredFieldLayout {
            name: field_name,
            type_id: lowered_field_type,
        });
    }

    Ok(LoweredTypeDecl {
        symbol_id,
        source_unit_id,
        name: name.to_string(),
        runtime_type,
        kind: LoweredTypeDeclKind::Record {
            fields: lowered_fields,
        },
    })
}

fn lower_entry_decl(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &LoweredPackage,
    symbol_id: SymbolId,
    source_unit_id: SourceUnitId,
    name: &str,
) -> Result<LoweredTypeDecl, LoweringError> {
    let checked_type = typed_package
        .program
        .typed_symbol(symbol_id)
        .and_then(|typed_symbol| typed_symbol.declared_type)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("entry symbol {} does not retain a typed runtime shape", symbol_id.0),
            )
        })?;
    let runtime_type = lowered_package
        .checked_type_map
        .get(&checked_type)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("entry symbol {} does not map to a lowered runtime type", symbol_id.0),
            )
        })?;
    let CheckedType::Entry { variants } = typed_package
        .program
        .type_table()
        .get(checked_type)
        .cloned()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("entry symbol {} lost its typed runtime definition", symbol_id.0),
            )
        })?
    else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("entry symbol {} no longer lowers to an entry shape", symbol_id.0),
        ));
    };
    let mut lowered_variants = Vec::new();
    for (variant_name, variant_type) in variants {
        let lowered_variant_type = variant_type
            .map(|variant_type| {
                lowered_package
                    .checked_type_map
                    .get(&variant_type)
                    .copied()
                    .ok_or_else(|| {
                        LoweringError::with_kind(
                            LoweringErrorKind::InvalidInput,
                            format!(
                                "entry variant '{variant_name}' on symbol {} does not map to a lowered type",
                                symbol_id.0
                            ),
                        )
                    })
            })
            .transpose()?;
        lowered_variants.push(LoweredVariantLayout {
            name: variant_name,
            payload_type: lowered_variant_type,
        });
    }

    Ok(LoweredTypeDecl {
        symbol_id,
        source_unit_id,
        name: name.to_string(),
        runtime_type,
        kind: LoweredTypeDeclKind::Entry {
            variants: lowered_variants,
        },
    })
}

fn routine_name(node: &AstNode) -> Option<&str> {
    match node {
        AstNode::FunDecl { name, .. }
        | AstNode::ProDecl { name, .. }
        | AstNode::LogDecl { name, .. } => Some(name.as_str()),
        _ => None,
    }
}

fn find_local_symbol_id(
    typed_program: &fol_typecheck::TypedProgram,
    source_unit_id: SourceUnitId,
    kind: SymbolKind,
    name: &str,
) -> Option<SymbolId> {
    typed_program
        .resolved()
        .symbols
        .iter_with_ids()
        .find(|(_, symbol)| {
            symbol.source_unit == source_unit_id
                && symbol.kind == kind
                && symbol.name == name
                && symbol.mounted_from.is_none()
        })
        .map(|(symbol_id, _)| symbol_id)
}

#[cfg(test)]
mod tests {
    use super::{
        lower_alias_declarations, lower_entry_declarations, lower_record_declarations,
        lower_routine_signatures,
    };
    use crate::{
        types::LoweredType, LoweredBuiltinType, LoweredFieldLayout, LoweredPackage,
        LoweredTypeDeclKind, LoweredVariantLayout,
    };
    use fol_parser::ast::AstParser;
    use fol_resolver::resolve_workspace;
    use fol_stream::FileStream;
    use fol_typecheck::Typechecker;

    #[test]
    fn declaration_lowering_maps_top_level_routine_signatures_to_lowered_types() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_routine_sig_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] greet(count: int): str = { return \"ok\" }",
        )
        .expect("should write lowering signature fixture");

        let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
            .expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");
        let lowered_workspace = crate::LoweringSession::new(typed.clone())
            .lower_workspace()
            .expect("workspace lowering should succeed");
        let typed_package = typed.entry_package();
        let mut lowered_package =
            LoweredPackage::new(crate::LoweredPackageId(0), typed_package.identity.clone());
        lowered_package.checked_type_map = lowered_workspace.entry_package().checked_type_map.clone();

        lower_routine_signatures(typed_package, &mut lowered_package)
            .expect("top-level routine signatures should lower cleanly");

        let lowered_signature = lowered_package
            .routine_signatures
            .values()
            .next()
            .expect("routine signature should be recorded");
        let signature = lowered_workspace
            .type_table()
            .get(*lowered_signature)
            .expect("lowered signature type should exist");

        match signature {
            LoweredType::Routine(signature) => {
                assert_eq!(
                    lowered_workspace.type_table().get(signature.params[0]),
                    Some(&LoweredType::Builtin(LoweredBuiltinType::Int))
                );
                assert_eq!(
                    signature
                        .return_type
                        .and_then(|type_id| lowered_workspace.type_table().get(type_id)),
                    Some(&LoweredType::Builtin(LoweredBuiltinType::Str))
                );
            }
            other => panic!("expected lowered routine type, got {other:?}"),
        }
    }

    #[test]
    fn declaration_lowering_records_aliases_as_erased_runtime_shapes() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_alias_decl_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "ali Counter: int\nfun[] main(value: Counter): Counter = { return value }",
        )
        .expect("should write lowering alias fixture");

        let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
            .expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");
        let lowered_workspace = crate::LoweringSession::new(typed.clone())
            .lower_workspace()
            .expect("workspace lowering should succeed");
        let typed_package = typed.entry_package();
        let mut lowered_package =
            LoweredPackage::new(crate::LoweredPackageId(0), typed_package.identity.clone());
        lowered_package.checked_type_map = lowered_workspace.entry_package().checked_type_map.clone();

        lower_alias_declarations(typed_package, &mut lowered_package)
            .expect("aliases should lower as erased runtime declarations");

        let alias_decl = lowered_package
            .type_decls
            .values()
            .next()
            .expect("alias declaration should be recorded");
        assert_eq!(alias_decl.name, "Counter");
        assert_eq!(
            lowered_workspace.type_table().get(alias_decl.runtime_type),
            Some(&LoweredType::Builtin(LoweredBuiltinType::Int))
        );
        assert_eq!(
            alias_decl.kind,
            LoweredTypeDeclKind::Alias {
                target_type: alias_decl.runtime_type
            }
        );
    }

    #[test]
    fn declaration_lowering_records_explicit_record_field_layouts() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_record_decl_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "typ Point: { x: int, y: str }\nfun[] main(): int = { return 0 }",
        )
        .expect("should write lowering record fixture");

        let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
            .expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");
        let lowered_workspace = crate::LoweringSession::new(typed.clone())
            .lower_workspace()
            .expect("workspace lowering should succeed");
        let typed_package = typed.entry_package();
        let mut lowered_package =
            LoweredPackage::new(crate::LoweredPackageId(0), typed_package.identity.clone());
        lowered_package.checked_type_map = lowered_workspace.entry_package().checked_type_map.clone();

        lower_record_declarations(typed_package, &mut lowered_package)
            .expect("record declarations should lower cleanly");

        let record_decl = lowered_package
            .type_decls
            .values()
            .next()
            .expect("record declaration should be recorded");

        assert_eq!(record_decl.name, "Point");
        assert_eq!(
            record_decl.kind,
            LoweredTypeDeclKind::Record {
                fields: vec![
                    LoweredFieldLayout {
                        name: "x".to_string(),
                        type_id: lowered_workspace
                            .entry_package()
                            .checked_type_map[&fol_typecheck::CheckedTypeId(0)],
                    },
                    LoweredFieldLayout {
                        name: "y".to_string(),
                        type_id: lowered_workspace
                            .entry_package()
                            .checked_type_map[&fol_typecheck::CheckedTypeId(4)],
                    },
                ],
            }
        );
        assert_eq!(
            lowered_workspace.type_table().get(record_decl.runtime_type),
            Some(&LoweredType::Record {
                fields: std::collections::BTreeMap::from([
                    (
                        "x".to_string(),
                        lowered_workspace.entry_package().checked_type_map[&fol_typecheck::CheckedTypeId(0)],
                    ),
                    (
                        "y".to_string(),
                        lowered_workspace.entry_package().checked_type_map[&fol_typecheck::CheckedTypeId(4)],
                    ),
                ]),
            })
        );
    }

    #[test]
    fn declaration_lowering_records_explicit_entry_variant_layouts() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_entry_decl_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "typ Outcome: ent = { var Ok: int; var Err: str }\nfun[] main(): int = { return 0 }",
        )
        .expect("should write lowering entry fixture");

        let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
            .expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");
        let lowered_workspace = crate::LoweringSession::new(typed.clone())
            .lower_workspace()
            .expect("workspace lowering should succeed");
        let typed_package = typed.entry_package();
        let mut lowered_package =
            LoweredPackage::new(crate::LoweredPackageId(0), typed_package.identity.clone());
        lowered_package.checked_type_map = lowered_workspace.entry_package().checked_type_map.clone();

        lower_entry_declarations(typed_package, &mut lowered_package)
            .expect("entry declarations should lower cleanly");

        let entry_decl = lowered_package
            .type_decls
            .values()
            .next()
            .expect("entry declaration should be recorded");

        assert_eq!(
            entry_decl.kind,
            LoweredTypeDeclKind::Entry {
                variants: vec![
                    LoweredVariantLayout {
                        name: "Err".to_string(),
                        payload_type: Some(
                            lowered_workspace.entry_package().checked_type_map
                                [&fol_typecheck::CheckedTypeId(4)],
                        ),
                    },
                    LoweredVariantLayout {
                        name: "Ok".to_string(),
                        payload_type: Some(
                            lowered_workspace.entry_package().checked_type_map
                                [&fol_typecheck::CheckedTypeId(0)],
                        ),
                    },
                ],
            }
        );
        assert_eq!(
            lowered_workspace.type_table().get(entry_decl.runtime_type),
            Some(&LoweredType::Entry {
                variants: std::collections::BTreeMap::from([
                    (
                        "Err".to_string(),
                        Some(
                            lowered_workspace.entry_package().checked_type_map
                                [&fol_typecheck::CheckedTypeId(4)],
                        ),
                    ),
                    (
                        "Ok".to_string(),
                        Some(
                            lowered_workspace.entry_package().checked_type_map
                                [&fol_typecheck::CheckedTypeId(0)],
                        ),
                    ),
                ]),
            })
        );
    }
}
