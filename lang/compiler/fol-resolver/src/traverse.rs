use crate::{
    collect::{
        binding_names, insert_import_record, semantic_node, top_level_duplicate_key,
        top_level_scope_id,
    },
    errors::{format_origin_brief, symbol_kind_label},
    imports,
    model::{
        ReferenceKind, ResolvedProgram, ResolvedReference, ResolvedSymbol, ScopeKind, SymbolKind,
    },
    ReferenceId, ResolverError, ResolverErrorKind, ResolverSession, ScopeId, SourceUnitId,
    SymbolId,
};
use fol_parser::ast::{
    AstNode, CallSurface, FolType, Generic, InquiryTarget, LoopCondition, ParsedDeclVisibility,
    ParsedTopLevel, QualifiedPath, TypeDefinition, WhenCase,
};

#[derive(Debug, Clone, Copy)]
struct RoutineContext {
    this_available: bool,
}

pub fn collect_routine_scopes(
    session: &mut ResolverSession,
    program: &mut ResolvedProgram,
) -> Result<(), Vec<ResolverError>> {
    let mut errors = Vec::new();
    let work_items = program
        .syntax()
        .source_units
        .iter()
        .enumerate()
        .flat_map(|(source_unit_id, syntax_unit)| {
            syntax_unit
                .items
                .iter()
                .cloned()
                .map(move |item| (SourceUnitId(source_unit_id), item))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    for (source_unit_id, item) in work_items {
        let scope_id = top_level_scope_id(program, source_unit_id, &item);
        if let Err(error) =
            traverse_top_level_item(session, program, source_unit_id, scope_id, &item)
        {
            errors.push(error);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn traverse_top_level_item(
    session: &mut ResolverSession,
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    _scope_id: ScopeId,
    item: &ParsedTopLevel,
) -> Result<(), ResolverError> {
    let traversal_scope = program
        .source_unit(source_unit_id)
        .expect("top-level traversal source unit should exist")
        .scope_id;
    traverse_node(
        session,
        program,
        source_unit_id,
        traversal_scope,
        &item.node,
        true,
        None,
    )
}

fn traverse_node(
    session: &mut ResolverSession,
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    node: &AstNode,
    is_top_level_node: bool,
    routine_context: Option<RoutineContext>,
) -> Result<(), ResolverError> {
    match semantic_node(node) {
        AstNode::FunDecl {
            syntax_id,
            generics,
            receiver_type,
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::ProDecl {
            syntax_id,
            generics,
            receiver_type,
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::LogDecl {
            syntax_id,
            generics,
            receiver_type,
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        } => {
            if !is_top_level_node {
                insert_local_named_symbol(
                    program,
                    source_unit_id,
                    scope_id,
                    node,
                    SymbolKind::Routine,
                )?;
            }

            let routine_scope = program.add_scope(ScopeKind::Routine, scope_id, source_unit_id);
            let nested_routine_context = Some(RoutineContext {
                this_available: return_type.is_some(),
            });
            program.record_scope_for_syntax(*syntax_id, routine_scope);

            insert_generic_symbols(program, source_unit_id, routine_scope, generics)?;
            for generic in generics {
                for constraint in &generic.constraints {
                    resolve_type_reference(
                        session,
                        program,
                        source_unit_id,
                        routine_scope,
                        constraint,
                    )?;
                }
            }
            if let Some(receiver_type) = receiver_type {
                resolve_type_reference(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    receiver_type,
                )?;
            }
            for param in params {
                resolve_type_reference(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    &param.param_type,
                )?;
            }
            if let Some(return_type) = return_type {
                resolve_type_reference(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    return_type,
                )?;
            }
            if let Some(error_type) = error_type {
                resolve_type_reference(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    error_type,
                )?;
            }

            for capture in captures {
                insert_local_symbol(
                    program,
                    source_unit_id,
                    routine_scope,
                    capture,
                    SymbolKind::Capture,
                    format!("symbol#{}", fol_types::canonical_identifier_key(capture)),
                )?;
            }
            for param in params {
                insert_local_symbol(
                    program,
                    source_unit_id,
                    routine_scope,
                    &param.name,
                    SymbolKind::Parameter,
                    format!(
                        "symbol#{}",
                        fol_types::canonical_identifier_key(&param.name)
                    ),
                )?;
            }

            for statement in body {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    statement,
                    false,
                    nested_routine_context,
                )?;
            }
            for inquiry in inquiries {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    inquiry,
                    false,
                    nested_routine_context,
                )?;
            }
        }
        AstNode::AnonymousFun {
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::AnonymousPro {
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        }
        | AstNode::AnonymousLog {
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
            ..
        } => {
            let routine_scope = program.add_scope(ScopeKind::Routine, scope_id, source_unit_id);
            let nested_routine_context = Some(RoutineContext {
                this_available: return_type.is_some(),
            });

            for param in params {
                resolve_type_reference(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    &param.param_type,
                )?;
            }
            if let Some(return_type) = return_type {
                resolve_type_reference(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    return_type,
                )?;
            }
            if let Some(error_type) = error_type {
                resolve_type_reference(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    error_type,
                )?;
            }

            for capture in captures {
                insert_local_symbol(
                    program,
                    source_unit_id,
                    routine_scope,
                    capture,
                    SymbolKind::Capture,
                    format!("symbol#{}", fol_types::canonical_identifier_key(capture)),
                )?;
            }

            for param in params {
                insert_local_symbol(
                    program,
                    source_unit_id,
                    routine_scope,
                    &param.name,
                    SymbolKind::Parameter,
                    format!(
                        "symbol#{}",
                        fol_types::canonical_identifier_key(&param.name)
                    ),
                )?;
            }

            for statement in body {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    statement,
                    false,
                    nested_routine_context,
                )?;
            }
            for inquiry in inquiries {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    routine_scope,
                    inquiry,
                    false,
                    nested_routine_context,
                )?;
            }
        }
        AstNode::Comment { .. } | AstNode::Commented { .. } => {}
        AstNode::Identifier { name, syntax_id } => {
            record_identifier_reference(
                program,
                source_unit_id,
                scope_id,
                name,
                *syntax_id,
                syntax_id
                    .and_then(|syntax_id| program.syntax_index().origin(syntax_id))
                    .cloned(),
            )?;
        }
        AstNode::QualifiedIdentifier { path } => {
            record_qualified_identifier_reference(program, source_unit_id, scope_id, path)?;
        }
        AstNode::BinaryOp { left, right, .. } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                left,
                false,
                routine_context,
            )?;
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                right,
                false,
                routine_context,
            )?;
        }
        AstNode::UnaryOp { operand, .. } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                operand,
                false,
                routine_context,
            )?;
        }
        AstNode::FunctionCall {
            surface: CallSurface::DotIntrinsic,
            args,
            ..
        } => {
            for arg in args {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    arg,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::FunctionCall {
            name,
            args,
            syntax_id,
            ..
        } => {
            if !is_builtin_diagnostic_call(name) {
                record_function_call_reference(
                    program,
                    source_unit_id,
                    scope_id,
                    name,
                    *syntax_id,
                    syntax_id
                        .and_then(|syntax_id| program.syntax_index().origin(syntax_id))
                        .cloned(),
                )?;
            }
            for arg in args {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    arg,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::QualifiedFunctionCall { path, args } => {
            record_qualified_function_call_reference(program, source_unit_id, scope_id, path)?;
            for arg in args {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    arg,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::Invoke { callee, args } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                callee,
                false,
                routine_context,
            )?;
            for arg in args {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    arg,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::NamedArgument { value, .. }
        | AstNode::Unpack { value }
        | AstNode::Spawn { task: value }
        | AstNode::Return { value: Some(value) } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                value,
                false,
                routine_context,
            )?;
        }
        AstNode::Assignment { target, value } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                target,
                false,
                routine_context,
            )?;
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                value,
                false,
                routine_context,
            )?;
        }
        AstNode::Return { value: None }
        | AstNode::Break
        | AstNode::AsyncStage
        | AstNode::AwaitStage => {}
        AstNode::MethodCall { object, args, .. } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                object,
                false,
                routine_context,
            )?;
            for arg in args {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    arg,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::TemplateCall { object, .. }
        | AstNode::FieldAccess { object, .. }
        | AstNode::AvailabilityAccess { target: object } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                object,
                false,
                routine_context,
            )?;
        }
        AstNode::IndexAccess { container, index } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                container,
                false,
                routine_context,
            )?;
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                index,
                false,
                routine_context,
            )?;
        }
        AstNode::ChannelAccess { channel, .. } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                channel,
                false,
                routine_context,
            )?;
        }
        AstNode::SliceAccess {
            container,
            start,
            end,
            ..
        } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                container,
                false,
                routine_context,
            )?;
            if let Some(start) = start {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    start,
                    false,
                    routine_context,
                )?;
            }
            if let Some(end) = end {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    end,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::PatternAccess {
            container,
            patterns,
        } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                container,
                false,
                routine_context,
            )?;
            for pattern in patterns {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    pattern,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::PatternCapture { pattern, .. } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                pattern,
                false,
                routine_context,
            )?;
        }
        AstNode::ContainerLiteral { elements, .. } => {
            for element in elements {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    element,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::RecordInit { fields, .. } => {
            for field in fields {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    &field.value,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::Rolling {
            expr,
            bindings,
            condition,
        } => {
            let rolling_scope =
                program.add_scope(ScopeKind::RollingBinder, scope_id, source_unit_id);
            for binding in bindings {
                if let Some(type_hint) = &binding.type_hint {
                    resolve_type_reference(
                        session,
                        program,
                        source_unit_id,
                        rolling_scope,
                        type_hint,
                    )?;
                }
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    rolling_scope,
                    &binding.iterable,
                    false,
                    routine_context,
                )?;
                insert_local_symbol(
                    program,
                    source_unit_id,
                    rolling_scope,
                    &binding.name,
                    SymbolKind::RollingBinder,
                    format!(
                        "symbol#{}",
                        fol_types::canonical_identifier_key(&binding.name)
                    ),
                )?;
            }
            traverse_node(
                session,
                program,
                source_unit_id,
                rolling_scope,
                expr,
                false,
                routine_context,
            )?;
            if let Some(condition) = condition {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    rolling_scope,
                    condition,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::Range { start, end, .. } => {
            if let Some(start) = start {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    start,
                    false,
                    routine_context,
                )?;
            }
            if let Some(end) = end {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    end,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::VarDecl { name, value, .. } => {
            if let AstNode::VarDecl {
                type_hint: Some(type_hint),
                ..
            } = semantic_node(node)
            {
                resolve_type_reference(session, program, source_unit_id, scope_id, type_hint)?;
            }
            if let Some(value) = value {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    value,
                    false,
                    routine_context,
                )?;
            }
            if !is_top_level_node {
                insert_local_symbol(
                    program,
                    source_unit_id,
                    scope_id,
                    name,
                    SymbolKind::ValueBinding,
                    format!("symbol#{}", fol_types::canonical_identifier_key(name)),
                )?;
            }
        }
        AstNode::LabDecl { name, value, .. } => {
            if let AstNode::LabDecl {
                type_hint: Some(type_hint),
                ..
            } = semantic_node(node)
            {
                resolve_type_reference(session, program, source_unit_id, scope_id, type_hint)?;
            }
            if let Some(value) = value {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    value,
                    false,
                    routine_context,
                )?;
            }
            if !is_top_level_node {
                insert_local_symbol(
                    program,
                    source_unit_id,
                    scope_id,
                    name,
                    SymbolKind::LabelBinding,
                    format!("symbol#{}", fol_types::canonical_identifier_key(name)),
                )?;
            }
        }
        AstNode::DestructureDecl { pattern, value, .. } => {
            if let AstNode::DestructureDecl {
                type_hint: Some(type_hint),
                ..
            } = semantic_node(node)
            {
                resolve_type_reference(session, program, source_unit_id, scope_id, type_hint)?;
            }
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                value,
                false,
                routine_context,
            )?;
            if !is_top_level_node {
                for name in binding_names(pattern) {
                    insert_local_symbol(
                        program,
                        source_unit_id,
                        scope_id,
                        &name,
                        SymbolKind::DestructureBinding,
                        format!("symbol#{}", fol_types::canonical_identifier_key(&name)),
                    )?;
                }
            }
        }
        AstNode::Yield { value } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                value,
                false,
                routine_context,
            )?;
        }
        AstNode::When {
            expr,
            cases,
            default,
        } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                expr,
                false,
                routine_context,
            )?;
            for case in cases {
                match case {
                    WhenCase::Case { condition, body } => {
                        traverse_node(
                            session,
                            program,
                            source_unit_id,
                            scope_id,
                            condition,
                            false,
                            routine_context,
                        )?;
                        traverse_block_body(
                            session,
                            program,
                            source_unit_id,
                            scope_id,
                            body,
                            routine_context,
                        )?;
                    }
                    WhenCase::Is { value, body }
                    | WhenCase::In { range: value, body }
                    | WhenCase::Has {
                        member: value,
                        body,
                    }
                    | WhenCase::On {
                        channel: value,
                        body,
                    } => {
                        traverse_node(
                            session,
                            program,
                            source_unit_id,
                            scope_id,
                            value,
                            false,
                            routine_context,
                        )?;
                        traverse_block_body(
                            session,
                            program,
                            source_unit_id,
                            scope_id,
                            body,
                            routine_context,
                        )?;
                    }
                    WhenCase::Of { type_match, body } => {
                        resolve_type_reference(
                            session,
                            program,
                            source_unit_id,
                            scope_id,
                            type_match,
                        )?;
                        traverse_block_body(
                            session,
                            program,
                            source_unit_id,
                            scope_id,
                            body,
                            routine_context,
                        )?;
                    }
                }
            }
            if let Some(default_body) = default {
                traverse_block_body(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    default_body,
                    routine_context,
                )?;
            }
        }
        AstNode::Loop { condition, body } => match condition.as_ref() {
            LoopCondition::Condition(condition) => {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    condition,
                    false,
                    routine_context,
                )?;
                for statement in body {
                    traverse_node(
                        session,
                        program,
                        source_unit_id,
                        scope_id,
                        statement,
                        false,
                        routine_context,
                    )?;
                }
            }
            LoopCondition::Iteration {
                var,
                type_hint,
                iterable,
                condition,
                ..
            } => {
                if let Some(type_hint) = type_hint {
                    resolve_type_reference(session, program, source_unit_id, scope_id, type_hint)?;
                }
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    iterable,
                    false,
                    routine_context,
                )?;
                let binder_scope =
                    program.add_scope(ScopeKind::LoopBinder, scope_id, source_unit_id);
                insert_local_symbol(
                    program,
                    source_unit_id,
                    binder_scope,
                    var,
                    SymbolKind::LoopBinder,
                    format!("symbol#{}", fol_types::canonical_identifier_key(var)),
                )?;
                if let Some(condition) = condition {
                    traverse_node(
                        session,
                        program,
                        source_unit_id,
                        binder_scope,
                        condition,
                        false,
                        routine_context,
                    )?;
                }
                for statement in body {
                    traverse_node(
                        session,
                        program,
                        source_unit_id,
                        binder_scope,
                        statement,
                        false,
                        routine_context,
                    )?;
                }
            }
        },
        AstNode::Select {
            channel,
            binding,
            body,
        } => {
            traverse_node(
                session,
                program,
                source_unit_id,
                scope_id,
                channel,
                false,
                routine_context,
            )?;
            let select_scope = program.add_scope(ScopeKind::Block, scope_id, source_unit_id);
            if let Some(binding) = binding {
                insert_local_symbol(
                    program,
                    source_unit_id,
                    select_scope,
                    binding,
                    SymbolKind::ValueBinding,
                    format!("symbol#{}", fol_types::canonical_identifier_key(binding)),
                )?;
            }
            for statement in body {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    select_scope,
                    statement,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::Block { statements } => {
            traverse_block_body(
                session,
                program,
                source_unit_id,
                scope_id,
                statements,
                routine_context,
            )?;
        }
        AstNode::Program {
            declarations: statements,
        } => {
            for statement in statements {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    statement,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::Inquiry { target, body } => {
            resolve_inquiry_target(program, source_unit_id, scope_id, target, routine_context)?;
            for statement in body {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    statement,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::TypeDecl {
            generics,
            contracts,
            type_def,
            ..
        } => {
            let type_scope =
                program.add_scope(ScopeKind::TypeDeclaration, scope_id, source_unit_id);
            insert_generic_symbols(program, source_unit_id, type_scope, generics)?;
            for generic in generics {
                for constraint in &generic.constraints {
                    resolve_type_reference(
                        session,
                        program,
                        source_unit_id,
                        type_scope,
                        constraint,
                    )?;
                }
            }
            for contract in contracts {
                resolve_type_reference(session, program, source_unit_id, type_scope, contract)?;
            }
            resolve_type_definition(session, program, source_unit_id, type_scope, type_def)?;
        }
        AstNode::AliasDecl { target, .. } => {
            resolve_type_reference(session, program, source_unit_id, scope_id, target)?;
        }
        AstNode::DefDecl {
            params, def_type, ..
        } => {
            for param in params {
                resolve_type_reference(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    &param.param_type,
                )?;
            }
            resolve_type_reference(session, program, source_unit_id, scope_id, def_type)?;
        }
        AstNode::SegDecl { seg_type, .. } => {
            resolve_type_reference(session, program, source_unit_id, scope_id, seg_type)?;
        }
        AstNode::ImpDecl {
            generics,
            target,
            body,
            ..
        } => {
            let impl_scope =
                program.add_scope(ScopeKind::TypeDeclaration, scope_id, source_unit_id);
            insert_generic_symbols(program, source_unit_id, impl_scope, generics)?;
            for generic in generics {
                for constraint in &generic.constraints {
                    resolve_type_reference(
                        session,
                        program,
                        source_unit_id,
                        impl_scope,
                        constraint,
                    )?;
                }
            }
            resolve_type_reference(session, program, source_unit_id, impl_scope, target)?;
            for member in body {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    impl_scope,
                    member,
                    false,
                    routine_context,
                )?;
            }
        }
        AstNode::StdDecl { .. } | AstNode::Literal(_) | AstNode::PatternWildcard => {}
        AstNode::UseDecl {
            name,
            path_type,
            path_segments,
            ..
        } => {
            if !is_top_level_node {
                let symbol_id = insert_local_symbol(
                    program,
                    source_unit_id,
                    scope_id,
                    name,
                    SymbolKind::ImportAlias,
                    format!("symbol#{}", fol_types::canonical_identifier_key(name)),
                )?;
                let import_id = insert_import_record(
                    program,
                    source_unit_id,
                    scope_id,
                    symbol_id,
                    name,
                    path_type.clone(),
                    path_segments.clone(),
                );
                imports::resolve_import_target_with_session(session, program, import_id)?;
            }
        }
    }

    Ok(())
}

fn traverse_block_body(
    session: &mut ResolverSession,
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    parent_scope: ScopeId,
    statements: &[AstNode],
    routine_context: Option<RoutineContext>,
) -> Result<(), ResolverError> {
    let block_scope = program.add_scope(ScopeKind::Block, parent_scope, source_unit_id);
    for statement in statements {
        traverse_node(
            session,
            program,
            source_unit_id,
            block_scope,
            statement,
            false,
            routine_context,
        )?;
    }
    Ok(())
}

fn insert_local_named_symbol(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    node: &AstNode,
    kind: SymbolKind,
) -> Result<SymbolId, ResolverError> {
    let name = match semantic_node(node) {
        AstNode::FunDecl { name, .. }
        | AstNode::ProDecl { name, .. }
        | AstNode::LogDecl { name, .. } => name.as_str(),
        _ => {
            return Err(ResolverError::new(
                ResolverErrorKind::Internal,
                "attempted to bind a local named symbol from a non-routine node",
            ));
        }
    };
    let canonical_name = fol_types::canonical_identifier_key(name);
    let duplicate_key = top_level_duplicate_key(semantic_node(node), &canonical_name);
    insert_local_symbol(program, source_unit_id, scope_id, name, kind, duplicate_key)
}

fn insert_local_symbol(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    name: &str,
    kind: SymbolKind,
    duplicate_key: String,
) -> Result<SymbolId, ResolverError> {
    let canonical_name = fol_types::canonical_identifier_key(name);
    // Resolver contract: one scope cannot redefine the same binding shape, but
    // nested scopes may intentionally shadow names from parent scopes.
    if let Some(existing) = program
        .scope(scope_id)
        .and_then(|scope| scope.symbol_keys.get(&canonical_name))
        .into_iter()
        .flat_map(|ids| ids.iter())
        .filter_map(|id| program.symbol(*id))
        .find(|symbol| symbol.duplicate_key == duplicate_key)
    {
        let existing_site = existing
            .origin
            .as_ref()
            .map(format_origin_brief)
            .unwrap_or_else(|| "an unknown location".to_string());
        return Err(ResolverError::with_origin(
            ResolverErrorKind::DuplicateSymbol,
            format!(
                "duplicate local symbol '{}' conflicts with existing {} declaration first declared at {}",
                name,
                symbol_kind_label(existing.kind),
                existing_site
            ),
            existing
                .origin
                .clone()
                .unwrap_or(fol_parser::ast::SyntaxOrigin {
                    file: None,
                    line: 1,
                    column: 1,
                    length: name.len(),
                }),
        )
        .with_related_origin(
            existing
                .origin
                .clone()
                .unwrap_or(fol_parser::ast::SyntaxOrigin {
                    file: None,
                    line: 1,
                    column: 1,
                    length: name.len(),
                }),
            format!("first {} declaration", symbol_kind_label(existing.kind)),
        ));
    }

    let symbol_id = program.symbols.push(ResolvedSymbol {
        id: SymbolId(0),
        name: name.to_string(),
        canonical_name: canonical_name.clone(),
        duplicate_key,
        kind,
        scope: scope_id,
        source_unit: source_unit_id,
        origin: None,
        visibility: None,
        declaration_scope: None,
        mounted_from: None,
    });
    if let Some(symbol) = program.symbols.get_mut(symbol_id) {
        symbol.id = symbol_id;
    }

    let scope = program
        .scopes
        .get_mut(scope_id)
        .expect("local symbol target scope should exist");
    scope.symbols.push(symbol_id);
    scope
        .symbol_keys
        .entry(canonical_name)
        .or_default()
        .push(symbol_id);

    Ok(symbol_id)
}

fn record_identifier_reference(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    name: &str,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<ReferenceId, ResolverError> {
    let symbol_id = resolve_visible_symbol(program, scope_id, name, origin)?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::Identifier,
        syntax_id,
        name: name.to_string(),
        scope: scope_id,
        source_unit: source_unit_id,
        resolved: Some(symbol_id),
    });
    if let Some(reference) = program.references.get_mut(reference_id) {
        reference.id = reference_id;
    }
    Ok(reference_id)
}

fn is_builtin_diagnostic_call(name: &str) -> bool {
    matches!(name, "panic" | "report" | "check" | "assert")
}

fn record_function_call_reference(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    name: &str,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<ReferenceId, ResolverError> {
    let symbol_id = resolve_visible_or_imported_symbol_of_kinds(
        program,
        scope_id,
        name,
        &[SymbolKind::Routine],
        Some("callable routine"),
        origin,
    )?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::FunctionCall,
        syntax_id,
        name: name.to_string(),
        scope: scope_id,
        source_unit: source_unit_id,
        resolved: Some(symbol_id),
    });
    if let Some(reference) = program.references.get_mut(reference_id) {
        reference.id = reference_id;
    }
    Ok(reference_id)
}

fn record_qualified_identifier_reference(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    path: &QualifiedPath,
) -> Result<ReferenceId, ResolverError> {
    let symbol_id = resolve_qualified_symbol(
        program,
        scope_id,
        path,
        &[],
        "qualified identifier",
        qualified_path_origin(program, path),
    )?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::QualifiedIdentifier,
        syntax_id: path.syntax_id(),
        name: path.joined(),
        scope: scope_id,
        source_unit: source_unit_id,
        resolved: Some(symbol_id),
    });
    if let Some(reference) = program.references.get_mut(reference_id) {
        reference.id = reference_id;
    }
    Ok(reference_id)
}

