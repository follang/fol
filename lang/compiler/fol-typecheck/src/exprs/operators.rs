use crate::{CheckedType, TypecheckError, TypecheckErrorKind, TypedProgram};
use fol_parser::ast::{AstNode, BinaryOperator, UnaryOperator};
use fol_resolver::ResolvedProgram;

use super::helpers::{
    apparent_type_id, invalid_binary_operator_error, invalid_unary_operator_error, is_equality_type,
    is_error_shell_type, is_ordered_type, merge_recoverable_effects, node_origin,
    observe_context, plain_value_expr, unsupported_binary_surface, unsupported_conversion_intrinsic,
    unwrap_shell_result_type, with_node_origin,
};
use super::{TypeContext, TypedExpr};
use super::type_node;
use super::type_node_with_expectation;

pub(crate) fn type_binary_op(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    op: &BinaryOperator,
    left: &AstNode,
    right: &AstNode,
) -> Result<TypedExpr, TypecheckError> {
    match op {
        BinaryOperator::As => {
            return Err(unsupported_conversion_intrinsic(
                resolved, left, right, "as",
            ));
        }
        BinaryOperator::Cast => {
            return Err(unsupported_conversion_intrinsic(
                resolved, left, right, "cast",
            ));
        }
        BinaryOperator::Pipe | BinaryOperator::PipeOr
            if matches!(super::helpers::strip_comments(right), AstNode::AsyncStage) =>
        {
            return Err(unsupported_binary_surface(
                resolved,
                left,
                right,
                "async pipe stages are planned for a future release",
            ));
        }
        BinaryOperator::Pipe | BinaryOperator::PipeOr
            if matches!(super::helpers::strip_comments(right), AstNode::AwaitStage) =>
        {
            return Err(unsupported_binary_surface(
                resolved,
                left,
                right,
                "await pipe stages are planned for a future release",
            ));
        }
        BinaryOperator::PipeOr => return type_pipe_or(typed, resolved, context, left, right),
        _ => {}
    }

    let left_raw = type_node(typed, resolved, context, left)?;
    let left_expr = plain_value_expr(
        typed,
        context,
        left_raw,
        node_origin(resolved, left),
        "plain use of an errorful expression",
    )?;
    let right_raw = type_node(typed, resolved, context, right)?;
    let right_expr = plain_value_expr(
        typed,
        context,
        right_raw,
        node_origin(resolved, right),
        "plain use of an errorful expression",
    )?;
    let left_type =
        left_expr.required_value("binary operator left operand does not have a type")?;
    let right_type =
        right_expr.required_value("binary operator right operand does not have a type")?;
    let left_apparent = apparent_type_id(typed, left_type)?;
    let right_apparent = apparent_type_id(typed, right_type)?;
    let merged_effect = merge_recoverable_effects(
        typed,
        node_origin(resolved, left).or_else(|| node_origin(resolved, right)),
        "binary expression",
        [left_expr.recoverable_effect, right_expr.recoverable_effect],
    )?;

    match op {
        BinaryOperator::Add => {
            match (
                typed.type_table().get(left_apparent),
                typed.type_table().get(right_apparent),
            ) {
                (
                    Some(CheckedType::Builtin(crate::BuiltinType::Int)),
                    Some(CheckedType::Builtin(crate::BuiltinType::Int)),
                ) => {
                    Ok(TypedExpr::value(typed.builtin_types().int)
                        .with_optional_effect(merged_effect))
                }
                (
                    Some(CheckedType::Builtin(crate::BuiltinType::Float)),
                    Some(CheckedType::Builtin(crate::BuiltinType::Float)),
                ) => Ok(TypedExpr::value(typed.builtin_types().float)
                    .with_optional_effect(merged_effect)),
                (
                    Some(CheckedType::Builtin(crate::BuiltinType::Str)),
                    Some(CheckedType::Builtin(crate::BuiltinType::Str)),
                ) => Ok(TypedExpr::value(typed.builtin_types().str_)
                    .with_optional_effect(merged_effect)),
                _ => Err(invalid_binary_operator_error(
                    typed, op, left_type, right_type,
                )),
            }
        }
        BinaryOperator::Sub
        | BinaryOperator::Mul
        | BinaryOperator::Div
        | BinaryOperator::Mod
        | BinaryOperator::Pow => match (
            typed.type_table().get(left_apparent),
            typed.type_table().get(right_apparent),
        ) {
            (
                Some(CheckedType::Builtin(crate::BuiltinType::Int)),
                Some(CheckedType::Builtin(crate::BuiltinType::Int)),
            ) => {
                Ok(TypedExpr::value(typed.builtin_types().int).with_optional_effect(merged_effect))
            }
            (
                Some(CheckedType::Builtin(crate::BuiltinType::Float)),
                Some(CheckedType::Builtin(crate::BuiltinType::Float)),
            ) => {
                Ok(TypedExpr::value(typed.builtin_types().float)
                    .with_optional_effect(merged_effect))
            }
            _ => Err(invalid_binary_operator_error(
                typed, op, left_type, right_type,
            )),
        },
        BinaryOperator::Eq | BinaryOperator::Ne => {
            if left_apparent == right_apparent && is_equality_type(typed, left_apparent) {
                Ok(TypedExpr::value(typed.builtin_types().bool_)
                    .with_optional_effect(merged_effect))
            } else {
                Err(invalid_binary_operator_error(
                    typed, op, left_type, right_type,
                ))
            }
        }
        BinaryOperator::Lt | BinaryOperator::Le | BinaryOperator::Gt | BinaryOperator::Ge => {
            if left_apparent == right_apparent && is_ordered_type(typed, left_apparent) {
                Ok(TypedExpr::value(typed.builtin_types().bool_)
                    .with_optional_effect(merged_effect))
            } else {
                Err(invalid_binary_operator_error(
                    typed, op, left_type, right_type,
                ))
            }
        }
        BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor => {
            if left_apparent == typed.builtin_types().bool_
                && right_apparent == typed.builtin_types().bool_
            {
                Ok(TypedExpr::value(typed.builtin_types().bool_)
                    .with_optional_effect(merged_effect))
            } else {
                Err(invalid_binary_operator_error(
                    typed, op, left_type, right_type,
                ))
            }
        }
        BinaryOperator::In | BinaryOperator::Has => Err(unsupported_binary_surface(
            resolved,
            left,
            right,
            "membership operators 'in' and 'has' are not yet supported",
        )),
        BinaryOperator::Is => Err(unsupported_binary_surface(
            resolved,
            left,
            right,
            "type testing operator 'is' is not yet supported",
        )),
        BinaryOperator::Pipe => Err(unsupported_binary_surface(
            resolved,
            left,
            right,
            "pipe operator '|>' is not yet supported",
        )),
        BinaryOperator::As | BinaryOperator::Cast | BinaryOperator::PipeOr => {
            unreachable!("handled before plain binary typing")
        }
    }
}

