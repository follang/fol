use super::calls::{
    lower_dot_intrinsic_call, lower_function_call, lower_keyword_intrinsic_expression,
    lower_pipe_or_expression, reference_type_id, resolve_method_target,
};
use super::containers::{
    apply_expected_shell_wrap, field_access_type, index_access_type, lower_container_literal,
    lower_nil_literal, lower_record_initializer, slice_access_type,
};
use super::cursor::{LoweredValue, RoutineCursor, WorkspaceDeclIndex};
use super::flow::lower_when_expression;
use super::helpers::{
    describe_binary_operator, describe_unary_operator,
    literal_type_id, lower_assignment_target, lower_entry_variant_access, lower_unwrap_expression,
};
use crate::{
    control::{LoweredBinaryOp, LoweredInstrKind, LoweredUnaryOp},
    ids::LoweredTypeId,
    LoweringError, LoweringErrorKind,
};
use fol_intrinsics::{select_intrinsic, IntrinsicSurface};
use fol_parser::ast::{AstNode, CallSurface, Literal};
use fol_resolver::{PackageIdentity, ReferenceKind, ScopeId, SourceUnitId};
use std::collections::BTreeMap;

pub(crate) fn lower_expression(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    node: &AstNode,
) -> Result<LoweredValue, LoweringError> {
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
        node,
    )
}

pub(crate) fn lower_expression_expected(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expected_type: Option<LoweredTypeId>,
    node: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let lowered = lower_expression_observed(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        expected_type,
        node,
    )?;
    if let Some(error_type) = lowered.recoverable_error_type {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "recoverable value with lowered error type {} cannot enter plain expected lowering; handle it with '||' or check(...)",
                error_type.0
            ),
        ));
    }
    apply_expected_shell_wrap(type_table, cursor, expected_type, lowered)
}

