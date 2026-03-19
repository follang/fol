use super::body::routine_error_type;
use super::cursor::{canonical_symbol_key, LoweredValue, RoutineCursor, WorkspaceDeclIndex};
use super::expressions::{lower_expression, lower_expression_expected, lower_expression_observed};
use crate::{
    control::LoweredInstrKind,
    ids::LoweredTypeId,
    LoweringError, LoweringErrorKind,
};
use fol_intrinsics::{select_intrinsic, IntrinsicEntry, IntrinsicSurface};
use fol_parser::ast::AstNode;
use fol_resolver::{PackageIdentity, ReferenceKind, ScopeId, SourceUnitId, SymbolId, SymbolKind};
use std::collections::BTreeMap;

pub(crate) fn lower_dot_intrinsic_call(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    name: &str,
    args: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    let syntax_id = syntax_id.ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("dot intrinsic '.{name}(...)' does not retain a syntax id"),
        )
    })?;
    let typed_node = typed_package.program.typed_node(syntax_id).ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("dot intrinsic '.{name}(...)' does not retain typed node facts"),
        )
    })?;
    let intrinsic_id = typed_node.intrinsic_id.ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("dot intrinsic '.{name}(...)' does not retain a selected intrinsic id"),
        )
    })?;
    let checked_result = typed_node.inferred_type.ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("dot intrinsic '.{name}(...)' does not retain a checked result type"),
        )
    })?;
    let result_type = checked_type_map
        .get(&checked_result)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("dot intrinsic '.{name}(...)' does not retain a lowered result type"),
            )
        })?;
    let lowered_args = args
        .iter()
        .map(|arg| {
            lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                arg,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let kind = match fol_intrinsics::lowering_mode_for_intrinsic(intrinsic_id) {
        Some(fol_intrinsics::IntrinsicLoweringMode::DedicatedIr) if name == "len" => {
            let [operand] = lowered_args.as_slice() else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("dot intrinsic '.{name}(...)' expected exactly 1 lowered operand"),
                ));
            };
            let result_local = cursor.allocate_local(result_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::LengthOf {
                    operand: operand.local_id,
                },
            )?;
            return Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
                recoverable_error_type: None,
            });
        }
        Some(fol_intrinsics::IntrinsicLoweringMode::RuntimeHook) if name == "echo" => {
            let [operand] = lowered_args.as_slice() else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("dot intrinsic '.{name}(...)' expected exactly 1 lowered operand"),
                ));
            };
            cursor.push_instr(
                None,
                LoweredInstrKind::RuntimeHook {
                    intrinsic: intrinsic_id,
                    args: vec![operand.local_id],
                },
            )?;
            return Ok(*operand);
        }
        _ => LoweredInstrKind::IntrinsicCall {
            intrinsic: intrinsic_id,
            args: lowered_args.iter().map(|value| value.local_id).collect(),
        },
    };
    let result_local = cursor.allocate_local(result_type, None);
    cursor.push_instr(Some(result_local), kind)?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: result_type,
        recoverable_error_type: None,
    })
}

pub(crate) fn lower_pipe_or_expression(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    left: &AstNode,
    right: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let observed_left = lower_expression_observed(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        None,
        left,
    )?;
    if observed_left.recoverable_error_type.is_none() {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "'||' lowering requires a recoverable expression on the left in V1",
        ));
    }
    let bool_type = checked_type_map
        .get(&typed_package.program.builtin_types().bool_)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                "lowered workspace lost builtin bool while lowering '||'",
            )
        })?;
    let condition_local = cursor.allocate_local(bool_type, None);
    cursor.push_instr(
        Some(condition_local),
        LoweredInstrKind::CheckRecoverable {
            operand: observed_left.local_id,
        },
    )?;

    let error_block = cursor.create_block();
    let success_block = cursor.create_block();
    let join_block = cursor.create_block();
    let result_local = cursor.allocate_local(observed_left.type_id, None);

    cursor.terminate_current_block(crate::LoweredTerminator::Branch {
        condition: condition_local,
        then_block: error_block,
        else_block: success_block,
    })?;

    cursor.switch_block(success_block)?;
    let success_value = cursor.allocate_local(observed_left.type_id, None);
    cursor.push_instr(
        Some(success_value),
        LoweredInstrKind::UnwrapRecoverable {
            operand: observed_left.local_id,
        },
    )?;
    cursor.push_instr(
        None,
        LoweredInstrKind::StoreLocal {
            local: result_local,
            value: success_value,
        },
    )?;
    cursor.terminate_current_block(crate::LoweredTerminator::Jump { target: join_block })?;

    cursor.switch_block(error_block)?;
    let fallback_value = lower_pipe_or_fallback(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        observed_left.type_id,
        right,
    )?;
    if let Some(fallback_value) = fallback_value {
        cursor.push_instr(
            None,
            LoweredInstrKind::StoreLocal {
                local: result_local,
                value: fallback_value.local_id,
            },
        )?;
        cursor.terminate_current_block(crate::LoweredTerminator::Jump { target: join_block })?;
    }

    cursor.switch_block(join_block)?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: observed_left.type_id,
        recoverable_error_type: None,
    })
}

