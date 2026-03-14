use crate::{
    decls, CheckedType, CheckedTypeId, RoutineType, TypecheckError, TypecheckErrorKind,
    TypecheckResult, TypedProgram,
};
use fol_parser::ast::{AstNode, Literal, QualifiedPath, SyntaxNodeId, SyntaxOrigin, WhenCase};
use fol_resolver::{
    ReferenceId, ReferenceKind, ResolvedProgram, ScopeId, SourceUnitId, SymbolId, SymbolKind,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy)]
struct TypeContext {
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    routine_return_type: Option<CheckedTypeId>,
    routine_error_type: Option<CheckedTypeId>,
}

pub fn type_program(typed: &mut TypedProgram) -> TypecheckResult<()> {
    let resolved = typed.resolved().clone();
    let syntax = resolved.syntax().clone();
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in syntax.source_units.iter().enumerate() {
        let source_unit_id = SourceUnitId(source_unit_index);
        let scope_id = match resolved.source_unit(source_unit_id).map(|unit| unit.scope_id) {
            Some(scope_id) => scope_id,
            None => {
                return Err(vec![internal_error(
                    "resolved source unit disappeared",
                    None,
                )])
            }
        };
        let context = TypeContext {
            source_unit_id,
            scope_id,
            routine_return_type: None,
            routine_error_type: None,
        };
        for item in &source_unit.items {
            if let Err(error) = type_node(typed, &resolved, context, &item.node) {
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

fn type_node(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    node: &AstNode,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    match node {
        AstNode::Comment { .. } => Ok(None),
        AstNode::Commented { node, .. } => type_node(typed, resolved, context, node),
        AstNode::VarDecl {
            name,
            type_hint: _,
            value,
            ..
        }
        | AstNode::LabDecl {
            name,
            type_hint: _,
            value,
            ..
        } => type_binding_initializer(
            typed,
            resolved,
            context,
            name,
            value.as_deref(),
            binding_kind_for(node),
        ),
        AstNode::Literal(literal) => Ok(Some(type_literal(typed, literal)?)),
        AstNode::Identifier { name, syntax_id } => {
            type_identifier_reference(typed, resolved, context, name, *syntax_id)
        }
        AstNode::QualifiedIdentifier { path } => {
            type_qualified_identifier_reference(typed, resolved, context, path)
        }
        AstNode::FunDecl {
            name,
            syntax_id,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::ProDecl {
            name,
            syntax_id,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::LogDecl {
            name,
            syntax_id,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        } => {
            let routine_scope = syntax_id
                .and_then(|syntax_id| resolved.scope_for_syntax(syntax_id))
                .unwrap_or(context.scope_id);
            let expected_return_type = return_type
                .as_ref()
                .map(|ty| decls::lower_type(typed, resolved, routine_scope, ty))
                .transpose()?;
            let expected_error_type = error_type
                .as_ref()
                .map(|ty| decls::lower_type(typed, resolved, routine_scope, ty))
                .transpose()?;
            let routine_context = TypeContext {
                source_unit_id: context.source_unit_id,
                scope_id: routine_scope,
                routine_return_type: expected_return_type,
                routine_error_type: expected_error_type,
            };
            let body_type = type_body(typed, resolved, routine_context, body)?;
            let _ = type_body(typed, resolved, routine_context, inquiries)?;
            if let (Some(expected), Some(actual)) = (expected_return_type, body_type) {
                ensure_assignable(
                    typed,
                    expected,
                    actual,
                    format!("routine '{name}' body"),
                    syntax_id.and_then(|id| origin_for(resolved, id)),
                )?;
            }
            if let (Some(syntax_id), Some(type_id)) =
                (syntax_id, expected_return_type.or(body_type))
            {
                typed.record_node_type(*syntax_id, context.source_unit_id, type_id)?;
            }
            Ok(body_type)
        }
        AstNode::Block { statements } => type_body(typed, resolved, context, statements),
        AstNode::Program { declarations } => type_body(typed, resolved, context, declarations),
        AstNode::When {
            expr,
            cases,
            default,
        } => type_when(typed, resolved, context, expr, cases, default.as_deref()),
        AstNode::Assignment { target, value } => {
            ensure_assignable_target(target)?;
            let expected = type_node(typed, resolved, context, target)?.ok_or_else(|| {
                TypecheckError::new(
                    TypecheckErrorKind::InvalidInput,
                    "assignment target does not have a type",
                )
            })?;
            let actual = type_node(typed, resolved, context, value)?.ok_or_else(|| {
                TypecheckError::new(
                    TypecheckErrorKind::InvalidInput,
                    "assignment value does not have a type",
                )
            })?;
            ensure_assignable(
                typed,
                expected,
                actual,
                "assignment".to_string(),
                None,
            )?;
            Ok(Some(expected))
        }
        AstNode::FunctionCall {
            name,
            args,
            syntax_id,
        } if name == "report" => type_report_call(typed, resolved, context, args, *syntax_id),
        AstNode::FunctionCall {
            name,
            args,
            syntax_id,
        } => type_function_call(typed, resolved, context, name, args, *syntax_id),
        AstNode::QualifiedFunctionCall { path, args } => {
            type_qualified_function_call(typed, resolved, context, path, args)
        }
        AstNode::MethodCall { object, method, args } => {
            type_method_call(typed, resolved, context, object, method, args)
        }
        AstNode::FieldAccess { object, field } => {
            type_field_access(typed, resolved, context, object, field)
        }
        AstNode::IndexAccess { container, index } => {
            type_index_access(typed, resolved, context, container, index)
        }
        AstNode::SliceAccess {
            container,
            start,
            end,
            ..
        } => type_slice_access(
            typed,
            resolved,
            context,
            container,
            start.as_deref(),
            end.as_deref(),
        ),
        AstNode::PatternAccess { .. } => Err(TypecheckError::new(
            TypecheckErrorKind::Unsupported,
            "pattern access is not part of the V1 typecheck milestone",
        )),
        AstNode::Return { value } => type_return(typed, resolved, context, value.as_deref()),
        AstNode::Yield { value } => type_node(typed, resolved, context, value),
        _ => {
            for child in node.children() {
                let _ = type_node(typed, resolved, context, child)?;
            }
            Ok(None)
        }
    }
}

fn type_body(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    nodes: &[AstNode],
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let mut final_type = None;
    for node in nodes {
        let node_type = type_node(typed, resolved, context, node)?;
        if node_type.is_some() {
            final_type = node_type;
        }
    }
    Ok(final_type)
}

fn type_literal(
    typed: &mut TypedProgram,
    literal: &Literal,
) -> Result<CheckedTypeId, TypecheckError> {
    Ok(match literal {
        Literal::Integer(_) => typed.builtin_types().int,
        Literal::Float(_) => typed.builtin_types().float,
        Literal::String(_) => typed.builtin_types().str_,
        Literal::Character(_) => typed.builtin_types().char_,
        Literal::Boolean(_) => typed.builtin_types().bool_,
        Literal::Nil => {
            return Err(TypecheckError::new(
                TypecheckErrorKind::Unsupported,
                "nil literals are not part of the V1 expression typing milestone",
            ));
        }
    })
}

fn type_binding_initializer(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    name: &str,
    value: Option<&AstNode>,
    symbol_kind: SymbolKind,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let initializer_type = value
        .map(|value| type_node(typed, resolved, context, value))
        .transpose()?
        .flatten();

    let Some(symbol_id) = find_symbol_in_scope(
        resolved,
        context.source_unit_id,
        context.scope_id,
        name,
        symbol_kind,
    ) else {
        return Ok(initializer_type);
    };
    let declared_type = typed
        .typed_symbol(symbol_id)
        .and_then(|symbol| symbol.declared_type);

    match (declared_type, initializer_type) {
        (Some(expected), Some(actual)) => {
            ensure_assignable(typed, expected, actual, format!("initializer for '{name}'"), None)?;
            Ok(Some(expected))
        }
        (None, Some(inferred)) => {
            let symbol = typed
                .typed_symbol_mut(symbol_id)
                .ok_or_else(|| internal_error("typed symbol table lost an inferred binding", None))?;
            symbol.declared_type = Some(inferred);
            Ok(Some(inferred))
        }
        (Some(expected), None) => Ok(Some(expected)),
        (None, None) => Ok(None),
    }
}

fn type_when(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    expr: &AstNode,
    cases: &[WhenCase],
    default: Option<&[AstNode]>,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let _ = type_node(typed, resolved, context, expr)?;
    let mut case_types = Vec::new();

    for case in cases {
        match case {
            WhenCase::Case { condition, body }
            | WhenCase::Is {
                value: condition,
                body,
            }
            | WhenCase::In {
                range: condition,
                body,
            }
            | WhenCase::Has {
                member: condition,
                body,
            }
            | WhenCase::On {
                channel: condition,
                body,
            } => {
                let _ = type_node(typed, resolved, context, condition)?;
                case_types.push(type_body(typed, resolved, context, body)?);
            }
            WhenCase::Of { type_match, body } => {
                let _ = decls::lower_type(typed, resolved, context.scope_id, type_match)?;
                case_types.push(type_body(typed, resolved, context, body)?);
            }
        }
    }

    let Some(default) = default else {
        return Ok(None);
    };
    let Some(expected) = type_body(typed, resolved, context, default)? else {
        return Ok(None);
    };

    for case_type in case_types {
        let Some(actual) = case_type else {
            return Ok(None);
        };
        ensure_assignable(typed, expected, actual, "when branch".to_string(), None)?;
    }

    Ok(Some(expected))
}

fn type_function_call(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    name: &str,
    args: &[AstNode],
    syntax_id: Option<SyntaxNodeId>,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let syntax_id = syntax_id.ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!("function call '{name}' does not retain a syntax id"),
        )
    })?;
    let reference_id =
        find_reference_by_syntax(resolved, syntax_id, ReferenceKind::FunctionCall, name)?;
    let signature = routine_signature_for_reference(typed, resolved, reference_id, origin_for(resolved, syntax_id))?;
    check_call_arguments(typed, resolved, context, &signature, args, name, origin_for(resolved, syntax_id))?;
    if let Some(return_type) = signature.return_type {
        let typed_reference = typed
            .typed_reference_mut(reference_id)
            .ok_or_else(|| internal_error("typed call reference disappeared", None))?;
        typed_reference.resolved_type = Some(return_type);
        typed.record_node_type(syntax_id, context.source_unit_id, return_type)?;
        Ok(Some(return_type))
    } else {
        Ok(None)
    }
}

fn type_report_call(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    args: &[AstNode],
    syntax_id: Option<SyntaxNodeId>,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let origin = syntax_id.and_then(|syntax_id| origin_for(resolved, syntax_id));
    let Some(expected) = context.routine_error_type else {
        return Err(TypecheckError::with_origin(
            TypecheckErrorKind::InvalidInput,
            "report requires a declared routine error type in V1",
            origin.unwrap_or(SyntaxOrigin {
                file: None,
                line: 1,
                column: 1,
                length: "report".len(),
            }),
        ));
    };

    if args.len() != 1 {
        return Err(TypecheckError::with_origin(
            TypecheckErrorKind::InvalidInput,
            format!("report expects exactly 1 value in V1 but got {}", args.len()),
            origin.unwrap_or(SyntaxOrigin {
                file: None,
                line: 1,
                column: 1,
                length: "report".len(),
            }),
        ));
    }

    let actual = type_node(typed, resolved, context, &args[0])?.ok_or_else(|| {
        TypecheckError::with_origin(
            TypecheckErrorKind::InvalidInput,
            "report expression does not have a type",
            origin.clone().unwrap_or(SyntaxOrigin {
                file: None,
                line: 1,
                column: 1,
                length: "report".len(),
            }),
        )
    })?;
    ensure_assignable(typed, expected, actual, "report".to_string(), origin)?;
    Ok(Some(actual))
}

fn type_qualified_function_call(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    path: &QualifiedPath,
    args: &[AstNode],
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let syntax_id = path.syntax_id().ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!("qualified function call '{}' does not retain a syntax id", path.joined()),
        )
    })?;
    let reference_id = find_reference_by_syntax(
        resolved,
        syntax_id,
        ReferenceKind::QualifiedFunctionCall,
        &path.joined(),
    )?;
    let signature = routine_signature_for_reference(typed, resolved, reference_id, origin_for(resolved, syntax_id))?;
    check_call_arguments(
        typed,
        resolved,
        context,
        &signature,
        args,
        &path.joined(),
        origin_for(resolved, syntax_id),
    )?;
    if let Some(return_type) = signature.return_type {
        let typed_reference = typed
            .typed_reference_mut(reference_id)
            .ok_or_else(|| internal_error("typed qualified call reference disappeared", None))?;
        typed_reference.resolved_type = Some(return_type);
        typed.record_node_type(syntax_id, context.source_unit_id, return_type)?;
        Ok(Some(return_type))
    } else {
        Ok(None)
    }
}