fn record_qualified_function_call_reference(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    path: &QualifiedPath,
) -> Result<ReferenceId, ResolverError> {
    let symbol_id = resolve_qualified_symbol(
        program,
        scope_id,
        path,
        &[SymbolKind::Routine],
        "qualified callable routine",
        qualified_path_origin(program, path),
    )?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::QualifiedFunctionCall,
        syntax_id: path.syntax_id(),
        name: path.joined(),
        scope: scope_id,
        source_unit: source_unit_id,
        resolved: Some(symbol_id),
    });
    if let Some(reference) = program.references.get_mut(reference_id) {
        reference.id = reference_id;
    }
    Ok(reference_id)
}

fn record_named_type_reference(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    name: &str,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<ReferenceId, ResolverError> {
    let symbol_id = resolve_visible_or_imported_symbol_of_kinds(
        program,
        scope_id,
        name,
        &[
            SymbolKind::Type,
            SymbolKind::Alias,
            SymbolKind::GenericParameter,
        ],
        Some("type"),
        origin,
    )?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::TypeName,
        syntax_id,
        name: name.to_string(),
        scope: scope_id,
        source_unit: source_unit_id,
        resolved: Some(symbol_id),
    });
    if let Some(reference) = program.references.get_mut(reference_id) {
        reference.id = reference_id;
    }
    Ok(reference_id)
}

