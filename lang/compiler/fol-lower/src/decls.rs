use crate::{
    LoweredBlock, LoweredFieldLayout, LoweredGlobal, LoweredLocal, LoweredPackage, LoweredRoutine,
    LoweredTypeDecl, LoweredTypeDeclKind, LoweredVariantLayout, LoweringError, LoweringErrorKind,
    LoweringResult,
};
use fol_parser::ast::{AstNode, ParsedSourceUnitKind, TypeDefinition};
use fol_resolver::{SourceUnitId, SymbolId, SymbolKind};
use fol_typecheck::CheckedType;

pub fn lower_routine_signatures(
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
            let Some(name) = routine_name(&item.node) else {
                continue;
            };

            match find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Routine,
                name,
            ) {
                Some(symbol_id) => {
                    match lower_symbol_signature(typed_package, lowered_package, symbol_id) {
                        Ok(signature_type) => {
                            lowered_package
                                .routine_signatures
                                .insert(symbol_id, signature_type);
                        }
                        Err(error) => errors.push(error),
                    }
                }
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

pub fn lower_routine_declarations(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &mut LoweredPackage,
    next_routine_index: &mut usize,
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
            let (name, syntax_id, params) = match &item.node {
                AstNode::FunDecl {
                    name,
                    syntax_id,
                    params,
                    ..
                }
                | AstNode::ProDecl {
                    name,
                    syntax_id,
                    params,
                    ..
                }
                | AstNode::LogDecl {
                    name,
                    syntax_id,
                    params,
                    ..
                } => (name.as_str(), *syntax_id, params.as_slice()),
                _ => continue,
            };

            match find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Routine,
                name,
            ) {
                Some(symbol_id) => match lower_routine_decl(
                    typed_package,
                    lowered_package,
                    symbol_id,
                    source_unit_id,
                    name,
                    syntax_id,
                    params,
                    next_routine_index,
                ) {
                    Ok(routine) => {
                        lowered_package.routines.push(routine.id);
                        lowered_package.routine_decls.insert(routine.id, routine);
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

fn lower_global_decl(
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

fn lower_routine_decl(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &LoweredPackage,
    symbol_id: SymbolId,
    source_unit_id: SourceUnitId,
    name: &str,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    params: &[fol_parser::ast::Parameter],
    next_routine_index: &mut usize,
) -> Result<LoweredRoutine, LoweringError> {
    let signature = lowered_package
        .routine_signatures
        .get(&symbol_id)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "routine symbol {} does not retain a lowered signature",
                    symbol_id.0
                ),
            )
        })?;
    let typed_symbol = typed_package
        .program
        .typed_symbol(symbol_id)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "routine symbol {} disappeared from typed facts",
                    symbol_id.0
                ),
            )
        })?;

    let mut routine = LoweredRoutine::new(
        crate::LoweredRoutineId(*next_routine_index),
        name,
        crate::LoweredBlockId(0),
    );
    *next_routine_index += 1;
    routine.symbol_id = Some(symbol_id);
    routine.source_unit_id = Some(source_unit_id);
    routine.signature = Some(signature);
    routine.receiver_type = typed_symbol.receiver_type.and_then(|receiver_type| {
        lowered_package
            .checked_type_map
            .get(&receiver_type)
            .copied()
    });
    let entry_block = routine.blocks.push(LoweredBlock {
        id: crate::LoweredBlockId(0),
        instructions: Vec::new(),
        terminator: None,
    });
    routine.entry_block = entry_block;

    let mut next_local_index = 0;
    if let Some(receiver_type) = routine.receiver_type {
        let local_id = routine.locals.push(LoweredLocal {
            id: crate::LoweredLocalId(next_local_index),
            type_id: Some(receiver_type),
            recoverable_error_type: None,
            name: Some("self".to_string()),
        });
        routine.params.push(local_id);
        next_local_index += 1;
    }

    let routine_scope_id = syntax_id
        .and_then(|syntax_id| typed_package.program.resolved().scope_for_syntax(syntax_id))
        .unwrap_or(typed_symbol.scope_id);
    let checked_signature = typed_symbol.declared_type.ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "routine symbol {} does not retain a checked signature",
                symbol_id.0
            ),
        )
    })?;
    let checked_param_types = match typed_package
        .program
        .type_table()
        .get(checked_signature)
        .cloned()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "routine symbol {} lost its checked signature shape",
                    symbol_id.0
                ),
            )
        })? {
        CheckedType::Routine(signature) => signature.params,
        _ => {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "routine symbol {} no longer lowers to a routine shape",
                    symbol_id.0
                ),
            ))
        }
    };

    for (param, checked_param_type) in params.iter().zip(checked_param_types.into_iter()) {
        let param_type = lowered_package
            .checked_type_map
            .get(&checked_param_type)
            .copied()
            .ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "routine parameter '{}' on symbol {} does not map to a lowered type",
                        param.name, symbol_id.0
                    ),
                )
            })?;
        let local_id = routine.locals.push(LoweredLocal {
            id: crate::LoweredLocalId(next_local_index),
            type_id: Some(param_type),
            recoverable_error_type: None,
            name: Some(param.name.clone()),
        });
        let Some(param_symbol_id) = find_symbol_in_scope_or_descendants(
            &typed_package.program,
            source_unit_id,
            routine_scope_id,
            SymbolKind::Parameter,
            &param.name,
        ) else {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "routine parameter '{}' does not map to a resolved symbol in its lowered scope",
                    param.name
                ),
            ));
        };
        routine.local_symbols.insert(param_symbol_id, local_id);
        routine.params.push(local_id);
        next_local_index += 1;
    }

    Ok(routine)
}