fn type_method_call(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    object: &AstNode,
    method: &str,
    args: &[AstNode],
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let object_type = type_node(typed, resolved, context, object)?.ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!("method receiver for '{method}' does not have a type"),
        )
    })?;
    let signature = routine_signature_for_method(typed, resolved, method, object_type)?;
    check_call_arguments(typed, resolved, context, &signature, args, method, None)?;
    Ok(signature.return_type)
}

fn type_return(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    value: Option<&AstNode>,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let Some(expected) = context.routine_return_type else {
        return match value {
            Some(_) => Err(TypecheckError::new(
                TypecheckErrorKind::InvalidInput,
                "return with a value requires a declared routine return type in V1",
            )),
            None => Ok(None),
        };
    };

    let Some(value) = value else {
        return Err(TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            "return requires a value for routines with a declared return type",
        ));
    };
    let actual = type_node(typed, resolved, context, value)?.ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            "return expression does not have a type",
        )
    })?;
    ensure_assignable(typed, expected, actual, "return".to_string(), None)?;
    Ok(Some(actual))
}

fn type_field_access(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    object: &AstNode,
    field: &str,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let object_type = type_node(typed, resolved, context, object)?.ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!("field access '.{field}' does not have a typed receiver"),
        )
    })?;
    let resolved_type = apparent_type_id(typed, object_type)?;
    match typed.type_table().get(resolved_type) {
        Some(CheckedType::Record { fields }) => fields.get(field).copied().map(Some).ok_or_else(|| {
            TypecheckError::new(
                TypecheckErrorKind::InvalidInput,
                format!("record receiver does not expose a field named '{field}'"),
            )
        }),
        _ => Err(TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!(
                "field access '.{field}' requires a record-like receiver, got '{}'",
                describe_type(typed, object_type)
            ),
        )),
    }
}