fn record_inquiry_target_reference(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    name: &str,
    resolved: SymbolId,
) -> ReferenceId {
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::InquiryTarget,
        syntax_id: None,
        name: name.to_string(),
        scope: scope_id,
        source_unit: source_unit_id,
        resolved: Some(resolved),
    });
    if let Some(reference) = program.references.get_mut(reference_id) {
        reference.id = reference_id;
    }
    reference_id
}

fn resolve_inquiry_target(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    target: &InquiryTarget,
    routine_context: Option<RoutineContext>,
) -> Result<(), ResolverError> {
    match target {
        InquiryTarget::SelfValue => {
            if routine_context.is_none() {
                return Err(ResolverError::new(
                    ResolverErrorKind::InvalidInput,
                    "inquiry target 'self' requires an enclosing routine context",
                ));
            }
        }
        InquiryTarget::ThisValue => {
            if !routine_context.is_some_and(|context| context.this_available) {
                return Err(ResolverError::new(
                    ResolverErrorKind::InvalidInput,
                    "inquiry target 'this' requires an enclosing routine with a declared return type",
                ));
            }
        }
        InquiryTarget::Named(name) | InquiryTarget::Quoted(name) => {
            let symbol_id = resolve_visible_symbol_of_kinds(
                program,
                scope_id,
                name,
                &[],
                Some("inquiry target"),
                None,
            )?;
            record_inquiry_target_reference(program, source_unit_id, scope_id, name, symbol_id);
        }
        InquiryTarget::Qualified(path) => {
            let symbol_id = resolve_qualified_symbol(
                program,
                scope_id,
                path,
                &[],
                "qualified inquiry target",
                qualified_path_origin(program, path),
            )?;
            record_inquiry_target_reference(
                program,
                source_unit_id,
                scope_id,
                &path.joined(),
                symbol_id,
            );
        }
    }

    Ok(())
}