fn routine_name(node: &AstNode) -> Option<&str> {
    match node {
        AstNode::FunDecl { name, .. }
        | AstNode::ProDecl { name, .. }
        | AstNode::LogDecl { name, .. } => Some(name.as_str()),
        _ => None,
    }
}

pub(crate) fn find_local_symbol_id(
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

pub(crate) fn find_symbol_in_exact_scope(
    typed_program: &fol_typecheck::TypedProgram,
    source_unit_id: SourceUnitId,
    scope_id: fol_resolver::ScopeId,
    kind: SymbolKind,
    name: &str,
) -> Option<SymbolId> {
    typed_program
        .resolved()
        .symbols
        .iter_with_ids()
        .find(|(_, symbol)| {
            symbol.source_unit == source_unit_id
                && symbol.scope == scope_id
                && symbol.kind == kind
                && symbol.name == name
                && symbol.mounted_from.is_none()
        })
        .map(|(symbol_id, _)| symbol_id)
}

pub(crate) fn find_symbol_in_scope_or_descendants(
    typed_program: &fol_typecheck::TypedProgram,
    source_unit_id: SourceUnitId,
    scope_id: fol_resolver::ScopeId,
    kind: SymbolKind,
    name: &str,
) -> Option<SymbolId> {
    let exact = find_symbol_in_exact_scope(typed_program, source_unit_id, scope_id, kind, name);
    if exact.is_some() {
        return exact;
    }

    let mut candidates = typed_program
        .resolved()
        .symbols
        .iter_with_ids()
        .filter_map(|(symbol_id, symbol)| {
            (symbol.source_unit == source_unit_id
                && symbol.kind == kind
                && symbol.name == name
                && symbol.mounted_from.is_none())
            .then_some((symbol_id, symbol.scope))
        })
        .filter_map(|(symbol_id, candidate_scope)| {
            scope_distance_from(typed_program.resolved(), candidate_scope, scope_id)
                .map(|distance| (distance, symbol_id))
        })
        .collect::<Vec<_>>();

    candidates.sort_by_key(|(distance, symbol_id)| (*distance, symbol_id.0));
    let (best_distance, best_symbol) = candidates.first().copied()?;
    let next_distance = candidates.get(1).map(|(distance, _)| *distance);
    (next_distance != Some(best_distance)).then_some(best_symbol)
}

fn scope_distance_from(
    resolved: &fol_resolver::ResolvedProgram,
    mut candidate_scope: fol_resolver::ScopeId,
    target_scope: fol_resolver::ScopeId,
) -> Option<usize> {
    let mut distance = 0usize;
    loop {
        if candidate_scope == target_scope {
            return Some(distance);
        }
        let scope = resolved.scopes.get(candidate_scope)?;
        candidate_scope = scope.parent?;
        distance += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        lower_alias_declarations, lower_entry_declarations, lower_global_declarations,
        lower_record_declarations, lower_routine_decl, lower_routine_declarations,
        lower_routine_signatures,
    };
    use crate::{
        types::LoweredType, LoweredBuiltinType, LoweredFieldLayout, LoweredPackage,
        LoweredTypeDeclKind, LoweredVariantLayout,
    };
    use fol_parser::ast::{AstNode, AstParser, FolType, Parameter};
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
        std::fs::write(&fixture, "fun[] greet(count: int): str = { return \"ok\" }")
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
        lowered_package.checked_type_map =
            lowered_workspace.entry_package().checked_type_map.clone();

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
        lowered_package.checked_type_map =
            lowered_workspace.entry_package().checked_type_map.clone();

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
        lowered_package.checked_type_map =
            lowered_workspace.entry_package().checked_type_map.clone();

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
                        type_id: lowered_workspace.entry_package().checked_type_map
                            [&fol_typecheck::CheckedTypeId(0)],
                    },
                    LoweredFieldLayout {
                        name: "y".to_string(),
                        type_id: lowered_workspace.entry_package().checked_type_map
                            [&fol_typecheck::CheckedTypeId(4)],
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
                        lowered_workspace.entry_package().checked_type_map
                            [&fol_typecheck::CheckedTypeId(0)],
                    ),
                    (
                        "y".to_string(),
                        lowered_workspace.entry_package().checked_type_map
                            [&fol_typecheck::CheckedTypeId(4)],
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
        lowered_package.checked_type_map =
            lowered_workspace.entry_package().checked_type_map.clone();

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

    #[test]
    fn declaration_lowering_records_top_level_globals_with_storage_types() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_globals_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(&fixture, "var count: int = 1\nlab label: str = \"ok\"")
            .expect("should write lowering global fixture");

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
        lowered_package.checked_type_map =
            lowered_workspace.entry_package().checked_type_map.clone();
        let mut next_global_index = 0;

        lower_global_declarations(typed_package, &mut lowered_package, &mut next_global_index)
            .expect("top-level globals should lower cleanly");

        assert_eq!(lowered_package.globals.len(), 2);
        let globals = lowered_package
            .globals
            .iter()
            .map(|id| {
                lowered_package
                    .global_decls
                    .get(id)
                    .expect("global id should resolve")
            })
            .collect::<Vec<_>>();
        assert_eq!(globals[0].name, "count");
        assert!(globals[0].mutable);
        assert_eq!(
            lowered_workspace.type_table().get(globals[0].type_id),
            Some(&LoweredType::Builtin(LoweredBuiltinType::Int))
        );
        assert_eq!(globals[1].name, "label");
        assert!(!globals[1].mutable);
        assert_eq!(
            lowered_workspace.type_table().get(globals[1].type_id),
            Some(&LoweredType::Builtin(LoweredBuiltinType::Str))
        );
    }

    #[test]
    fn declaration_lowering_records_routine_shells_with_parameter_locals() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_routines_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] add(left: int, right: int): int = { return left }",
        )
        .expect("should write lowering routine fixture");

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
        lowered_package.checked_type_map =
            lowered_workspace.entry_package().checked_type_map.clone();
        lower_routine_signatures(typed_package, &mut lowered_package)
            .expect("routine signatures should lower cleanly");
        let mut next_routine_index = 0;

        lower_routine_declarations(typed_package, &mut lowered_package, &mut next_routine_index)
            .expect("top-level routines should lower cleanly");

        assert_eq!(lowered_package.routines.len(), 1);
        let routine = lowered_package
            .routine_decls
            .get(&lowered_package.routines[0])
            .expect("routine id should resolve");
        assert_eq!(routine.name, "add");
        assert_eq!(routine.params.len(), 2);
        assert_eq!(routine.entry_block, crate::LoweredBlockId(0));
        assert_eq!(
            routine
                .params
                .iter()
                .map(|local_id| routine
                    .locals
                    .get(*local_id)
                    .and_then(|local| local.name.clone()))
                .collect::<Vec<_>>(),
            vec![Some("left".to_string()), Some("right".to_string())]
        );
    }

    #[test]
    fn declaration_lowering_reports_missing_parameter_symbol_matches_explicitly() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_missing_param_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(&fixture, "fun[] add(left: int): int = { return left }")
            .expect("should write lowering routine fixture");

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
        lowered_package.checked_type_map =
            lowered_workspace.entry_package().checked_type_map.clone();
        lower_routine_signatures(typed_package, &mut lowered_package)
            .expect("routine signatures should lower cleanly");

        let source_unit = typed_package
            .program
            .resolved()
            .syntax()
            .source_units
            .first()
            .expect("fixture source unit should exist");
        let AstNode::FunDecl {
            name, syntax_id, ..
        } = &source_unit.items[0].node
        else {
            panic!("fixture should retain one function declaration");
        };
        let symbol_id = super::find_local_symbol_id(
            &typed_package.program,
            fol_resolver::SourceUnitId(0),
            fol_resolver::SymbolKind::Routine,
            name,
        )
        .expect("routine symbol should exist");
        let mut next_routine_index = 0;
        let error = lower_routine_decl(
            typed_package,
            &lowered_package,
            symbol_id,
            fol_resolver::SourceUnitId(0),
            name,
            *syntax_id,
            &[Parameter {
                name: "missing".to_string(),
                param_type: FolType::Simple("int".to_string()),
                is_borrowable: false,
                is_mutex: false,
                default: None,
            }],
            &mut next_routine_index,
        )
        .expect_err("mismatched parameter names should stay explicit");

        assert_eq!(error.kind(), crate::LoweringErrorKind::InvalidInput);
        assert!(
            error
                .message()
                .contains("routine parameter 'missing' does not map to a resolved symbol in its lowered scope"),
            "missing parameter matches should report the lowered-scope guard explicitly, got: {error:?}",
        );
    }

    #[test]
    fn declaration_lowering_keeps_local_and_imported_packages_separate() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for tmp path")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("fol_lower_decl_parity_{stamp}"));
        let app_dir = root.join("app");
        let shared_dir = root.join("shared");
        fs::create_dir_all(&app_dir).expect("should create app dir");
        fs::create_dir_all(&shared_dir).expect("should create shared dir");
        fs::write(
            app_dir.join("main.fol"),
            "use shared: loc = {\"../shared\"}\nfun[] main(): int = { return answer() }",
        )
        .expect("should write app entry");
        fs::write(
            shared_dir.join("lib.fol"),
            "ali Counter: int\nvar[exp] base: Counter = 7\nfun[exp] answer(): Counter = { return base }",
        )
        .expect("should write shared library");

        let mut stream = FileStream::from_folder(app_dir.to_str().expect("utf8 temp path"))
            .expect("should open folder fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering folder fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering folder fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering folder fixture should typecheck");

        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("declaration lowering should succeed across imported packages");

        let app_package = lowered
            .packages()
            .find(|package| package.identity.display_name == "app")
            .expect("entry package should exist");
        let shared_package = lowered
            .packages()
            .find(|package| package.identity.display_name == "shared")
            .expect("imported package should exist");

        assert_eq!(app_package.global_decls.len(), 0);
        assert_eq!(app_package.type_decls.len(), 0);
        assert_eq!(app_package.routine_decls.len(), 1);
        assert_eq!(shared_package.type_decls.len(), 1);
        assert_eq!(shared_package.global_decls.len(), 1);
        assert_eq!(shared_package.routine_decls.len(), 1);
        assert!(
            app_package
                .symbol_ownership
                .values()
                .any(|ownership| ownership.mounted_from.is_some()),
            "entry package should retain mounted imported symbol ownership"
        );
        assert!(
            shared_package
                .symbol_ownership
                .values()
                .all(|ownership| ownership.mounted_from.is_none()),
            "owning package should not lower imported shells as local declarations"
        );
    }
}
