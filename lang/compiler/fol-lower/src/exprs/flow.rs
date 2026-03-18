use super::body::lower_body_sequence;
use super::cursor::{LoweredValue, RoutineCursor, WorkspaceDeclIndex};
use super::expressions::lower_expression_observed;
use crate::{
    control::LoweredInstrKind,
    ids::{LoweredBlockId, LoweredTypeId},
    LoweringError, LoweringErrorKind,
};
use fol_parser::ast::{AstNode, LoopCondition};
use fol_resolver::{PackageIdentity, ScopeId, SourceUnitId};
use std::collections::BTreeMap;

pub(crate) fn when_case_body(case: &fol_parser::ast::WhenCase) -> &[AstNode] {
    match case {
        fol_parser::ast::WhenCase::Case { body, .. }
        | fol_parser::ast::WhenCase::Is { body, .. }
        | fol_parser::ast::WhenCase::In { body, .. }
        | fol_parser::ast::WhenCase::Has { body, .. }
        | fol_parser::ast::WhenCase::On { body, .. }
        | fol_parser::ast::WhenCase::Of { body, .. } => body.as_slice(),
    }
}

pub(crate) fn when_always_terminates(
    cases: &[fol_parser::ast::WhenCase],
    default: Option<&[AstNode]>,
) -> bool {
    let Some(default) = default else {
        return false;
    };
    !cases.is_empty()
        && cases
            .iter()
            .all(|case| body_always_terminates(when_case_body(case)))
        && body_always_terminates(default)
}

fn body_always_terminates(nodes: &[AstNode]) -> bool {
    nodes
        .iter()
        .rev()
        .find(|node| !matches!(node, AstNode::Comment { .. }))
        .is_some_and(node_always_terminates)
}

fn node_always_terminates(node: &AstNode) -> bool {
    match node {
        AstNode::Comment { .. } => false,
        AstNode::Commented { node, .. } => node_always_terminates(node),
        AstNode::Return { .. } => true,
        AstNode::FunctionCall { name, .. } if name == "report" => true,
        AstNode::Block { statements } => body_always_terminates(statements),
        AstNode::When { cases, default, .. } => when_always_terminates(cases, default.as_deref()),
        _ => false,
    }
}

pub(crate) fn lower_when_statement(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expr: &AstNode,
    cases: &[fol_parser::ast::WhenCase],
    default: Option<&[AstNode]>,
) -> Result<(), LoweringError> {
    use super::expressions::lower_expression;
    let subject = lower_expression(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        expr,
    )?;

    let mut after_block = None;
    let mut has_fallthrough = false;

    for (index, case) in cases.iter().enumerate() {
        let (condition, body) = when_case_condition_and_body(case)?;
        let lowered_condition = lower_when_case_condition(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            &subject,
            condition,
        )?;
        let body_block = cursor.create_block();
        let else_block = if index + 1 < cases.len() || default.is_some() {
            cursor.create_block()
        } else {
            ensure_after_block(cursor, &mut after_block)
        };
        cursor.terminate_current_block(crate::LoweredTerminator::Branch {
            condition: lowered_condition.local_id,
            then_block: body_block,
            else_block,
        })?;

        cursor.switch_block(body_block)?;
        let _ = lower_body_sequence(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            body,
        )?;
        if !cursor.current_block_terminated()? {
            let after_block = ensure_after_block(cursor, &mut after_block);
            cursor.terminate_current_block(crate::LoweredTerminator::Jump {
                target: after_block,
            })?;
            has_fallthrough = true;
        }

        if Some(else_block) != after_block {
            cursor.switch_block(else_block)?;
        }
    }

    if let Some(default) = default {
        has_fallthrough |= lower_default_when_body(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            default,
            &mut after_block,
        )?;
    }

    if let Some(after_block) = after_block.filter(|_| has_fallthrough) {
        cursor.switch_block(after_block)?;
    }

    Ok(())
}

pub(crate) fn lower_loop_statement(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    condition: &LoopCondition,
    body: &[AstNode],
) -> Result<(), LoweringError> {
    use super::expressions::lower_expression;
    match condition {
        LoopCondition::Condition(condition) => {
            let header_block = cursor.create_block();
            let body_block = cursor.create_block();
            let exit_block = cursor.create_block();

            cursor.terminate_current_block(crate::LoweredTerminator::Jump {
                target: header_block,
            })?;

            cursor.switch_block(header_block)?;
            let lowered_condition = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                condition,
            )?;
            cursor.terminate_current_block(crate::LoweredTerminator::Branch {
                condition: lowered_condition.local_id,
                then_block: body_block,
                else_block: exit_block,
            })?;

            cursor.switch_block(body_block)?;
            cursor.push_loop_exit(exit_block);
            let _ = lower_body_sequence(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                body,
            )?;
            cursor.pop_loop_exit();
            if !cursor.current_block_terminated()? {
                cursor.terminate_current_block(crate::LoweredTerminator::Jump {
                    target: header_block,
                })?;
            }

            cursor.switch_block(exit_block)?;
            Ok(())
        }
        LoopCondition::Iteration { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "iteration loop lowering is not part of the current lowered V1 control-flow milestone",
        )),
    }
}