fn record_qualified_type_reference(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    path: &QualifiedPath,
) -> Result<ReferenceId, ResolverError> {
    let symbol_id = resolve_qualified_symbol(
        program,
        scope_id,
        path,
        &[SymbolKind::Type, SymbolKind::Alias],
        "qualified type",
        qualified_path_origin(program, path),
    )?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::QualifiedTypeName,
        syntax_id: path.syntax_id(),
        name: path.joined(),
        scope: scope_id,
        source_unit: source_unit_id,
        resolved: Some(symbol_id),
    });
    if let Some(reference) = program.references.get_mut(reference_id) {
        reference.id = reference_id;
    }
    Ok(reference_id)
}

fn resolve_visible_symbol(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    name: &str,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<SymbolId, ResolverError> {
    match resolve_lexical_symbol_of_kinds(program, starting_scope, name, &[], None, origin.clone())
    {
        Ok(symbol_id) => Ok(symbol_id),
        Err(error) if error.kind() == ResolverErrorKind::UnresolvedName => {
            resolve_imported_symbol_of_kinds(program, starting_scope, name, &[], None, origin)
        }
        Err(error) => Err(error),
    }
}

fn resolve_lexical_symbol_of_kinds(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    name: &str,
    allowed_kinds: &[SymbolKind],
    missing_role: Option<&str>,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<SymbolId, ResolverError> {
    let canonical_name = fol_types::canonical_identifier_key(name);
    let mut current_scope = Some(starting_scope);

    while let Some(scope_id) = current_scope {
        let symbols = program.symbols_named_in_scope(scope_id, &canonical_name);
        if !symbols.is_empty() {
            let matching_symbols = if allowed_kinds.is_empty() {
                symbols
            } else {
                symbols
                    .into_iter()
                    .filter(|symbol| allowed_kinds.contains(&symbol.kind))
                    .collect::<Vec<_>>()
            };

            if matching_symbols.len() == 1 {
                return Ok(matching_symbols[0].id);
            }
            if matching_symbols.len() > 1 {
                return Err(ambiguity_error_with_optional_origin(
                    lexical_ambiguity_message(name, missing_role, &matching_symbols),
                    origin,
                    &matching_symbols,
                ));
            }

            if allowed_kinds.is_empty() {
                return Err(ambiguity_error_with_optional_origin(
                    lexical_ambiguity_message(name, missing_role, &matching_symbols),
                    origin,
                    &matching_symbols,
                ));
            }

            return Err(error_with_optional_origin(
                ResolverErrorKind::UnresolvedName,
                format!(
                    "could not resolve {} '{}'",
                    missing_role.unwrap_or("name"),
                    name
                ),
                origin,
            ));
        }

        current_scope = program.scope(scope_id).and_then(|scope| scope.parent);
    }

    Err(error_with_optional_origin(
        ResolverErrorKind::UnresolvedName,
        format!(
            "could not resolve {} '{}'",
            missing_role.unwrap_or("name"),
            name
        ),
        origin,
    ))
}

fn resolve_imported_symbol_of_kinds(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    name: &str,
    allowed_kinds: &[SymbolKind],
    missing_role: Option<&str>,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<SymbolId, ResolverError> {
    let canonical_name = fol_types::canonical_identifier_key(name);
    let mut current_scope = Some(starting_scope);
    let mut matches = std::collections::BTreeMap::new();

    while let Some(scope_id) = current_scope {
        for import in program.imports_in_scope(scope_id) {
            let Some(target_scope) = import.target_scope else {
                continue;
            };
            let imported_symbols = program.symbols_named_in_scope(target_scope, &canonical_name);
            if allowed_kinds.is_empty() {
                for symbol in imported_symbols.into_iter().filter(import_visible_symbol) {
                    matches.entry(symbol.id).or_insert(symbol);
                }
            } else {
                for symbol in imported_symbols
                    .into_iter()
                    .filter(import_visible_symbol)
                    .filter(|symbol| allowed_kinds.contains(&symbol.kind))
                {
                    matches.entry(symbol.id).or_insert(symbol);
                }
            }
        }
        current_scope = program.scope(scope_id).and_then(|scope| scope.parent);
    }

    let matches = matches.into_values().collect::<Vec<_>>();

    match matches.as_slice() {
        [symbol] => Ok(symbol.id),
        [] => Err(error_with_optional_origin(
            ResolverErrorKind::UnresolvedName,
            format!(
                "could not resolve {} '{}'",
                missing_role.unwrap_or("name"),
                name
            ),
            origin,
        )),
        _ => Err(ambiguity_error_with_optional_origin(
            lexical_ambiguity_message(name, missing_role, &matches),
            origin,
            &matches,
        )),
    }
}

fn resolve_visible_or_imported_symbol_of_kinds(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    name: &str,
    allowed_kinds: &[SymbolKind],
    missing_role: Option<&str>,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<SymbolId, ResolverError> {
    match resolve_lexical_symbol_of_kinds(
        program,
        starting_scope,
        name,
        allowed_kinds,
        missing_role,
        origin.clone(),
    ) {
        Ok(symbol_id) => Ok(symbol_id),
        Err(error) if error.kind() == ResolverErrorKind::UnresolvedName => {
            resolve_imported_symbol_of_kinds(
                program,
                starting_scope,
                name,
                allowed_kinds,
                missing_role,
                origin,
            )
        }
        Err(error) => Err(error),
    }
}

fn resolve_visible_symbol_of_kinds(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    name: &str,
    allowed_kinds: &[SymbolKind],
    missing_role: Option<&str>,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<SymbolId, ResolverError> {
    resolve_lexical_symbol_of_kinds(
        program,
        starting_scope,
        name,
        allowed_kinds,
        missing_role,
        origin,
    )
}

fn insert_generic_symbols(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    generics: &[Generic],
) -> Result<(), ResolverError> {
    for generic in generics {
        insert_local_symbol(
            program,
            source_unit_id,
            scope_id,
            &generic.name,
            SymbolKind::GenericParameter,
            format!(
                "symbol#{}",
                fol_types::canonical_identifier_key(&generic.name)
            ),
        )?;
    }

    Ok(())
}

fn resolve_type_reference(
    session: &mut ResolverSession,
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    typ: &FolType,
) -> Result<(), ResolverError> {
    match typ {
        typ if typ.is_builtin_str() => {}
        FolType::Named { name, syntax_id } => {
            record_named_type_reference(
                program,
                source_unit_id,
                scope_id,
                name,
                *syntax_id,
                syntax_id
                    .and_then(|syntax_id| program.syntax_index().origin(syntax_id))
                    .cloned(),
            )?;
        }
        FolType::Array { element_type, .. }
        | FolType::Vector { element_type }
        | FolType::Sequence { element_type }
        | FolType::Channel { element_type } => {
            resolve_type_reference(session, program, source_unit_id, scope_id, element_type)?;
        }
        FolType::Matrix { element_type, .. } => {
            resolve_type_reference(session, program, source_unit_id, scope_id, element_type)?;
        }
        FolType::Set { types } | FolType::Multiple { types } | FolType::Union { types } => {
            for part in types {
                resolve_type_reference(session, program, source_unit_id, scope_id, part)?;
            }
        }
        FolType::Map {
            key_type,
            value_type,
        } => {
            resolve_type_reference(session, program, source_unit_id, scope_id, key_type)?;
            resolve_type_reference(session, program, source_unit_id, scope_id, value_type)?;
        }
        FolType::Record { fields } => {
            for field_type in fields.values() {
                resolve_type_reference(session, program, source_unit_id, scope_id, field_type)?;
            }
        }
        FolType::Entry { variants } => {
            for variant in variants.values().flatten() {
                resolve_type_reference(session, program, source_unit_id, scope_id, variant)?;
            }
        }
        FolType::Optional { inner } | FolType::Pointer { target: inner } => {
            resolve_type_reference(session, program, source_unit_id, scope_id, inner)?;
        }
        FolType::Error { inner } => {
            if let Some(inner) = inner {
                resolve_type_reference(session, program, source_unit_id, scope_id, inner)?;
            }
        }
        FolType::Limited { base, limits } => {
            resolve_type_reference(session, program, source_unit_id, scope_id, base)?;
            for limit in limits {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    limit,
                    false,
                    None,
                )?;
            }
        }
        FolType::Function {
            params,
            return_type,
        } => {
            for param in params {
                resolve_type_reference(session, program, source_unit_id, scope_id, param)?;
            }
            resolve_type_reference(session, program, source_unit_id, scope_id, return_type)?;
        }
        FolType::Generic { constraints, .. } => {
            for constraint in constraints {
                resolve_type_reference(session, program, source_unit_id, scope_id, constraint)?;
            }
        }
        FolType::QualifiedNamed { path } => {
            record_qualified_type_reference(program, source_unit_id, scope_id, path)?;
        }
        FolType::Int { .. }
        | FolType::Float { .. }
        | FolType::Char { .. }
        | FolType::Bool
        | FolType::Never
        | FolType::Any
        | FolType::None
        | FolType::Package { .. }
        | FolType::Module { .. }
        | FolType::Block { .. }
        | FolType::Test { .. }
        | FolType::Location { .. }
        | FolType::Standard { .. } => {}
    }

    Ok(())
}

fn resolve_type_definition(
    session: &mut ResolverSession,
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    type_def: &TypeDefinition,
) -> Result<(), ResolverError> {
    match type_def {
        TypeDefinition::Record {
            fields, members, ..
        } => {
            for field_type in fields.values() {
                resolve_type_reference(session, program, source_unit_id, scope_id, field_type)?;
            }
            for member in members {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    member,
                    false,
                    None,
                )?;
            }
        }
        TypeDefinition::Entry {
            variants, members, ..
        } => {
            for variant_type in variants.values().flatten() {
                resolve_type_reference(session, program, source_unit_id, scope_id, variant_type)?;
            }
            for member in members {
                traverse_node(
                    session,
                    program,
                    source_unit_id,
                    scope_id,
                    member,
                    false,
                    None,
                )?;
            }
        }
        TypeDefinition::Alias { target } => {
            resolve_type_reference(session, program, source_unit_id, scope_id, target)?;
        }
    }

    Ok(())
}

fn resolve_qualified_symbol(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    path: &QualifiedPath,
    allowed_kinds: &[SymbolKind],
    missing_role: &str,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<SymbolId, ResolverError> {
    if path.segments.len() < 2 {
        return Err(ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "qualified path '{}' must contain at least two segments",
                path.joined()
            ),
        ));
    }

    let (mut current_scope, mut current_namespace) = resolve_qualified_root(
        program,
        starting_scope,
        &path.segments[0],
        &path.joined(),
        missing_role,
        origin.clone(),
    )?;

    for segment in &path.segments[1..path.segments.len() - 1] {
        current_namespace.push_str("::");
        current_namespace.push_str(segment);
        current_scope = program.namespace_scope(&current_namespace).ok_or_else(|| {
            error_with_optional_origin(
                ResolverErrorKind::UnresolvedName,
                format!("could not resolve {} '{}'", missing_role, path.joined()),
                origin.clone(),
            )
        })?;
    }

    let final_name = path
        .segments
        .last()
        .expect("qualified paths with at least two segments should have a final segment");
    resolve_symbol_in_scope(
        program,
        current_scope,
        final_name,
        allowed_kinds,
        &path.joined(),
        missing_role,
        origin,
    )
}

fn resolve_qualified_root(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    root_segment: &str,
    full_path: &str,
    missing_role: &str,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<(ScopeId, String), ResolverError> {
    if root_segment == program.package_name() {
        return Ok((program.program_scope, program.package_name().to_string()));
    }

    if let Ok(import_symbol) = resolve_visible_symbol_of_kinds(
        program,
        starting_scope,
        root_segment,
        &[SymbolKind::ImportAlias],
        Some("import alias"),
        origin.clone(),
    ) {
        let import = program
            .imports
            .iter()
            .find(|import| import.alias_symbol == import_symbol)
            .and_then(|import| import.target_scope);
        if let Some(target_scope) = import {
            return Ok((target_scope, scope_namespace(program, target_scope)));
        }
    }

    let namespace = format!("{}::{}", program.package_name(), root_segment);
    if let Some(scope_id) = program.namespace_scope(&namespace) {
        return Ok((scope_id, namespace));
    }

    Err(error_with_optional_origin(
        ResolverErrorKind::UnresolvedName,
        format!("could not resolve {} '{}'", missing_role, full_path),
        origin,
    ))
}

fn resolve_symbol_in_scope(
    program: &ResolvedProgram,
    scope_id: ScopeId,
    name: &str,
    allowed_kinds: &[SymbolKind],
    full_path: &str,
    missing_role: &str,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<SymbolId, ResolverError> {
    let canonical_name = fol_types::canonical_identifier_key(name);
    let symbols = program.symbols_named_in_scope(scope_id, &canonical_name);
    let matching_symbols = if allowed_kinds.is_empty() {
        symbols
    } else {
        symbols
            .into_iter()
            .filter(|symbol| allowed_kinds.contains(&symbol.kind))
            .collect::<Vec<_>>()
    };

    match matching_symbols.as_slice() {
        [symbol] => Ok(symbol.id),
        [] => Err(error_with_optional_origin(
            ResolverErrorKind::UnresolvedName,
            format!("could not resolve {} '{}'", missing_role, full_path),
            origin.clone(),
        )),
        _ => Err(ambiguity_error_with_optional_origin(
            format!(
                "{} '{}' is ambiguous; candidates: {}",
                missing_role,
                full_path,
                describe_symbol_candidates(&matching_symbols)
            ),
            origin,
            &matching_symbols,
        )),
    }
}

fn error_with_optional_origin(
    kind: ResolverErrorKind,
    message: String,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> ResolverError {
    match origin {
        Some(origin) => ResolverError::with_origin(kind, message, origin),
        None => ResolverError::new(kind, message),
    }
}

fn ambiguity_error_with_optional_origin(
    message: String,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
    symbols: &[&ResolvedSymbol],
) -> ResolverError {
    let mut error =
        error_with_optional_origin(ResolverErrorKind::AmbiguousReference, message, origin);
    let mut seen = std::collections::BTreeSet::new();

    for symbol in symbols {
        let Some(symbol_origin) = symbol.origin.clone() else {
            continue;
        };
        let dedupe_key = (
            symbol_origin.file.clone(),
            symbol_origin.line,
            symbol_origin.column,
            symbol_origin.length,
        );
        if seen.insert(dedupe_key) {
            error = error.with_related_origin(
                symbol_origin,
                format!("candidate {} declaration", symbol_kind_label(symbol.kind)),
            );
        }
    }

    error
}

fn qualified_path_origin(
    program: &ResolvedProgram,
    path: &QualifiedPath,
) -> Option<fol_parser::ast::SyntaxOrigin> {
    path.syntax_id()
        .and_then(|syntax_id| program.syntax_index().origin(syntax_id))
        .cloned()
}

fn scope_namespace(program: &ResolvedProgram, scope_id: ScopeId) -> String {
    match program
        .scope(scope_id)
        .map(|scope| &scope.kind)
        .expect("qualified path scope should exist")
    {
        ScopeKind::ProgramRoot { package } => package.clone(),
        ScopeKind::NamespaceRoot { namespace } => namespace.clone(),
        other => panic!("qualified path root scope must be package or namespace, got {other:?}"),
    }
}

fn describe_symbol_candidates(symbols: &[&ResolvedSymbol]) -> String {
    symbols
        .iter()
        .map(|symbol| {
            let site = symbol
                .origin
                .as_ref()
                .map(format_origin_brief)
                .unwrap_or_else(|| "an unknown location".to_string());
            format!(
                "{} '{}' at {}",
                symbol_kind_label(symbol.kind),
                symbol.name,
                site
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn import_visible_symbol(symbol: &&ResolvedSymbol) -> bool {
    symbol.visibility == Some(ParsedDeclVisibility::Exported)
}

fn lexical_ambiguity_message(
    name: &str,
    missing_role: Option<&str>,
    symbols: &[&ResolvedSymbol],
) -> String {
    let subject = match missing_role {
        Some(role) => format!("{role} '{name}'"),
        None => format!("name '{name}'"),
    };
    format!(
        "{subject} is ambiguous in lexical scope; candidates: {}",
        describe_symbol_candidates(symbols)
    )
}
