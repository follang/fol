use crate::{
    collect::{semantic_node, top_level_duplicate_key, top_level_scope_id},
    model::{ResolvedProgram, ResolvedSymbol, ScopeKind, SymbolKind},
    ResolverError, ResolverErrorKind, ScopeId, SourceUnitId, SymbolId,
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
        | AstNode::Identifier { .. }
        | AstNode::QualifiedIdentifier { .. } => {}
        AstNode::BinaryOp { left, right, .. } => {
            traverse_node(program, source_unit_id, scope_id, left, false)?;
            traverse_node(program, source_unit_id, scope_id, right, false)?;
        }
        AstNode::UnaryOp { operand, .. } => {
            traverse_node(program, source_unit_id, scope_id, operand, false)?;
        }
        AstNode::FunctionCall { args, .. }
        | AstNode::QualifiedFunctionCall { args, .. }
        | AstNode::Invoke { args, .. } => {
            for arg in args {
                traverse_node(program, source_unit_id, scope_id, arg, false)?;
            }
        }
        AstNode::NamedArgument { value, .. }
        | AstNode::Unpack { value }
        | AstNode::Spawn { task: value }
        | AstNode::Assignment { value, .. }
        | AstNode::Return { value: Some(value) } => {
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
            for binding in bindings {
                traverse_node(program, source_unit_id, scope_id, &binding.iterable, false)?;
            }
            traverse_node(program, source_unit_id, scope_id, expr, false)?;
            if let Some(condition) = condition {
                traverse_node(program, source_unit_id, scope_id, condition, false)?;
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
        AstNode::VarDecl { value, .. } | AstNode::LabDecl { value, .. } => {
            if let Some(value) = value {
                traverse_node(program, source_unit_id, scope_id, value, false)?;
            }
        }
        AstNode::DestructureDecl { value, .. } | AstNode::Yield { value } => {
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
                        for statement in body {
                            traverse_node(program, source_unit_id, scope_id, statement, false)?;
                        }
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
                        for statement in body {
                            traverse_node(program, source_unit_id, scope_id, statement, false)?;
                        }
                    }
                    WhenCase::Of { body, .. } => {
                        for statement in body {
                            traverse_node(program, source_unit_id, scope_id, statement, false)?;
                        }
                    }
                }
            }
            if let Some(default_body) = default {
                for statement in default_body {
                    traverse_node(program, source_unit_id, scope_id, statement, false)?;
                }
            }
        }
        AstNode::Loop { condition, body } => {
            match condition.as_ref() {
                LoopCondition::Condition(condition) => {
                    traverse_node(program, source_unit_id, scope_id, condition, false)?;
                }
                LoopCondition::Iteration {
                    iterable,
                    condition,
                    ..
                } => {
                    traverse_node(program, source_unit_id, scope_id, iterable, false)?;
                    if let Some(condition) = condition {
                        traverse_node(program, source_unit_id, scope_id, condition, false)?;
                    }
                }
            }
            for statement in body {
                traverse_node(program, source_unit_id, scope_id, statement, false)?;
            }
        }
        AstNode::Select { channel, body, .. } => {
            traverse_node(program, source_unit_id, scope_id, channel, false)?;
            for statement in body {
                traverse_node(program, source_unit_id, scope_id, statement, false)?;
            }
        }
        AstNode::Block { statements }
        | AstNode::Program {
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
        | AstNode::UseDecl { .. }
        | AstNode::AliasDecl { .. }
        | AstNode::DefDecl { .. }
        | AstNode::SegDecl { .. }
        | AstNode::ImpDecl { .. }
        | AstNode::StdDecl { .. }
        | AstNode::Literal(_)
        | AstNode::PatternWildcard => {}
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