fn type_index_access(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    container: &AstNode,
    index: &AstNode,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let container_type = type_node(typed, resolved, context, container)?.ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            "index access does not have a typed container",
        )
    })?;
    let index_type = type_node(typed, resolved, context, index)?.ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            "index access does not have a typed index expression",
        )
    })?;
    let resolved_type = apparent_type_id(typed, container_type)?;
    match typed.type_table().get(resolved_type) {
        Some(CheckedType::Array { element_type, .. })
        | Some(CheckedType::Vector { element_type })
        | Some(CheckedType::Sequence { element_type }) => {
            ensure_assignable(
                typed,
                typed.builtin_types().int,
                index_type,
                "container index".to_string(),
                None,
            )?;
            Ok(Some(*element_type))
        }
        Some(CheckedType::Map {
            key_type,
            value_type,
        }) => {
            ensure_assignable(
                typed,
                *key_type,
                index_type,
                "map key".to_string(),
                None,
            )?;
            Ok(Some(*value_type))
        }
        _ => Err(TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!(
                "index access requires an array, vector, sequence, or map receiver, got '{}'",
                describe_type(typed, container_type)
            ),
        )),
    }
}

fn type_slice_access(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    container: &AstNode,
    start: Option<&AstNode>,
    end: Option<&AstNode>,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let container_type = type_node(typed, resolved, context, container)?.ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            "slice access does not have a typed container",
        )
    })?;
    for bound in [start, end].into_iter().flatten() {
        let bound_type = type_node(typed, resolved, context, bound)?.ok_or_else(|| {
            TypecheckError::new(
                TypecheckErrorKind::InvalidInput,
                "slice bound does not have a type",
            )
        })?;
        ensure_assignable(
            typed,
            typed.builtin_types().int,
            bound_type,
            "slice bound".to_string(),
            None,
        )?;
    }
    let resolved_type = apparent_type_id(typed, container_type)?;
    match typed.type_table().get(resolved_type) {
        Some(CheckedType::Array { element_type, .. }) => {
            let element_type = *element_type;
            Ok(Some(typed.type_table_mut().intern(CheckedType::Array {
                element_type,
                size: None,
            })))
        }
        Some(CheckedType::Vector { element_type }) => {
            let element_type = *element_type;
            Ok(Some(
                typed
                    .type_table_mut()
                    .intern(CheckedType::Vector { element_type }),
            ))
        }
        Some(CheckedType::Sequence { element_type }) => {
            let element_type = *element_type;
            Ok(Some(
                typed
                    .type_table_mut()
                    .intern(CheckedType::Sequence { element_type }),
            ))
        }
        _ => Err(TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!(
                "slice access requires an array, vector, or sequence receiver, got '{}'",
                describe_type(typed, container_type)
            ),
        )),
    }
}

