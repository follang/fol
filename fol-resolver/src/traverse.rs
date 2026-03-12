use crate::{
    collect::{
        binding_names, insert_import_record, semantic_node, top_level_duplicate_key,
        top_level_scope_id,
    },
    model::{
        ReferenceKind, ResolvedProgram, ResolvedReference, ResolvedSymbol, ScopeKind, SymbolKind,
    },
    ReferenceId, ResolverError, ResolverErrorKind, ScopeId, SourceUnitId, SymbolId,
};
use fol_parser::ast::{
    AstNode, FolType, Generic, LoopCondition, ParsedTopLevel, QualifiedPath, TypeDefinition,
    WhenCase,
};

pub fn collect_routine_scopes(program: &mut ResolvedProgram) -> Result<(), Vec<ResolverError>> {
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
        })
        .collect::<Vec<_>>();

    for (source_unit_id, item) in work_items {
        let scope_id = top_level_scope_id(program, source_unit_id, &item);
        if let Err(error) = traverse_top_level_item(program, source_unit_id, scope_id, &item) {
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
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    item: &ParsedTopLevel,
) -> Result<(), ResolverError> {
    traverse_node(program, source_unit_id, scope_id, &item.node, true)
}

fn traverse_node(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    node: &AstNode,
    is_top_level_node: bool,
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
            program.record_scope_for_syntax(*syntax_id, routine_scope);

            insert_generic_symbols(program, source_unit_id, routine_scope, generics)?;
            for generic in generics {
                for constraint in &generic.constraints {
                    resolve_type_reference(program, source_unit_id, routine_scope, constraint)?;
                }
            }
            if let Some(receiver_type) = receiver_type {
                resolve_type_reference(program, source_unit_id, routine_scope, receiver_type)?;
            }
            for param in params {
                resolve_type_reference(program, source_unit_id, routine_scope, &param.param_type)?;
            }
            if let Some(return_type) = return_type {
                resolve_type_reference(program, source_unit_id, routine_scope, return_type)?;
            }
            if let Some(error_type) = error_type {
                resolve_type_reference(program, source_unit_id, routine_scope, error_type)?;
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
                traverse_node(program, source_unit_id, routine_scope, statement, false)?;
            }
            for inquiry in inquiries {
                traverse_node(program, source_unit_id, routine_scope, inquiry, false)?;
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

            for param in params {
                resolve_type_reference(program, source_unit_id, routine_scope, &param.param_type)?;
            }
            if let Some(return_type) = return_type {
                resolve_type_reference(program, source_unit_id, routine_scope, return_type)?;
            }
            if let Some(error_type) = error_type {
                resolve_type_reference(program, source_unit_id, routine_scope, error_type)?;
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
                traverse_node(program, source_unit_id, routine_scope, statement, false)?;
            }
            for inquiry in inquiries {
                traverse_node(program, source_unit_id, routine_scope, inquiry, false)?;
            }
        }
        AstNode::Comment { .. } | AstNode::Commented { .. } => {}
        AstNode::Identifier { name } => {
            record_identifier_reference(program, source_unit_id, scope_id, name)?;
        }
        AstNode::QualifiedIdentifier { path } => {
            record_qualified_identifier_reference(program, source_unit_id, scope_id, path)?;
        }
        AstNode::BinaryOp { left, right, .. } => {
            traverse_node(program, source_unit_id, scope_id, left, false)?;
            traverse_node(program, source_unit_id, scope_id, right, false)?;
        }
        AstNode::UnaryOp { operand, .. } => {
            traverse_node(program, source_unit_id, scope_id, operand, false)?;
        }
        AstNode::FunctionCall { name, args } => {
            record_function_call_reference(program, source_unit_id, scope_id, name)?;
            for arg in args {
                traverse_node(program, source_unit_id, scope_id, arg, false)?;
            }
        }
        AstNode::QualifiedFunctionCall { path, args } => {
            record_qualified_function_call_reference(program, source_unit_id, scope_id, path)?;
            for arg in args {
                traverse_node(program, source_unit_id, scope_id, arg, false)?;
            }
        }
        AstNode::Invoke { callee, args } => {
            traverse_node(program, source_unit_id, scope_id, callee, false)?;
            for arg in args {
                traverse_node(program, source_unit_id, scope_id, arg, false)?;
            }
        }
        AstNode::NamedArgument { value, .. }
        | AstNode::Unpack { value }
        | AstNode::Spawn { task: value }
        | AstNode::Return { value: Some(value) } => {
            traverse_node(program, source_unit_id, scope_id, value, false)?;
        }
        AstNode::Assignment { target, value } => {
            traverse_node(program, source_unit_id, scope_id, target, false)?;
            traverse_node(program, source_unit_id, scope_id, value, false)?;
        }
        AstNode::Return { value: None }
        | AstNode::Break
        | AstNode::AsyncStage
        | AstNode::AwaitStage => {}
        AstNode::MethodCall { object, args, .. } => {
            traverse_node(program, source_unit_id, scope_id, object, false)?;
            for arg in args {
                traverse_node(program, source_unit_id, scope_id, arg, false)?;
            }
        }
        AstNode::TemplateCall { object, .. }
        | AstNode::FieldAccess { object, .. }
        | AstNode::AvailabilityAccess { target: object } => {
            traverse_node(program, source_unit_id, scope_id, object, false)?;
        }
        AstNode::IndexAccess { container, index } => {
            traverse_node(program, source_unit_id, scope_id, container, false)?;
            traverse_node(program, source_unit_id, scope_id, index, false)?;
        }
        AstNode::ChannelAccess { channel, .. } => {
            traverse_node(program, source_unit_id, scope_id, channel, false)?;
        }
        AstNode::SliceAccess {
            container,
            start,
            end,
            ..
        } => {
            traverse_node(program, source_unit_id, scope_id, container, false)?;
            if let Some(start) = start {
                traverse_node(program, source_unit_id, scope_id, start, false)?;
            }
            if let Some(end) = end {
                traverse_node(program, source_unit_id, scope_id, end, false)?;
            }
        }
        AstNode::PatternAccess {
            container,
            patterns,
        } => {
            traverse_node(program, source_unit_id, scope_id, container, false)?;
            for pattern in patterns {
                traverse_node(program, source_unit_id, scope_id, pattern, false)?;
            }
        }
        AstNode::PatternCapture { pattern, .. } => {
            traverse_node(program, source_unit_id, scope_id, pattern, false)?;
        }
        AstNode::ContainerLiteral { elements, .. } => {
            for element in elements {
                traverse_node(program, source_unit_id, scope_id, element, false)?;
            }
        }
        AstNode::RecordInit { fields } => {
            for field in fields {
                traverse_node(program, source_unit_id, scope_id, &field.value, false)?;
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
                traverse_node(
                    program,
                    source_unit_id,
                    rolling_scope,
                    &binding.iterable,
                    false,
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
            traverse_node(program, source_unit_id, rolling_scope, expr, false)?;
            if let Some(condition) = condition {
                traverse_node(program, source_unit_id, rolling_scope, condition, false)?;
            }
        }
        AstNode::Range { start, end, .. } => {
            if let Some(start) = start {
                traverse_node(program, source_unit_id, scope_id, start, false)?;
            }
            if let Some(end) = end {
                traverse_node(program, source_unit_id, scope_id, end, false)?;
            }
        }
        AstNode::VarDecl { name, value, .. } => {
            if let AstNode::VarDecl {
                type_hint: Some(type_hint),
                ..
            } = semantic_node(node)
            {
                resolve_type_reference(program, source_unit_id, scope_id, type_hint)?;
            }
            if let Some(value) = value {
                traverse_node(program, source_unit_id, scope_id, value, false)?;
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
                resolve_type_reference(program, source_unit_id, scope_id, type_hint)?;
            }
            if let Some(value) = value {
                traverse_node(program, source_unit_id, scope_id, value, false)?;
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
                resolve_type_reference(program, source_unit_id, scope_id, type_hint)?;
            }
            traverse_node(program, source_unit_id, scope_id, value, false)?;
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
            traverse_node(program, source_unit_id, scope_id, value, false)?;
        }
        AstNode::When {
            expr,
            cases,
            default,
        } => {
            traverse_node(program, source_unit_id, scope_id, expr, false)?;
            for case in cases {
                match case {
                    WhenCase::Case { condition, body } => {
                        traverse_node(program, source_unit_id, scope_id, condition, false)?;
                        traverse_block_body(program, source_unit_id, scope_id, body)?;
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
                        traverse_node(program, source_unit_id, scope_id, value, false)?;
                        traverse_block_body(program, source_unit_id, scope_id, body)?;
                    }
                    WhenCase::Of { body, .. } => {
                        traverse_block_body(program, source_unit_id, scope_id, body)?;
                    }
                }
            }
            if let Some(default_body) = default {
                traverse_block_body(program, source_unit_id, scope_id, default_body)?;
            }
        }
        AstNode::Loop { condition, body } => match condition.as_ref() {
            LoopCondition::Condition(condition) => {
                traverse_node(program, source_unit_id, scope_id, condition, false)?;
                for statement in body {
                    traverse_node(program, source_unit_id, scope_id, statement, false)?;
                }
            }
            LoopCondition::Iteration {
                var,
                iterable,
                condition,
                ..
            } => {
                traverse_node(program, source_unit_id, scope_id, iterable, false)?;
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
                    traverse_node(program, source_unit_id, binder_scope, condition, false)?;
                }
                for statement in body {
                    traverse_node(program, source_unit_id, binder_scope, statement, false)?;
                }
            }
        },
        AstNode::Select { channel, body, .. } => {
            traverse_node(program, source_unit_id, scope_id, channel, false)?;
            traverse_block_body(program, source_unit_id, scope_id, body)?;
        }
        AstNode::Block { statements } => {
            traverse_block_body(program, source_unit_id, scope_id, statements)?;
        }
        AstNode::Program {
            declarations: statements,
        } => {
            for statement in statements {
                traverse_node(program, source_unit_id, scope_id, statement, false)?;
            }
        }
        AstNode::Inquiry { body, .. } => {
            for statement in body {
                traverse_node(program, source_unit_id, scope_id, statement, false)?;
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
                    resolve_type_reference(program, source_unit_id, type_scope, constraint)?;
                }
            }
            for contract in contracts {
                resolve_type_reference(program, source_unit_id, type_scope, contract)?;
            }
            resolve_type_definition(program, source_unit_id, type_scope, type_def)?;
        }
        AstNode::AliasDecl { target, .. } => {
            resolve_type_reference(program, source_unit_id, scope_id, target)?;
        }
        AstNode::DefDecl {
            params, def_type, ..
        } => {
            for param in params {
                resolve_type_reference(program, source_unit_id, scope_id, &param.param_type)?;
            }
            resolve_type_reference(program, source_unit_id, scope_id, def_type)?;
        }
        AstNode::SegDecl { seg_type, .. } => {
            resolve_type_reference(program, source_unit_id, scope_id, seg_type)?;
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
                    resolve_type_reference(program, source_unit_id, impl_scope, constraint)?;
                }
            }
            resolve_type_reference(program, source_unit_id, impl_scope, target)?;
            for member in body {
                traverse_node(program, source_unit_id, impl_scope, member, false)?;
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
                insert_import_record(
                    program,
                    source_unit_id,
                    scope_id,
                    symbol_id,
                    name,
                    path_type.clone(),
                    path_segments.clone(),
                );
            }
        }
    }

    Ok(())
}