fn lower_pipe_or_fallback(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expected_type: LoweredTypeId,
    right: &AstNode,
) -> Result<Option<LoweredValue>, LoweringError> {
    match right {
        AstNode::FunctionCall { name, args, .. } if name == "report" => {
            let lowered = match args.as_slice() {
                [value] => Some(
                    lower_expression_expected(
                        typed_package,
                        type_table,
                        checked_type_map,
                        current_identity,
                        decl_index,
                        cursor,
                        source_unit_id,
                        scope_id,
                        routine_error_type(cursor, type_table),
                        value,
                    )?
                    .local_id,
                ),
                [] => None,
                _ => {
                    return Err(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "report expects exactly 1 value in lowered V1 bodies, got {}",
                            args.len()
                        ),
                    ))
                }
            };
            cursor.terminate_current_block(crate::LoweredTerminator::Report { value: lowered })?;
            Ok(None)
        }
        AstNode::FunctionCall { name, args, .. } if name == "panic" => {
            let entry = select_intrinsic(IntrinsicSurface::KeywordCall, "panic").map_err(|_| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    "panic should remain a keyword intrinsic in lowered V1",
                )
            })?;
            lower_keyword_intrinsic_statement(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                entry,
                None,
                args,
            )
        }
        AstNode::Return { .. } => {
            use super::body::lower_body_node;
            let _ = lower_body_node(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                right,
            )?;
            Ok(None)
        }
        _ => lower_expression_expected(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            Some(expected_type),
            right,
        )
        .map(Some),
    }
}

pub(crate) fn lower_check_call(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    _syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    args: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    let [operand] = args else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "check expects exactly 1 value in lowered V1, got {}",
                args.len()
            ),
        ));
    };
    let observed = lower_expression_observed(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        None,
        operand,
    )?;
    if observed.recoverable_error_type.is_none() {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "check lowering requires a recoverable routine result operand in V1",
        ));
    }
    let bool_type = checked_type_map
        .get(&typed_package.program.builtin_types().bool_)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                "lowered workspace lost builtin bool while lowering check(...)",
            )
        })?;
    let result_local = cursor.allocate_local(bool_type, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::CheckRecoverable {
            operand: observed.local_id,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: bool_type,
        recoverable_error_type: None,
    })
}

pub(crate) fn lower_keyword_intrinsic_expression(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    entry: &IntrinsicEntry,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    args: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    match entry.name {
        "check" => lower_check_call(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            syntax_id,
            args,
        ),
        "panic" => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "panic lowering requires a statement or control-flow context in V1",
        )),
        other => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("unsupported keyword intrinsic expression '{other}(...)'"),
        )),
    }
}

pub(crate) fn lower_keyword_intrinsic_statement(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    entry: &IntrinsicEntry,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    args: &[AstNode],
) -> Result<Option<LoweredValue>, LoweringError> {
    match entry.name {
        "panic" => {
            lower_keyword_panic_terminator(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                args,
            )?;
            Ok(None)
        }
        "check" => lower_keyword_intrinsic_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            entry,
            syntax_id,
            args,
        )
        .map(Some),
        other => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("unsupported keyword intrinsic statement '{other}(...)'"),
        )),
    }
}

fn lower_keyword_panic_terminator(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    args: &[AstNode],
) -> Result<(), LoweringError> {
    let lowered = match args {
        [value] => Some(
            lower_expression_expected(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                None,
                value,
            )?
            .local_id,
        ),
        [] => None,
        _ => {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "panic expects at most 1 value in lowered V1, got {}",
                    args.len()
                ),
            ))
        }
    };
    cursor.terminate_current_block(crate::LoweredTerminator::Panic { value: lowered })?;
    Ok(())
}