fn type_identifier_reference(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    name: &str,
    syntax_id: Option<SyntaxNodeId>,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let syntax_id = syntax_id.ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!("identifier '{name}' does not retain a syntax id"),
        )
    })?;
    let reference_id =
        find_reference_by_syntax(resolved, syntax_id, ReferenceKind::Identifier, name)?;
    let type_id = type_for_reference(typed, resolved, reference_id, origin_for(resolved, syntax_id))?;
    typed.record_node_type(syntax_id, context.source_unit_id, type_id)?;
    Ok(Some(type_id))
}

fn type_qualified_identifier_reference(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    path: &QualifiedPath,
) -> Result<Option<CheckedTypeId>, TypecheckError> {
    let syntax_id = path.syntax_id().ok_or_else(|| {
        TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!(
                "qualified identifier '{}' does not retain a syntax id",
                path.joined()
            ),
        )
    })?;
    let reference_id = find_reference_by_syntax(
        resolved,
        syntax_id,
        ReferenceKind::QualifiedIdentifier,
        &path.joined(),
    )?;
    let type_id = type_for_reference(typed, resolved, reference_id, origin_for(resolved, syntax_id))?;
    typed.record_node_type(syntax_id, context.source_unit_id, type_id)?;
    Ok(Some(type_id))
}