pub(crate) fn lower_expression_observed(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expected_type: Option<LoweredTypeId>,
    node: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let lowered = match node {
        AstNode::Literal(Literal::Nil) => lower_nil_literal(type_table, cursor, expected_type),
        AstNode::Literal(literal) => {
            let type_id =
                literal_type_id(typed_package, checked_type_map, literal).ok_or_else(|| {
                    LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        "literal expression does not retain a lowering-owned type",
                    )
                })?;
            cursor.lower_literal(literal, type_id)
        }
        AstNode::UnaryOp {
            op: fol_parser::ast::UnaryOperator::Unwrap,
            operand,
        } => lower_unwrap_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            operand,
        ),
        AstNode::UnaryOp {
            op: fol_parser::ast::UnaryOperator::Neg,
            operand,
        } => lower_unary_op(
            typed_package, type_table, checked_type_map, current_identity,
            decl_index, cursor, source_unit_id, scope_id, LoweredUnaryOp::Neg, operand,
        ),
        AstNode::UnaryOp {
            op: fol_parser::ast::UnaryOperator::Not,
            operand,
        } => lower_unary_op(
            typed_package, type_table, checked_type_map, current_identity,
            decl_index, cursor, source_unit_id, scope_id, LoweredUnaryOp::Not, operand,
        ),
        AstNode::UnaryOp { op, .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            format!(
                "unary operator lowering for '{}' is deferred beyond V1",
                describe_unary_operator(op)
            ),
        )),
        AstNode::BinaryOp {
            op: fol_parser::ast::BinaryOperator::PipeOr,
            left,
            right,
        } => lower_pipe_or_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            left,
            right,
        ),
        AstNode::BinaryOp { op, left, right } => {
            let lowered_op = match op {
                fol_parser::ast::BinaryOperator::Add => LoweredBinaryOp::Add,
                fol_parser::ast::BinaryOperator::Sub => LoweredBinaryOp::Sub,
                fol_parser::ast::BinaryOperator::Mul => LoweredBinaryOp::Mul,
                fol_parser::ast::BinaryOperator::Div => LoweredBinaryOp::Div,
                fol_parser::ast::BinaryOperator::Mod => LoweredBinaryOp::Mod,
                fol_parser::ast::BinaryOperator::Pow => LoweredBinaryOp::Pow,
                fol_parser::ast::BinaryOperator::Eq => LoweredBinaryOp::Eq,
                fol_parser::ast::BinaryOperator::Ne => LoweredBinaryOp::Ne,
                fol_parser::ast::BinaryOperator::Lt => LoweredBinaryOp::Lt,
                fol_parser::ast::BinaryOperator::Le => LoweredBinaryOp::Le,
                fol_parser::ast::BinaryOperator::Gt => LoweredBinaryOp::Gt,
                fol_parser::ast::BinaryOperator::Ge => LoweredBinaryOp::Ge,
                fol_parser::ast::BinaryOperator::And => LoweredBinaryOp::And,
                fol_parser::ast::BinaryOperator::Or => LoweredBinaryOp::Or,
                fol_parser::ast::BinaryOperator::Xor => LoweredBinaryOp::Xor,
                other => {
                    return Err(LoweringError::with_kind(
                        LoweringErrorKind::Unsupported,
                        format!(
                            "binary operator lowering for '{}' is deferred beyond V1",
                            describe_binary_operator(other)
                        ),
                    ));
                }
            };
            lower_binary_op(
                typed_package, type_table, checked_type_map, current_identity,
                decl_index, cursor, source_unit_id, scope_id, lowered_op, left, right,
            )
        }
        AstNode::RecordInit { fields, .. } => lower_record_initializer(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            expected_type,
            fields,
        ),
        AstNode::ContainerLiteral {
            container_type,
            elements,
        } => lower_container_literal(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            container_type.clone(),
            expected_type,
            elements,
        ),
        AstNode::Assignment { target, value } => {
            let lowered_value = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                value,
            )?;
            lower_assignment_target(
                typed_package,
                current_identity,
                decl_index,
                cursor,
                target,
                lowered_value,
            )
        }
        AstNode::FunctionCall {
            surface: CallSurface::DotIntrinsic,
            syntax_id,
            name,
            args,
            ..
        } => lower_dot_intrinsic_call(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            *syntax_id,
            name,
            args,
        ),
        AstNode::FunctionCall {
            syntax_id,
            name,
            args,
            ..
        } => {
            if let Ok(entry) = select_intrinsic(IntrinsicSurface::KeywordCall, name) {
                lower_keyword_intrinsic_expression(
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
                lower_function_call(
                    typed_package,
                    type_table,
                    checked_type_map,
                    current_identity,
                    decl_index,
                    cursor,
                    source_unit_id,
                    scope_id,
                    *syntax_id,
                    ReferenceKind::FunctionCall,
                    name,
                    args,
                )
            }
        }
        AstNode::QualifiedFunctionCall { path, args } => lower_function_call(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            path.syntax_id(),
            ReferenceKind::QualifiedFunctionCall,
            &path.joined(),
            args,
        ),
        AstNode::MethodCall {
            object,
            method,
            args,
        } => {
            let receiver = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                object,
            )?;
            let (callee, result_type, error_type) = resolve_method_target(
                typed_package,
                checked_type_map,
                current_identity,
                decl_index,
                method,
                receiver.type_id,
            )?;
            let mut lowered_args = vec![receiver.local_id];
            let param_types = decl_index
                .routine_param_types(callee)
                .ok_or_else(|| {
                    LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!("method '{method}' does not retain lowered parameter types"),
                    )
                })?
                .to_vec();
            lowered_args.extend(
                args.iter()
                    .enumerate()
                    .map(|(index, arg)| {
                        let expected = param_types.get(index + 1).copied();
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
                    .collect::<Result<Vec<_>, _>>()?,
            );
            let result_local = cursor.allocate_local(result_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::Call {
                    callee,
                    args: lowered_args,
                    error_type,
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
                recoverable_error_type: error_type,
            })
        }
        AstNode::FieldAccess { object, field } => {
            if let Some(entry_value) = lower_entry_variant_access(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                object,
                field,
                expected_type,
            )? {
                return apply_expected_shell_wrap(type_table, cursor, expected_type, entry_value);
            }
            let base = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                object,
            )?;
            if let Some(expected_type) = expected_type {
                if base.type_id == expected_type
                    && matches!(type_table.get(base.type_id), Some(crate::LoweredType::Entry { variants }) if variants.contains_key(field))
                {
                    return Err(LoweringError::with_kind(
                        LoweringErrorKind::Unsupported,
                        format!(
                            "entry construction lowering for variant '{}' lands in the pending aggregate slice",
                            field
                        ),
                    ));
                }
            }
            let Some(result_type) = field_access_type(type_table, base.type_id, field) else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("field access '.{field}' does not map to a lowered record field"),
                ));
            };
            let result_local = cursor.allocate_local(result_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::FieldAccess {
                    base: base.local_id,
                    field: field.clone(),
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
                recoverable_error_type: None,
            })
        }
        AstNode::IndexAccess { container, index } => {
            let lowered_container = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                container,
            )?;
            let lowered_index = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                index,
            )?;
            let Some(result_type) = index_access_type(type_table, lowered_container.type_id, index)
            else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    "index access does not map to a lowered container element type",
                ));
            };
            let result_local = cursor.allocate_local(result_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::IndexAccess {
                    container: lowered_container.local_id,
                    index: lowered_index.local_id,
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
                recoverable_error_type: None,
            })
        }
        AstNode::When {
            expr,
            cases,
            default,
        } => lower_when_expression(
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
        ),
        AstNode::Identifier { syntax_id, name } => {
            let syntax_id = syntax_id.ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' does not retain a syntax id"),
                )
            })?;
            let Some(reference) =
                typed_package
                    .program
                    .resolved()
                    .references
                    .iter()
                    .find(|reference| {
                        reference.syntax_id == Some(syntax_id)
                            && reference.kind == ReferenceKind::Identifier
                    })
            else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' is missing from resolver output"),
                ));
            };
            let Some(symbol_id) = reference.resolved else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' does not resolve to a lowered symbol"),
                ));
            };
            let resolved_symbol = typed_package
                .program
                .resolved()
                .symbol(symbol_id)
                .ok_or_else(|| {
                    LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!("identifier '{name}' lost its resolved symbol"),
                    )
                })?;
            let result_type = reference_type_id(typed_package, reference.id).ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' does not retain a lowered reference type"),
                )
            })?;
            let result_type = checked_type_map.get(&result_type).copied().ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' does not retain a lowered reference type"),
                )
            })?;
            cursor.lower_identifier_reference(
                current_identity,
                decl_index,
                resolved_symbol,
                result_type,
            )
        }
        AstNode::QualifiedIdentifier { path } => {
            let syntax_id = path.syntax_id().ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "qualified identifier '{}' does not retain a syntax id",
                        path.joined()
                    ),
                )
            })?;
            let Some(reference) =
                typed_package
                    .program
                    .resolved()
                    .references
                    .iter()
                    .find(|reference| {
                        reference.syntax_id == Some(syntax_id)
                            && reference.kind == ReferenceKind::QualifiedIdentifier
                    })
            else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "qualified identifier '{}' is missing from resolver output",
                        path.joined()
                    ),
                ));
            };
            let Some(symbol_id) = reference.resolved else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "qualified identifier '{}' does not resolve to a lowered symbol",
                        path.joined()
                    ),
                ));
            };
            let resolved_symbol = typed_package
                .program
                .resolved()
                .symbol(symbol_id)
                .ok_or_else(|| {
                    LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "qualified identifier '{}' lost its resolved symbol",
                            path.joined()
                        ),
                    )
                })?;
            let result_type = reference_type_id(typed_package, reference.id).ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "qualified identifier '{}' does not retain a lowered reference type",
                        path.joined()
                    ),
                )
            })?;
            let result_type = checked_type_map.get(&result_type).copied().ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "qualified identifier '{}' does not retain a lowered reference type",
                        path.joined()
                    ),
                )
            })?;
            cursor.lower_identifier_reference(
                current_identity,
                decl_index,
                resolved_symbol,
                result_type,
            )
        }
        AstNode::Commented { node, .. } => lower_expression_expected(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            expected_type,
            node,
        ),
        // V1 pipeline — not yet implemented (Phase 1.3, 1.4)
        AstNode::Invoke { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "invoke expression lowering is not yet implemented",
        )),
        AstNode::AnonymousFun { .. }
        | AstNode::AnonymousPro { .. }
        | AstNode::AnonymousLog { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "anonymous routine lowering is not yet implemented",
        )),
        // V1 pipeline gaps (Phase 3)
        AstNode::TemplateCall { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "template call lowering is not yet implemented",
        )),
        AstNode::AvailabilityAccess { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "availability access lowering is not yet implemented",
        )),
        AstNode::SliceAccess {
            container,
            start,
            end,
            ..
        } => {
            let lowered_container = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                container,
            )?;
            let lowered_start = if let Some(start) = start {
                lower_expression(
                    typed_package,
                    type_table,
                    checked_type_map,
                    current_identity,
                    decl_index,
                    cursor,
                    source_unit_id,
                    scope_id,
                    start,
                )?
            } else {
                let int_type = literal_type_id(typed_package, checked_type_map, &Literal::Integer(0))
                    .ok_or_else(|| {
                        LoweringError::with_kind(
                            LoweringErrorKind::InvalidInput,
                            "int type not found for slice default start bound",
                        )
                    })?;
                let zero_local = cursor.allocate_local(int_type, None);
                cursor.push_instr(
                    Some(zero_local),
                    LoweredInstrKind::Const(crate::control::LoweredOperand::Int(0)),
                )?;
                LoweredValue {
                    local_id: zero_local,
                    type_id: int_type,
                    recoverable_error_type: None,
                }
            };
            let lowered_end = if let Some(end) = end {
                lower_expression(
                    typed_package,
                    type_table,
                    checked_type_map,
                    current_identity,
                    decl_index,
                    cursor,
                    source_unit_id,
                    scope_id,
                    end,
                )?
            } else {
                let int_type = literal_type_id(typed_package, checked_type_map, &Literal::Integer(0))
                    .ok_or_else(|| {
                        LoweringError::with_kind(
                            LoweringErrorKind::InvalidInput,
                            "int type not found for slice default end bound",
                        )
                    })?;
                let len_local = cursor.allocate_local(int_type, None);
                cursor.push_instr(
                    Some(len_local),
                    LoweredInstrKind::LengthOf {
                        operand: lowered_container.local_id,
                    },
                )?;
                LoweredValue {
                    local_id: len_local,
                    type_id: int_type,
                    recoverable_error_type: None,
                }
            };
            let Some(result_type) =
                slice_access_type(type_table, lowered_container.type_id)
            else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    "slice access does not map to a lowered container type",
                ));
            };
            let result_local = cursor.allocate_local(result_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::SliceAccess {
                    container: lowered_container.local_id,
                    start: lowered_start.local_id,
                    end: lowered_end.local_id,
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
                recoverable_error_type: None,
            })
        }
        AstNode::Loop { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "loop lowering is not yet implemented",
        )),
        AstNode::Block { statements: _ } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "block expression lowering is not yet implemented",
        )),
        // Beyond V1 — deferred
        AstNode::AsyncStage
        | AstNode::AwaitStage
        | AstNode::Spawn { .. }
        | AstNode::ChannelAccess { .. }
        | AstNode::Select { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "concurrency primitives (async, await, spawn, channels, select) are deferred beyond V1",
        )),
        AstNode::Rolling { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "rolling/comprehension expressions are deferred beyond V1",
        )),
        AstNode::Range { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "range expressions are deferred beyond V1",
        )),
        AstNode::Yield { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "yield expressions are deferred beyond V1",
        )),
        AstNode::PatternAccess { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "pattern access is deferred beyond V1",
        )),
        // Structural nodes consumed by parent lowering
        AstNode::NamedArgument { .. } | AstNode::Unpack { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "named arguments and unpacks should be consumed by call-site lowering",
        )),
        AstNode::PatternWildcard | AstNode::PatternCapture { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "pattern elements should be consumed by pattern matching lowering",
        )),
        // Statement nodes in expression position
        AstNode::Return { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "return statement should not appear in expression lowering",
        )),
        AstNode::Break => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "break statement should not appear in expression lowering",
        )),
        AstNode::Inquiry { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "inquiry clause should not appear in expression lowering",
        )),
        // Declaration nodes should never appear in expression position
        AstNode::VarDecl { .. }
        | AstNode::DestructureDecl { .. }
        | AstNode::FunDecl { .. }
        | AstNode::ProDecl { .. }
        | AstNode::LogDecl { .. }
        | AstNode::TypeDecl { .. }
        | AstNode::UseDecl { .. }
        | AstNode::AliasDecl { .. }
        | AstNode::DefDecl { .. }
        | AstNode::SegDecl { .. }
        | AstNode::ImpDecl { .. }
        | AstNode::StdDecl { .. }
        | AstNode::LabDecl { .. }
        | AstNode::Comment { .. }
        | AstNode::Program { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "declaration node should not appear in expression lowering",
        )),
    }?;
    apply_expected_shell_wrap(type_table, cursor, expected_type, lowered)
}