fn lower_default_when_body(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    default: &[AstNode],
    after_block: &mut Option<LoweredBlockId>,
) -> Result<bool, LoweringError> {
    let _ = lower_body_sequence(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        default,
    )?;
    if !cursor.current_block_terminated()? {
        let after_block = ensure_after_block(cursor, after_block);
        cursor.terminate_current_block(crate::LoweredTerminator::Jump {
            target: after_block,
        })?;
        return Ok(true);
    }
    Ok(false)
}

fn ensure_after_block(
    cursor: &mut RoutineCursor<'_>,
    after_block: &mut Option<LoweredBlockId>,
) -> LoweredBlockId {
    *after_block.get_or_insert_with(|| cursor.create_block())
}

pub(crate) fn lower_when_expression(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expr: &AstNode,
    cases: &[fol_parser::ast::WhenCase],
    default: Option<&[AstNode]>,
) -> Result<LoweredValue, LoweringError> {
    use super::expressions::lower_expression;
    let Some(default) = default else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "value-producing when expressions require a default branch in lowered V1",
        ));
    };

    let subject = lower_expression(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        expr,
    )?;

    let join_block = cursor.create_block();
    let mut join_local = None;

    for (index, case) in cases.iter().enumerate() {
        let (condition, body) = when_case_condition_and_body(case)?;
        let lowered_condition = lower_when_case_condition(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            &subject,
            condition,
        )?;
        let body_block = cursor.create_block();
        let else_block = if index + 1 < cases.len() || !default.is_empty() {
            cursor.create_block()
        } else {
            join_block
        };
        cursor.terminate_current_block(crate::LoweredTerminator::Branch {
            condition: lowered_condition.local_id,
            then_block: body_block,
            else_block,
        })?;

        cursor.switch_block(body_block)?;
        let branch_value = lower_body_sequence(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            body,
        )?;
        lower_when_branch_value(cursor, &mut join_local, branch_value, join_block)?;

        if else_block != join_block {
            cursor.switch_block(else_block)?;
        }
    }

    let default_value = lower_body_sequence(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        default,
    )?;
    lower_when_branch_value(cursor, &mut join_local, default_value, join_block)?;

    cursor.switch_block(join_block)?;
    join_local.ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "value-producing when did not retain a lowered join value",
        )
    })
}

fn lower_when_case_condition(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    subject: &LoweredValue,
    condition: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let lowered_condition = lower_expression_observed(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        Some(subject.type_id),
        condition,
    )?;
    if subject.type_id != lowered_condition.type_id {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "when case condition type {} does not match subject type {} in lowered V1",
                lowered_condition.type_id.0, subject.type_id.0
            ),
        ));
    }
    let bool_type = checked_type_map
        .get(&typed_package.program.builtin_types().bool_)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                "lowered workspace lost builtin bool while lowering when conditions",
            )
        })?;
    let eq_intrinsic = fol_intrinsics::intrinsic_by_canonical_name("eq")
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                "intrinsic registry lost '.eq(...)' while lowering when conditions",
            )
        })?
        .id;
    let result_local = cursor.allocate_local(bool_type, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::IntrinsicCall {
            intrinsic: eq_intrinsic,
            args: vec![subject.local_id, lowered_condition.local_id],
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: bool_type,
        recoverable_error_type: None,
    })
}

fn lower_when_branch_value(
    cursor: &mut RoutineCursor<'_>,
    join_local: &mut Option<LoweredValue>,
    branch_value: Option<LoweredValue>,
    join_block: LoweredBlockId,
) -> Result<(), LoweringError> {
    match branch_value {
        Some(branch_value) => {
            let destination = if let Some(existing) = join_local {
                if existing.type_id != branch_value.type_id {
                    return Err(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        "value-producing when branches do not agree on one lowered join type",
                    ));
                }
                *existing
            } else {
                let local_id = cursor.allocate_local(branch_value.type_id, None);
                let value = LoweredValue {
                    local_id,
                    type_id: branch_value.type_id,
                    recoverable_error_type: None,
                };
                *join_local = Some(value);
                value
            };
            cursor.push_instr(
                None,
                crate::control::LoweredInstrKind::StoreLocal {
                    local: destination.local_id,
                    value: branch_value.local_id,
                },
            )?;
            if !cursor.current_block_terminated()? {
                cursor.terminate_current_block(crate::LoweredTerminator::Jump {
                    target: join_block,
                })?;
            }
            Ok(())
        }
        None if cursor.current_block_terminated()? => Ok(()),
        None => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "value-producing when branches must yield a value or terminate early",
        )),
    }
}

pub(crate) fn when_case_condition_and_body(
    case: &fol_parser::ast::WhenCase,
) -> Result<(&AstNode, &[AstNode]), LoweringError> {
    match case {
        fol_parser::ast::WhenCase::Case { condition, body }
        | fol_parser::ast::WhenCase::Is {
            value: condition,
            body,
        }
        | fol_parser::ast::WhenCase::In {
            range: condition,
            body,
        }
        | fol_parser::ast::WhenCase::Has {
            member: condition,
            body,
        }
        | fol_parser::ast::WhenCase::On {
            channel: condition,
            body,
        } => Ok((condition, body.as_slice())),
        fol_parser::ast::WhenCase::Of { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "type-matching when/of branches are not lowered in this slice yet",
        )),
    }
}