fn find_reference_by_syntax(
    resolved: &ResolvedProgram,
    syntax_id: SyntaxNodeId,
    kind: ReferenceKind,
    display_name: &str,
) -> Result<ReferenceId, TypecheckError> {
    resolved
        .references
        .iter_with_ids()
        .find(|(_, reference)| reference.syntax_id == Some(syntax_id) && reference.kind == kind)
        .map(|(reference_id, _)| reference_id)
        .ok_or_else(|| {
            TypecheckError::with_origin(
                TypecheckErrorKind::InvalidInput,
                format!("reference '{display_name}' is missing from resolver output"),
                origin_for(resolved, syntax_id).unwrap_or(SyntaxOrigin {
                    file: None,
                    line: 1,
                    column: 1,
                    length: display_name.len(),
                }),
            )
        })
}

fn routine_signature_for_reference(
    typed: &TypedProgram,
    resolved: &ResolvedProgram,
    reference_id: ReferenceId,
    origin: Option<SyntaxOrigin>,
) -> Result<RoutineType, TypecheckError> {
    let symbol_id = resolved
        .reference(reference_id)
        .and_then(|reference| reference.resolved)
        .ok_or_else(|| {
            TypecheckError::with_origin(
                TypecheckErrorKind::InvalidInput,
                "call reference lost its resolved routine symbol",
                origin.clone().unwrap_or(SyntaxOrigin {
                    file: None,
                    line: 1,
                    column: 1,
                    length: 1,
                }),
            )
        })?;
    routine_signature_for_symbol(typed, symbol_id, origin)
}

fn routine_signature_for_symbol(
    typed: &TypedProgram,
    symbol_id: SymbolId,
    origin: Option<SyntaxOrigin>,
) -> Result<RoutineType, TypecheckError> {
    let type_id = symbol_type(typed, symbol_id, origin.clone())?;
    match typed.type_table().get(type_id) {
        Some(CheckedType::Routine(signature)) => Ok(signature.clone()),
        _ => Err(TypecheckError::with_origin(
            TypecheckErrorKind::InvalidInput,
            format!("resolved routine symbol {} is not callable", symbol_id.0),
            origin.unwrap_or(SyntaxOrigin {
                file: None,
                line: 1,
                column: 1,
                length: 1,
            }),
        )),
    }
}