fn binary_op_result_type(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    op: LoweredBinaryOp,
    left_type: LoweredTypeId,
) -> Option<LoweredTypeId> {
    match op {
        LoweredBinaryOp::Add
        | LoweredBinaryOp::Sub
        | LoweredBinaryOp::Mul
        | LoweredBinaryOp::Div
        | LoweredBinaryOp::Mod
        | LoweredBinaryOp::Pow => Some(left_type),
        LoweredBinaryOp::Eq
        | LoweredBinaryOp::Ne
        | LoweredBinaryOp::Lt
        | LoweredBinaryOp::Le
        | LoweredBinaryOp::Gt
        | LoweredBinaryOp::Ge
        | LoweredBinaryOp::And
        | LoweredBinaryOp::Or
        | LoweredBinaryOp::Xor => {
            checked_type_map.get(&typed_package.program.builtin_types().bool_).copied()
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_binary_op(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    op: LoweredBinaryOp,
    left: &AstNode,
    right: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let left_val = lower_expression(
        typed_package, type_table, checked_type_map, current_identity,
        decl_index, cursor, source_unit_id, scope_id, left,
    )?;
    let right_val = lower_expression(
        typed_package, type_table, checked_type_map, current_identity,
        decl_index, cursor, source_unit_id, scope_id, right,
    )?;
    let result_type = binary_op_result_type(typed_package, checked_type_map, op, left_val.type_id)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                "binary operator result type could not be resolved in the lowered type table",
            )
        })?;
    let result_local = cursor.allocate_local(result_type, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::BinaryOp {
            op,
            left: left_val.local_id,
            right: right_val.local_id,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: result_type,
        recoverable_error_type: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn lower_unary_op(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    op: LoweredUnaryOp,
    operand: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let operand_val = lower_expression(
        typed_package, type_table, checked_type_map, current_identity,
        decl_index, cursor, source_unit_id, scope_id, operand,
    )?;
    let result_type = match op {
        LoweredUnaryOp::Neg => operand_val.type_id,
        LoweredUnaryOp::Not => {
            checked_type_map
                .get(&typed_package.program.builtin_types().bool_)
                .copied()
                .ok_or_else(|| {
                    LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        "boolean result type could not be resolved in the lowered type table",
                    )
                })?
        }
    };
    let result_local = cursor.allocate_local(result_type, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::UnaryOp {
            op,
            operand: operand_val.local_id,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: result_type,
        recoverable_error_type: None,
    })
}