pub(crate) fn type_pipe_or(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    left: &AstNode,
    right: &AstNode,
) -> Result<TypedExpr, TypecheckError> {
    let observed_left = type_node(typed, resolved, observe_context(context), left)?;
    let Some(success_type) = observed_left.value_type else {
        return Err(node_origin(resolved, left).map_or_else(
            || {
                TypecheckError::new(
                    TypecheckErrorKind::InvalidInput,
                    "left side of '||' must produce a value result in V1",
                )
            },
            |origin| {
                TypecheckError::with_origin(
                    TypecheckErrorKind::InvalidInput,
                    "left side of '||' must produce a value result in V1",
                    origin,
                )
            },
        ));
    };
    if observed_left.recoverable_effect.is_none() {
        let message = if observed_left
            .value_type
            .map(|type_id| is_error_shell_type(typed, type_id))
            .transpose()?
            .unwrap_or(false)
        {
            "'||' handles routine call results with '/ ErrorType', not err[...] shell values in V1"
        } else {
            "'||' requires a routine call result with '/ ErrorType' on the left in V1"
        };
        return Err(node_origin(resolved, left).map_or_else(
            || TypecheckError::new(TypecheckErrorKind::InvalidInput, message),
            |origin| TypecheckError::with_origin(TypecheckErrorKind::InvalidInput, message, origin),
        ));
    }

    let fallback = type_node_with_expectation(typed, resolved, context, right, Some(success_type))?;
    let fallback = plain_value_expr(
        typed,
        context,
        fallback,
        node_origin(resolved, right),
        "recoverable-error fallback",
    )?;

    match fallback.value_type {
        Some(actual) if actual == typed.builtin_types().never => {
            Ok(TypedExpr::value(success_type).with_optional_effect(fallback.recoverable_effect))
        }
        Some(actual) => {
            super::helpers::ensure_assignable(
                typed,
                success_type,
                actual,
                "recoverable-error fallback".to_string(),
                node_origin(resolved, right),
            )?;
            Ok(TypedExpr::value(success_type).with_optional_effect(fallback.recoverable_effect))
        }
        None => Err(node_origin(resolved, right).map_or_else(
            || {
                TypecheckError::new(
                    TypecheckErrorKind::InvalidInput,
                    "right side of '||' must produce a value or early-exit in V1",
                )
            },
            |origin| {
                TypecheckError::with_origin(
                    TypecheckErrorKind::InvalidInput,
                    "right side of '||' must produce a value or early-exit in V1",
                    origin,
                )
            },
        )),
    }
}