pub(crate) fn lower_function_call(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    kind: ReferenceKind,
    display_name: &str,
    args: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    let resolved_symbol = resolve_reference_symbol(typed_package, syntax_id, kind, display_name)?;
    let (owning_identity, owning_symbol_id) = canonical_symbol_key(
        current_identity,
        resolved_symbol.mounted_from.as_ref(),
        resolved_symbol.id,
    );
    let Some(callee) = decl_index.routine_id_for_symbol(&owning_identity, owning_symbol_id) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("call target '{display_name}' does not map to a lowered routine definition"),
        ));
    };
    let Some(result_type) =
        resolve_reference_type_id(typed_package, checked_type_map, syntax_id, kind)
    else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            format!(
                "procedure-style calls without a value result are not lowered in this slice yet: '{display_name}'"
            ),
        ));
    };
    let param_types = decl_index
        .routine_param_types(callee)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("call target '{display_name}' does not retain lowered parameter types"),
            )
        })?
        .to_vec();
    let lowered_args = args
        .iter()
        .enumerate()
        .map(|(index, arg)| {
            let expected = param_types.get(index).copied();
            lower_expression_expected(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                expected,
                arg,
            )
            .map(|value| value.local_id)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let call_error_type =
        lowered_symbol_error_type(typed_package, checked_type_map, resolved_symbol.id);
    let result_local = cursor.allocate_local(result_type, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::Call {
            callee,
            args: lowered_args,
            error_type: call_error_type,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: result_type,
        recoverable_error_type: call_error_type,
    })
}

pub(crate) fn resolve_reference_type_id(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    kind: ReferenceKind,
) -> Option<LoweredTypeId> {
    let syntax_id = syntax_id?;
    let reference = typed_package
        .program
        .resolved()
        .references
        .iter()
        .find(|reference| reference.syntax_id == Some(syntax_id) && reference.kind == kind)?;
    let checked_type = reference_type_id(typed_package, reference.id)?;
    checked_type_map.get(&checked_type).copied()
}

pub(crate) fn reference_type_id(
    typed_package: &fol_typecheck::TypedPackage,
    reference_id: fol_resolver::ReferenceId,
) -> Option<fol_typecheck::CheckedTypeId> {
    typed_package
        .program
        .typed_reference(reference_id)?
        .resolved_type
}

fn lowered_symbol_error_type(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    symbol_id: SymbolId,
) -> Option<LoweredTypeId> {
    let declared_type = typed_package
        .program
        .typed_symbol(symbol_id)?
        .declared_type?;
    let fol_typecheck::CheckedType::Routine(signature) =
        typed_package.program.type_table().get(declared_type)?
    else {
        return None;
    };
    signature
        .error_type
        .and_then(|error_type| checked_type_map.get(&error_type).copied())
}

pub(crate) fn resolve_method_target(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    method: &str,
    receiver_type: LoweredTypeId,
) -> Result<(crate::LoweredRoutineId, LoweredTypeId, Option<LoweredTypeId>), LoweringError> {
    let mut matches = Vec::new();

    for (symbol_id, symbol) in typed_package.program.resolved().symbols.iter_with_ids() {
        if symbol.kind != SymbolKind::Routine || symbol.name != method {
            continue;
        }
        let Some(typed_symbol) = typed_package.program.typed_symbol(symbol_id) else {
            continue;
        };
        let Some(receiver_checked_type) = typed_symbol.receiver_type else {
            continue;
        };
        let Some(lowered_receiver_type) = checked_type_map.get(&receiver_checked_type).copied()
        else {
            continue;
        };
        if lowered_receiver_type != receiver_type {
            continue;
        }

        let (owning_identity, owning_symbol_id) =
            canonical_symbol_key(current_identity, symbol.mounted_from.as_ref(), symbol_id);
        let Some(routine_id) = decl_index.routine_id_for_symbol(&owning_identity, owning_symbol_id)
        else {
            continue;
        };
        let Some(signature_checked_type) = typed_symbol.declared_type else {
            continue;
        };
        let Some(fol_typecheck::CheckedType::Routine(signature)) = typed_package
            .program
            .type_table()
            .get(signature_checked_type)
        else {
            continue;
        };
        let Some(return_type) = signature
            .return_type
            .and_then(|return_type| checked_type_map.get(&return_type).copied())
        else {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::Unsupported,
                format!(
                    "procedure-style method calls without a value result are not lowered in this slice yet: '{method}'"
                ),
            ));
        };
        let error_type = signature
            .error_type
            .and_then(|error_type| checked_type_map.get(&error_type).copied());
        matches.push((routine_id, return_type, error_type));
    }

    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("method '{method}' is not available for the lowered receiver type"),
        )),
        _ => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("method '{method}' is ambiguous for the lowered receiver type"),
        )),
    }
}

pub(crate) fn resolve_reference_symbol<'a>(
    typed_package: &'a fol_typecheck::TypedPackage,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    kind: ReferenceKind,
    display_name: &str,
) -> Result<&'a fol_resolver::ResolvedSymbol, LoweringError> {
    let syntax_id = syntax_id.ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("reference '{display_name}' does not retain a syntax id"),
        )
    })?;
    let Some(reference) = typed_package
        .program
        .resolved()
        .references
        .iter()
        .find(|reference| reference.syntax_id == Some(syntax_id) && reference.kind == kind)
    else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("reference '{display_name}' is missing from resolver output"),
        ));
    };
    let Some(symbol_id) = reference.resolved else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("reference '{display_name}' does not resolve to a lowered symbol"),
        ));
    };
    typed_package
        .program
        .resolved()
        .symbol(symbol_id)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("reference '{display_name}' lost its resolved symbol"),
            )
        })
}
