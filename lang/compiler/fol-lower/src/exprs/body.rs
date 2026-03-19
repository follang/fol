use super::bindings::lower_local_binding;
use super::calls::lower_keyword_intrinsic_statement;
use super::cursor::{RoutineCursor, WorkspaceDeclIndex};
use super::expressions::{lower_expression, lower_expression_expected};
use super::flow::{lower_loop_statement, lower_when_statement, when_always_terminates};
use crate::{
    ids::{LoweredTypeId},
    LoweredPackage, LoweredRoutine, LoweringError, LoweringErrorKind,
};
use fol_intrinsics::{select_intrinsic, IntrinsicSurface};
use fol_parser::ast::AstNode;
use fol_resolver::{PackageIdentity, ScopeId, SourceUnitId, SymbolKind};
use std::collections::BTreeMap;

pub(crate) fn lower_routine_bodies(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    decl_index: &WorkspaceDeclIndex,
    lowered_package: &mut LoweredPackage,
) -> Result<(), Vec<LoweringError>> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package
        .program
        .resolved()
        .syntax()
        .source_units
        .iter()
        .enumerate()
    {
        if source_unit.kind == fol_parser::ast::ParsedSourceUnitKind::Build {
            continue;
        }
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            let (name, syntax_id, body) = match &item.node {
                AstNode::FunDecl {
                    name,
                    syntax_id,
                    body,
                    ..
                }
                | AstNode::ProDecl {
                    name,
                    syntax_id,
                    body,
                    ..
                }
                | AstNode::LogDecl {
                    name,
                    syntax_id,
                    body,
                    ..
                } => (name.as_str(), *syntax_id, body.as_slice()),
                AstNode::Commented { node, .. } => match node.as_ref() {
                    AstNode::FunDecl {
                        name,
                        syntax_id,
                        body,
                        ..
                    }
                    | AstNode::ProDecl {
                        name,
                        syntax_id,
                        body,
                        ..
                    }
                    | AstNode::LogDecl {
                        name,
                        syntax_id,
                        body,
                        ..
                    } => (name.as_str(), *syntax_id, body.as_slice()),
                    _ => continue,
                },
                _ => continue,
            };
            let Some(symbol_id) = crate::decls::find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Routine,
                name,
            ) else {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("routine '{name}' does not retain a local lowering symbol"),
                ));
                continue;
            };
            let Some(scope_id) = syntax_id
                .and_then(|syntax_id| typed_package.program.resolved().scope_for_syntax(syntax_id))
            else {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("routine '{name}' does not retain typed scope information"),
                ));
                continue;
            };
            let Some(routine_id) =
                lowered_package
                    .routine_decls
                    .iter()
                    .find_map(|(routine_id, routine)| {
                        (routine.symbol_id == Some(symbol_id)).then_some(*routine_id)
                    })
            else {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("routine '{name}' does not map to a lowered routine shell"),
                ));
                continue;
            };
            let Some(routine) = lowered_package.routine_decls.get_mut(&routine_id) else {
                continue;
            };
            if let Err(error) = lower_body_nodes(
                typed_package,
                type_table,
                &lowered_package.checked_type_map,
                lowered_package.identity.clone(),
                decl_index,
                routine,
                source_unit_id,
                scope_id,
                body,
            ) {
                errors.push(error);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn lower_body_nodes(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    routine: &mut LoweredRoutine,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    nodes: &[AstNode],
) -> Result<(), LoweringError> {
    let entry_block = routine.entry_block;
    let mut cursor = RoutineCursor::new(routine, entry_block);
    cursor.routine.body_result = lower_body_sequence(
        typed_package,
        type_table,
        checked_type_map,
        &current_identity,
        decl_index,
        &mut cursor,
        source_unit_id,
        scope_id,
        nodes,
    )?
    .map(|value| value.local_id);

    Ok(())
}

pub(crate) fn lower_body_sequence(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    nodes: &[AstNode],
) -> Result<Option<super::cursor::LoweredValue>, LoweringError> {
    let mut final_value = None;

    for node in nodes {
        if let Some(value) = lower_body_node(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            node,
        )? {
            final_value = Some(value);
        }
        if cursor.current_block_terminated()? {
            break;
        }
    }

    Ok(final_value)
}

pub(crate) fn lower_body_node(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    node: &AstNode,
) -> Result<Option<super::cursor::LoweredValue>, LoweringError> {
    match node {
        AstNode::Comment { .. } => Ok(None),
        AstNode::Commented { node, .. } => lower_body_node(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            node,
        ),
        AstNode::VarDecl { name, value, .. } => lower_local_binding(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            name,
            SymbolKind::ValueBinding,
            value.as_deref(),
        ),
        AstNode::LabDecl { name, value, .. } => lower_local_binding(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            name,
            SymbolKind::LabelBinding,
            value.as_deref(),
        ),
        AstNode::Return { value } => match value.as_deref() {
            Some(value) => {
                let lowered = lower_expression_expected(
                    typed_package,
                    type_table,
                    checked_type_map,
                    current_identity,
                    decl_index,
                    cursor,
                    source_unit_id,
                    scope_id,
                    routine_return_type(cursor, type_table),
                    value,
                )?;
                cursor.terminate_current_block(crate::LoweredTerminator::Return {
                    value: Some(lowered.local_id),
                })?;
                Ok(None)
            }
            None => {
                cursor.terminate_current_block(crate::LoweredTerminator::Return { value: None })?;
                Ok(None)
            }
        },
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
        AstNode::FunctionCall {
            syntax_id,
            name,
            args,
            ..
        } => {
            if let Ok(entry) = select_intrinsic(IntrinsicSurface::KeywordCall, name) {
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
                    *syntax_id,
                    args,
                )
            } else {
                lower_expression(
                    typed_package,
                    type_table,
                    checked_type_map,
                    current_identity,
                    decl_index,
                    cursor,
                    source_unit_id,
                    scope_id,
                    node,
                )
                .map(Some)
            }
        }
        AstNode::When {
            expr,
            cases,
            default,
        } if default.is_none() || when_always_terminates(cases, default.as_deref()) => {
            lower_when_statement(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                expr,
                cases,
                default.as_deref(),
            )?;
            Ok(None)
        }
        AstNode::Loop { condition, body } => {
            lower_loop_statement(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                condition,
                body,
            )?;
            Ok(None)
        }
        AstNode::Break => {
            let Some(exit_block) = cursor.current_loop_exit() else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    "break lowering requires an active loop exit block",
                ));
            };
            cursor
                .terminate_current_block(crate::LoweredTerminator::Jump { target: exit_block })?;
            Ok(None)
        }
        AstNode::Yield { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "yield lowering is not part of the current V1 lowering milestone",
        )),
        _ => lower_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            node,
        )
        .map(Some),
    }
}

pub(crate) fn routine_return_type(
    cursor: &RoutineCursor<'_>,
    type_table: &crate::LoweredTypeTable,
) -> Option<LoweredTypeId> {
    let signature_id = cursor.routine.signature?;
    match type_table.get(signature_id) {
        Some(crate::LoweredType::Routine(signature)) => signature.return_type,
        _ => None,
    }
}

pub(crate) fn routine_error_type(
    cursor: &RoutineCursor<'_>,
    type_table: &crate::LoweredTypeTable,
) -> Option<LoweredTypeId> {
    let signature_id = cursor.routine.signature?;
    match type_table.get(signature_id) {
        Some(crate::LoweredType::Routine(signature)) => signature.error_type,
        _ => None,
    }
}