fn traverse_block_body(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    parent_scope: ScopeId,
    statements: &[AstNode],
) -> Result<(), ResolverError> {
    let block_scope = program.add_scope(ScopeKind::Block, parent_scope, source_unit_id);
    for statement in statements {
        traverse_node(program, source_unit_id, block_scope, statement, false)?;
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
    if let Some(existing) = program
        .scope(scope_id)
        .and_then(|scope| scope.symbol_keys.get(&canonical_name))
        .into_iter()
        .flat_map(|ids| ids.iter())
        .filter_map(|id| program.symbol(*id))
        .find(|symbol| symbol.duplicate_key == duplicate_key)
    {
        return Err(ResolverError::with_origin(
            ResolverErrorKind::DuplicateSymbol,
            format!(
                "duplicate local symbol '{}' conflicts with existing {:?} declaration",
                name, existing.kind
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
) -> Result<ReferenceId, ResolverError> {
    let symbol_id = resolve_visible_symbol(program, scope_id, name)?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::Identifier,
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

fn record_function_call_reference(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    name: &str,
) -> Result<ReferenceId, ResolverError> {
    let symbol_id = resolve_visible_symbol_of_kinds(
        program,
        scope_id,
        name,
        &[SymbolKind::Routine],
        Some("callable routine"),
    )?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::FunctionCall,
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
    let symbol_id = resolve_qualified_symbol(program, scope_id, path, &[])?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::QualifiedIdentifier,
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
    let symbol_id = resolve_qualified_symbol(program, scope_id, path, &[SymbolKind::Routine])?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::QualifiedFunctionCall,
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
) -> Result<ReferenceId, ResolverError> {
    let symbol_id = resolve_visible_symbol_of_kinds(
        program,
        scope_id,
        name,
        &[
            SymbolKind::Type,
            SymbolKind::Alias,
            SymbolKind::GenericParameter,
        ],
        Some("type"),
    )?;
    let reference_id = program.references.push(ResolvedReference {
        id: ReferenceId(0),
        kind: ReferenceKind::TypeName,
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

fn resolve_visible_symbol(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    name: &str,
) -> Result<SymbolId, ResolverError> {
    resolve_visible_symbol_of_kinds(program, starting_scope, name, &[], None)
}

fn resolve_visible_symbol_of_kinds(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    name: &str,
    allowed_kinds: &[SymbolKind],
    missing_role: Option<&str>,
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
                return Err(ResolverError::new(
                    ResolverErrorKind::AmbiguousReference,
                    format!("name '{}' is ambiguous in lexical scope", name),
                ));
            }

            if allowed_kinds.is_empty() {
                return Err(ResolverError::new(
                    ResolverErrorKind::AmbiguousReference,
                    format!("name '{}' is ambiguous in lexical scope", name),
                ));
            }

            return Err(ResolverError::new(
                ResolverErrorKind::UnresolvedName,
                format!(
                    "could not resolve {} '{}'",
                    missing_role.unwrap_or("name"),
                    name
                ),
            ));
        }

        current_scope = program.scope(scope_id).and_then(|scope| scope.parent);
    }

    Err(ResolverError::new(
        ResolverErrorKind::UnresolvedName,
        format!(
            "could not resolve {} '{}'",
            missing_role.unwrap_or("name"),
            name
        ),
    ))
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
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    typ: &FolType,
) -> Result<(), ResolverError> {
    match typ {
        FolType::Named { name } => {
            record_named_type_reference(program, source_unit_id, scope_id, name)?;
        }
        FolType::Array { element_type, .. }
        | FolType::Vector { element_type }
        | FolType::Sequence { element_type }
        | FolType::Channel { element_type } => {
            resolve_type_reference(program, source_unit_id, scope_id, element_type)?;
        }
        FolType::Matrix { element_type, .. } => {
            resolve_type_reference(program, source_unit_id, scope_id, element_type)?;
        }
        FolType::Set { types } | FolType::Multiple { types } | FolType::Union { types } => {
            for part in types {
                resolve_type_reference(program, source_unit_id, scope_id, part)?;
            }
        }
        FolType::Map {
            key_type,
            value_type,
        } => {
            resolve_type_reference(program, source_unit_id, scope_id, key_type)?;
            resolve_type_reference(program, source_unit_id, scope_id, value_type)?;
        }
        FolType::Record { fields } => {
            for field_type in fields.values() {
                resolve_type_reference(program, source_unit_id, scope_id, field_type)?;
            }
        }
        FolType::Entry { variants } => {
            for variant in variants.values().flatten() {
                resolve_type_reference(program, source_unit_id, scope_id, variant)?;
            }
        }
        FolType::Optional { inner } | FolType::Pointer { target: inner } => {
            resolve_type_reference(program, source_unit_id, scope_id, inner)?;
        }
        FolType::Error { inner } => {
            if let Some(inner) = inner {
                resolve_type_reference(program, source_unit_id, scope_id, inner)?;
            }
        }
        FolType::Limited { base, limits } => {
            resolve_type_reference(program, source_unit_id, scope_id, base)?;
            for limit in limits {
                traverse_node(program, source_unit_id, scope_id, limit, false)?;
            }
        }
        FolType::Function {
            params,
            return_type,
        } => {
            for param in params {
                resolve_type_reference(program, source_unit_id, scope_id, param)?;
            }
            resolve_type_reference(program, source_unit_id, scope_id, return_type)?;
        }
        FolType::Generic { constraints, .. } => {
            for constraint in constraints {
                resolve_type_reference(program, source_unit_id, scope_id, constraint)?;
            }
        }
        FolType::QualifiedNamed { .. } => {}
        FolType::Int { .. }
        | FolType::Float { .. }
        | FolType::Char { .. }
        | FolType::Bool
        | FolType::Never
        | FolType::Any
        | FolType::None
        | FolType::Module { .. }
        | FolType::Block { .. }
        | FolType::Test { .. }
        | FolType::Url { .. }
        | FolType::Location { .. }
        | FolType::Standard { .. } => {}
    }

    Ok(())
}

fn resolve_type_definition(
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
                resolve_type_reference(program, source_unit_id, scope_id, field_type)?;
            }
            for member in members {
                traverse_node(program, source_unit_id, scope_id, member, false)?;
            }
        }
        TypeDefinition::Entry {
            variants, members, ..
        } => {
            for variant_type in variants.values().flatten() {
                resolve_type_reference(program, source_unit_id, scope_id, variant_type)?;
            }
            for member in members {
                traverse_node(program, source_unit_id, scope_id, member, false)?;
            }
        }
        TypeDefinition::Alias { target } => {
            resolve_type_reference(program, source_unit_id, scope_id, target)?;
        }
    }

    Ok(())
}

fn resolve_qualified_symbol(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    path: &QualifiedPath,
    allowed_kinds: &[SymbolKind],
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

    let (mut current_scope, mut current_namespace) =
        resolve_qualified_root(program, starting_scope, &path.segments[0])?;

    for segment in &path.segments[1..path.segments.len() - 1] {
        current_namespace.push_str("::");
        current_namespace.push_str(segment);
        current_scope = program.namespace_scope(&current_namespace).ok_or_else(|| {
            ResolverError::new(
                ResolverErrorKind::UnresolvedName,
                format!("could not resolve qualified path '{}'", path.joined()),
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
    )
}

fn resolve_qualified_root(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    root_segment: &str,
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

    Err(ResolverError::new(
        ResolverErrorKind::UnresolvedName,
        format!("could not resolve qualified path root '{}'", root_segment),
    ))
}

fn resolve_symbol_in_scope(
    program: &ResolvedProgram,
    scope_id: ScopeId,
    name: &str,
    allowed_kinds: &[SymbolKind],
    full_path: &str,
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
        [] => Err(ResolverError::new(
            ResolverErrorKind::UnresolvedName,
            format!("could not resolve qualified path '{}'", full_path),
        )),
        _ => Err(ResolverError::new(
            ResolverErrorKind::AmbiguousReference,
            format!("qualified path '{}' is ambiguous", full_path),
        )),
    }
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