pub(crate) fn type_unary_op(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    node: &AstNode,
    op: &UnaryOperator,
    operand: &AstNode,
) -> Result<TypedExpr, TypecheckError> {
    if matches!(op, UnaryOperator::Unwrap) {
        let operand_expr = type_node(typed, resolved, observe_context(context), operand)?;
        if operand_expr.recoverable_effect.is_some() {
            return Err(with_node_origin(
                resolved,
                node,
                TypecheckErrorKind::Unsupported,
                "postfix '!' unwrap applies to opt[...] and err[...] shell values, not to routine call results with '/ ErrorType' in V1",
            ));
        }
        let operand_type =
            operand_expr.required_value("unary operator operand does not have a type")?;
        return if let Some(inner) = unwrap_shell_result_type(typed, operand_type)? {
            Ok(TypedExpr::value(inner))
        } else {
            Err(with_node_origin(
                resolved,
                node,
                TypecheckErrorKind::InvalidInput,
                "unwrap requires an opt[...] or err[...] shell with a value type in V1",
            ))
        };
    }

    let operand_raw = type_node(typed, resolved, context, operand)?;
    let operand_expr = plain_value_expr(
        typed,
        context,
        operand_raw,
        node_origin(resolved, operand),
        "plain use of an errorful expression",
    )?;
    let operand_type =
        operand_expr.required_value("unary operator operand does not have a type")?;
    let apparent = apparent_type_id(typed, operand_type)?;

    match op {
        UnaryOperator::Neg => match typed.type_table().get(apparent) {
            Some(CheckedType::Builtin(crate::BuiltinType::Int)) => {
                Ok(TypedExpr::value(typed.builtin_types().int)
                    .with_optional_effect(operand_expr.recoverable_effect))
            }
            Some(CheckedType::Builtin(crate::BuiltinType::Float)) => {
                Ok(TypedExpr::value(typed.builtin_types().float)
                    .with_optional_effect(operand_expr.recoverable_effect))
            }
            _ => Err(invalid_unary_operator_error(typed, op, operand_type)),
        },
        UnaryOperator::Not => {
            if apparent == typed.builtin_types().bool_ {
                Ok(TypedExpr::value(typed.builtin_types().bool_)
                    .with_optional_effect(operand_expr.recoverable_effect))
            } else {
                Err(invalid_unary_operator_error(typed, op, operand_type))
            }
        }
        UnaryOperator::Ref | UnaryOperator::Deref => Err(TypecheckError::new(
            TypecheckErrorKind::Unsupported,
            "pointer operators are planned for a future release",
        )),
        UnaryOperator::Unwrap => unreachable!("unwrap is handled before plain unary typing"),
    }
}