fn routine_signature_for_method(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    method: &str,
    object_type: CheckedTypeId,
) -> Result<RoutineType, TypecheckError> {
    let mut matches = Vec::new();

    for (source_unit_index, source_unit) in resolved.syntax().source_units.iter().enumerate() {
        let source_unit_id = SourceUnitId(source_unit_index);
        let scope_id = resolved
            .source_unit(source_unit_id)
            .map(|unit| unit.scope_id)
            .ok_or_else(|| internal_error("resolved source unit disappeared for method lookup", None))?;
        for item in &source_unit.items {
            match &item.node {
                AstNode::FunDecl {
                    name,
                    receiver_type: Some(receiver_type),
                    ..
                }
                | AstNode::ProDecl {
                    name,
                    receiver_type: Some(receiver_type),
                    ..
                }
                | AstNode::LogDecl {
                    name,
                    receiver_type: Some(receiver_type),
                    ..
                } if name == method => {
                    let lowered_receiver = decls::lower_type(typed, resolved, scope_id, receiver_type)?;
                    if lowered_receiver == object_type {
                        if let Some(symbol_id) = resolved
                            .symbols
                            .iter_with_ids()
                            .find(|(_, symbol)| {
                                symbol.source_unit == source_unit_id
                                    && symbol.kind == SymbolKind::Routine
                                    && symbol.name == *name
                            })
                            .map(|(symbol_id, _)| symbol_id)
                        {
                            matches.push(routine_signature_for_symbol(typed, symbol_id, None)?);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!("method '{method}' is not available for the receiver type in V1"),
        )),
        _ => Err(TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            format!("method '{method}' is ambiguous for the receiver type"),
        )),
    }
}

fn check_call_arguments(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    context: TypeContext,
    signature: &RoutineType,
    args: &[AstNode],
    callee: &str,
    origin: Option<SyntaxOrigin>,
) -> Result<(), TypecheckError> {
    if signature.params.len() != args.len() {
        return Err(TypecheckError::with_origin(
            TypecheckErrorKind::InvalidInput,
            format!(
                "call to '{callee}' expects {} args but got {}",
                signature.params.len(),
                args.len()
            ),
            origin.unwrap_or(SyntaxOrigin {
                file: None,
                line: 1,
                column: 1,
                length: callee.len(),
            }),
        ));
    }

    for (expected, arg) in signature.params.iter().zip(args) {
        let actual = type_node(typed, resolved, context, arg)?.ok_or_else(|| {
            TypecheckError::new(
                TypecheckErrorKind::InvalidInput,
                format!("argument for '{callee}' does not have a type"),
            )
        })?;
        ensure_assignable(
            typed,
            *expected,
            actual,
            format!("call to '{callee}'"),
            origin.clone(),
        )?;
    }

    Ok(())
}

fn type_for_reference(
    typed: &mut TypedProgram,
    resolved: &ResolvedProgram,
    reference_id: ReferenceId,
    origin: Option<SyntaxOrigin>,
) -> Result<CheckedTypeId, TypecheckError> {
    let symbol_id = resolved
        .reference(reference_id)
        .and_then(|reference| reference.resolved)
        .ok_or_else(|| {
            TypecheckError::with_origin(
                TypecheckErrorKind::InvalidInput,
                "resolved reference lost its target symbol",
                origin.clone().unwrap_or(SyntaxOrigin {
                    file: None,
                    line: 1,
                    column: 1,
                    length: 1,
                }),
            )
        })?;
    let type_id = symbol_type(typed, symbol_id, origin.clone())?;
    let typed_reference = typed.typed_reference_mut(reference_id).ok_or_else(|| {
        TypecheckError::with_origin(
            TypecheckErrorKind::Internal,
            "typed reference table lost a resolved reference",
            origin.unwrap_or(SyntaxOrigin {
                file: None,
                line: 1,
                column: 1,
                length: 1,
            }),
        )
    })?;
    typed_reference.resolved_type = Some(type_id);
    Ok(type_id)
}

fn symbol_type(
    typed: &TypedProgram,
    symbol_id: SymbolId,
    origin: Option<SyntaxOrigin>,
) -> Result<CheckedTypeId, TypecheckError> {
    typed
        .typed_symbol(symbol_id)
        .and_then(|symbol| symbol.declared_type)
        .ok_or_else(|| {
            TypecheckError::with_origin(
                TypecheckErrorKind::InvalidInput,
                format!("resolved symbol {} does not have a lowered type yet", symbol_id.0),
                origin.unwrap_or(SyntaxOrigin {
                    file: None,
                    line: 1,
                    column: 1,
                    length: 1,
                }),
            )
        })
}

fn apparent_type_id(
    typed: &TypedProgram,
    type_id: CheckedTypeId,
) -> Result<CheckedTypeId, TypecheckError> {
    let mut current = type_id;
    let mut seen = BTreeSet::new();

    loop {
        match typed.type_table().get(current) {
            Some(CheckedType::Declared { symbol, .. }) => {
                if !seen.insert(*symbol) {
                    return Err(TypecheckError::new(
                        TypecheckErrorKind::InvalidInput,
                        "declared type expansion encountered a cycle",
                    ));
                }
                let Some(next) = typed.typed_symbol(*symbol).and_then(|symbol| symbol.declared_type)
                else {
                    return Ok(current);
                };
                if next == current {
                    return Ok(current);
                }
                current = next;
            }
            _ => return Ok(current),
        }
    }
}

fn origin_for(resolved: &ResolvedProgram, syntax_id: SyntaxNodeId) -> Option<SyntaxOrigin> {
    resolved.syntax_index().origin(syntax_id).cloned()
}

fn find_symbol_in_scope(
    resolved: &ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    name: &str,
    kind: SymbolKind,
) -> Option<SymbolId> {
    resolved
        .symbols
        .iter_with_ids()
        .find(|(_, symbol)| {
            symbol.source_unit == source_unit_id
                && symbol.scope == scope_id
                && symbol.name == name
                && symbol.kind == kind
        })
        .map(|(symbol_id, _)| symbol_id)
}

fn binding_kind_for(node: &AstNode) -> SymbolKind {
    match node {
        AstNode::LabDecl { .. } => SymbolKind::LabelBinding,
        _ => SymbolKind::ValueBinding,
    }
}

fn ensure_assignable(
    typed: &TypedProgram,
    expected: CheckedTypeId,
    actual: CheckedTypeId,
    surface: String,
    origin: Option<SyntaxOrigin>,
) -> Result<(), TypecheckError> {
    if expected == actual || actual == typed.builtin_types().never {
        return Ok(());
    }

    Err(TypecheckError::with_origin(
        TypecheckErrorKind::IncompatibleType,
        format!(
            "{surface} expects '{}' but got '{}'",
            describe_type(typed, expected),
            describe_type(typed, actual)
        ),
        origin.unwrap_or(SyntaxOrigin {
            file: None,
            line: 1,
            column: 1,
            length: 1,
        }),
    ))
}

fn describe_type(typed: &TypedProgram, type_id: CheckedTypeId) -> String {
    format!(
        "{:?}",
        typed
            .type_table()
            .get(type_id)
            .cloned()
            .unwrap_or(crate::CheckedType::Builtin(crate::BuiltinType::Never))
    )
}

fn internal_error(message: impl Into<String>, origin: Option<SyntaxOrigin>) -> TypecheckError {
    if let Some(origin) = origin {
        TypecheckError::with_origin(TypecheckErrorKind::Internal, message, origin)
    } else {
        TypecheckError::new(TypecheckErrorKind::Internal, message)
    }
}

fn ensure_assignable_target(target: &AstNode) -> Result<(), TypecheckError> {
    match target {
        AstNode::Identifier { .. } | AstNode::QualifiedIdentifier { .. } => Ok(()),
        _ => Err(TypecheckError::new(
            TypecheckErrorKind::InvalidInput,
            "assignment targets must currently be plain or qualified identifiers",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::type_literal;
    use crate::{BuiltinType, TypedProgram};
    use fol_parser::ast::{AstParser, Literal};
    use fol_resolver::resolve_package;
    use fol_stream::FileStream;

    #[test]
    fn literal_typing_maps_v1_scalar_literals_to_builtin_types() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open expression fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Expression fixture should parse");
        let resolved = resolve_package(syntax).expect("Expression fixture should resolve");
        let mut typed = TypedProgram::from_resolved(resolved);

        assert_eq!(
            typed.type_table().get(type_literal(&mut typed, &Literal::Integer(1)).unwrap()),
            Some(&crate::CheckedType::Builtin(BuiltinType::Int))
        );
        assert_eq!(
            typed
                .type_table()
                .get(type_literal(&mut typed, &Literal::String("ok".to_string())).unwrap()),
            Some(&crate::CheckedType::Builtin(BuiltinType::Str))
        );
    }
}
