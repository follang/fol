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
use fol_parser::ast::{AstNode, LoopCondition, ParsedTopLevel, WhenCase};

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
            captures,
            params,
            body,
            inquiries,
            ..
        }
        | AstNode::ProDecl {
            syntax_id,
            captures,
            params,
            body,
            inquiries,
            ..
        }
        | AstNode::LogDecl {
            syntax_id,
            captures,
            params,
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
            body,
            inquiries,
            ..
        }
        | AstNode::AnonymousPro {
            captures,
            params,
            body,
            inquiries,
            ..
        }
        | AstNode::AnonymousLog {
            captures,
            params,
            body,
            inquiries,
            ..
        } => {
            let routine_scope = program.add_scope(ScopeKind::Routine, scope_id, source_unit_id);

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
        AstNode::Comment { .. }
        | AstNode::Commented { .. }
        | AstNode::QualifiedIdentifier { .. } => {}
        AstNode::Identifier { name } => {
            record_identifier_reference(program, source_unit_id, scope_id, name)?;
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
        AstNode::QualifiedFunctionCall { args, .. } => {
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
        AstNode::TypeDecl { .. }
        | AstNode::AliasDecl { .. }
        | AstNode::DefDecl { .. }
        | AstNode::SegDecl { .. }
        | AstNode::ImpDecl { .. }
        | AstNode::StdDecl { .. }
        | AstNode::Literal(_)
        | AstNode::PatternWildcard => {}
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
    let symbol_id =
        resolve_visible_symbol_of_kinds(program, scope_id, name, &[SymbolKind::Routine])?;
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

fn resolve_visible_symbol(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    name: &str,
) -> Result<SymbolId, ResolverError> {
    resolve_visible_symbol_of_kinds(program, starting_scope, name, &[])
}

fn resolve_visible_symbol_of_kinds(
    program: &ResolvedProgram,
    starting_scope: ScopeId,
    name: &str,
    allowed_kinds: &[SymbolKind],
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
                format!("could not resolve callable routine '{}'", name),
            ));
        }

        current_scope = program.scope(scope_id).and_then(|scope| scope.parent);
    }

    if allowed_kinds.is_empty() {
        Err(ResolverError::new(
            ResolverErrorKind::UnresolvedName,
            format!("could not resolve name '{}'", name),
        ))
    } else {
        Err(ResolverError::new(
            ResolverErrorKind::UnresolvedName,
            format!("could not resolve callable routine '{}'", name),
        ))
    }
}
