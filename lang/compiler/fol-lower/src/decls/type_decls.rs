use crate::{
    LoweredFieldLayout, LoweredGlobal, LoweredPackage, LoweredTypeDecl, LoweredTypeDeclKind,
    LoweredVariantLayout, LoweringError, LoweringErrorKind, LoweringResult,
};
use fol_parser::ast::{AstNode, ParsedSourceUnitKind, TypeDefinition};
use fol_resolver::{SourceUnitId, SymbolId, SymbolKind};
use fol_typecheck::CheckedType;

use super::symbol_lookup::find_local_symbol_id;

pub fn lower_alias_declarations(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &mut LoweredPackage,
) -> LoweringResult<()> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package
        .program
        .resolved()
        .syntax()
        .source_units
        .iter()
        .enumerate()
    {
        if source_unit.kind == ParsedSourceUnitKind::Build {
            continue;
        }
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
                Some(symbol_id) => {
                    match lower_symbol_signature(typed_package, lowered_package, symbol_id) {
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
                    }
                }
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

    for (source_unit_index, source_unit) in typed_package
        .program
        .resolved()
        .syntax()
        .source_units
        .iter()
        .enumerate()
    {
        if source_unit.kind == ParsedSourceUnitKind::Build {
            continue;
        }
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

    for (source_unit_index, source_unit) in typed_package
        .program
        .resolved()
        .syntax()
        .source_units
        .iter()
        .enumerate()
    {
        if source_unit.kind == ParsedSourceUnitKind::Build {
            continue;
        }
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

pub fn lower_global_declarations(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &mut LoweredPackage,
    next_global_index: &mut usize,
) -> LoweringResult<()> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package
        .program
        .resolved()
        .syntax()
        .source_units
        .iter()
        .enumerate()
    {
        if source_unit.kind == ParsedSourceUnitKind::Build {
            continue;
        }
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            let (name, kind, mutable) = match &item.node {
                AstNode::VarDecl { name, .. } => (name.as_str(), SymbolKind::ValueBinding, true),
                AstNode::LabDecl { name, .. } => (name.as_str(), SymbolKind::LabelBinding, false),
                _ => continue,
            };

            match find_local_symbol_id(&typed_package.program, source_unit_id, kind, name) {
                Some(symbol_id) => match lower_global_decl(
                    typed_package,
                    lowered_package,
                    symbol_id,
                    source_unit_id,
                    name,
                    mutable,
                    next_global_index,
                ) {
                    Ok(global) => {
                        lowered_package.globals.push(global.id);
                        lowered_package.global_decls.insert(global.id, global);
                    }
                    Err(error) => errors.push(error),
                },
                None => errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("top-level binding '{name}' does not retain a resolved symbol"),
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

pub(super) fn lower_symbol_signature(
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
                format!(
                    "routine symbol {} does not retain a typed signature",
                    symbol_id.0
                ),
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
                format!(
                    "record symbol {} does not retain a typed runtime shape",
                    symbol_id.0
                ),
            )
        })?;
    let runtime_type = lowered_package
        .checked_type_map
        .get(&checked_type)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "record symbol {} does not map to a lowered runtime type",
                    symbol_id.0
                ),
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
                format!(
                    "record symbol {} lost its typed runtime definition",
                    symbol_id.0
                ),
            )
        })?
    else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "record symbol {} no longer lowers to a record shape",
                symbol_id.0
            ),
        ));
    };
    let mut lowered_fields = Vec::new();
    for (field_name, field_type) in fields {
        let lowered_field_type = lowered_package
            .checked_type_map
            .get(&field_type)
            .copied()
            .ok_or_else(|| {
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
                format!(
                    "entry symbol {} does not retain a typed runtime shape",
                    symbol_id.0
                ),
            )
        })?;
    let runtime_type = lowered_package
        .checked_type_map
        .get(&checked_type)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "entry symbol {} does not map to a lowered runtime type",
                    symbol_id.0
                ),
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
                format!(
                    "entry symbol {} lost its typed runtime definition",
                    symbol_id.0
                ),
            )
        })?
    else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "entry symbol {} no longer lowers to an entry shape",
                symbol_id.0
            ),
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

pub(super) fn lower_global_decl(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &LoweredPackage,
    symbol_id: SymbolId,
    source_unit_id: SourceUnitId,
    name: &str,
    mutable: bool,
    next_global_index: &mut usize,
) -> Result<LoweredGlobal, LoweringError> {
    let checked_type = typed_package
        .program
        .typed_symbol(symbol_id)
        .and_then(|typed_symbol| typed_symbol.declared_type)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "global symbol {} does not retain a checked type",
                    symbol_id.0
                ),
            )
        })?;
    let type_id = lowered_package
        .checked_type_map
        .get(&checked_type)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "global symbol {} does not map to a lowered type",
                    symbol_id.0
                ),
            )
        })?;
    let global = LoweredGlobal {
        id: crate::LoweredGlobalId(*next_global_index),
        symbol_id,
        source_unit_id,
        name: name.to_string(),
        type_id,
        recoverable_error_type: typed_package
            .program
            .typed_symbol(symbol_id)
            .and_then(|symbol| symbol.recoverable_effect)
            .and_then(|effect| {
                lowered_package
                    .checked_type_map
                    .get(&effect.error_type)
                    .copied()
            }),
        mutable,
    };
    *next_global_index += 1;
    Ok(global)
}
