use crate::{
    model::{ResolvedProgram, ResolvedSymbol, SymbolKind},
    ResolverError, ResolverErrorKind, ScopeId, SourceUnitId, SymbolId,
};
use fol_parser::ast::{AstNode, BindingPattern, FolType, ParsedDeclScope, ParsedTopLevel};

pub fn collect_top_level_symbols(program: &mut ResolvedProgram) -> Result<(), Vec<ResolverError>> {
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
        let origin = program.syntax_index().origin(item.node_id).cloned();

        if let Err(error) =
            collect_symbols_from_top_level(program, source_unit_id, scope_id, &item, origin)
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

fn top_level_scope_id(
    program: &ResolvedProgram,
    source_unit_id: SourceUnitId,
    item: &ParsedTopLevel,
) -> ScopeId {
    let source_unit = program
        .source_unit(source_unit_id)
        .expect("collected source unit should exist");

    match item.meta.scope {
        Some(ParsedDeclScope::File) => source_unit.scope_id,
        Some(ParsedDeclScope::Namespace) => program
            .namespace_scope(&source_unit.namespace)
            .expect("namespace-scoped top-level declaration should have a namespace scope"),
        Some(ParsedDeclScope::Package) | None => program.program_scope,
    }
}

fn collect_symbols_from_top_level(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    item: &ParsedTopLevel,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<(), ResolverError> {
    match semantic_node(&item.node) {
        AstNode::VarDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::ValueBinding,
                item,
                origin,
            )?;
        }
        AstNode::LabDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::LabelBinding,
                item,
                origin,
            )?;
        }
        AstNode::DestructureDecl { pattern, .. } => {
            for name in binding_names(pattern) {
                insert_symbol(
                    program,
                    source_unit_id,
                    scope_id,
                    &name,
                    SymbolKind::DestructureBinding,
                    item,
                    origin.clone(),
                )?;
            }
        }
        AstNode::FunDecl { name, .. }
        | AstNode::ProDecl { name, .. }
        | AstNode::LogDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::Routine,
                item,
                origin,
            )?;
        }
        AstNode::TypeDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::Type,
                item,
                origin,
            )?;
        }
        AstNode::AliasDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::Alias,
                item,
                origin,
            )?;
        }
        AstNode::DefDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::Definition,
                item,
                origin,
            )?;
        }
        AstNode::SegDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::Segment,
                item,
                origin,
            )?;
        }
        AstNode::ImpDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::Implementation,
                item,
                origin,
            )?;
        }
        AstNode::StdDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::Standard,
                item,
                origin,
            )?;
        }
        AstNode::UseDecl { name, .. } => {
            insert_symbol(
                program,
                source_unit_id,
                scope_id,
                name,
                SymbolKind::ImportAlias,
                item,
                origin,
            )?;
        }
        AstNode::Comment { .. } => {}
        node => {
            return Err(ResolverError::new(
                ResolverErrorKind::Unsupported,
                format!(
                    "top-level node is not a collectable declaration: {:?}",
                    node
                ),
            ));
        }
    }

    Ok(())
}

fn insert_symbol(
    program: &mut ResolvedProgram,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    name: &str,
    kind: SymbolKind,
    item: &ParsedTopLevel,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> Result<SymbolId, ResolverError> {
    let canonical_name = fol_types::canonical_identifier_key(name);
    let duplicate_key = top_level_duplicate_key(semantic_node(&item.node), &canonical_name);
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
                "duplicate symbol '{}' conflicts with existing {:?} declaration",
                name, existing.kind
            ),
            origin.unwrap_or_else(|| {
                existing
                    .origin
                    .clone()
                    .unwrap_or(fol_parser::ast::SyntaxOrigin {
                        file: None,
                        line: 1,
                        column: 1,
                        length: name.len(),
                    })
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
        origin,
        visibility: item.meta.visibility,
        declaration_scope: item.meta.scope,
    });

    if let Some(symbol) = program.symbols.get_mut(symbol_id) {
        symbol.id = symbol_id;
    }

    let scope = program
        .scopes
        .get_mut(scope_id)
        .expect("top-level symbol target scope should exist");
    scope.symbols.push(symbol_id);
    scope
        .symbol_keys
        .entry(canonical_name)
        .or_default()
        .push(symbol_id);

    Ok(symbol_id)
}

fn semantic_node(node: &AstNode) -> &AstNode {
    match node {
        AstNode::Commented { node, .. } => semantic_node(node),
        _ => node,
    }
}

fn binding_names(pattern: &BindingPattern) -> Vec<String> {
    let mut names = Vec::new();
    collect_binding_names(pattern, &mut names);
    names
}

fn collect_binding_names(pattern: &BindingPattern, output: &mut Vec<String>) {
    match pattern {
        BindingPattern::Name(name) | BindingPattern::Rest(name) => output.push(name.clone()),
        BindingPattern::Sequence(parts) => {
            for part in parts {
                collect_binding_names(part, output);
            }
        }
    }
}

fn top_level_duplicate_key(node: &AstNode, canonical_name: &str) -> String {
    match node {
        AstNode::FunDecl {
            receiver_type,
            params,
            ..
        }
        | AstNode::ProDecl {
            receiver_type,
            params,
            ..
        }
        | AstNode::LogDecl {
            receiver_type,
            params,
            ..
        } => {
            let receiver = receiver_type
                .as_ref()
                .map(fol_type_key)
                .unwrap_or_else(|| "_".to_string());
            let params = params
                .iter()
                .map(|param| fol_type_key(&param.param_type))
                .collect::<Vec<_>>()
                .join(",");
            format!("routine#{}#{}#{}", canonical_name, receiver, params)
        }
        _ => format!("symbol#{}", canonical_name),
    }
}

fn fol_type_key(typ: &FolType) -> String {
    match typ {
        FolType::Named { name } => fol_types::canonical_identifier_key(name),
        FolType::QualifiedNamed { path } => path
            .segments
            .iter()
            .map(|segment| fol_types::canonical_identifier_key(segment))
            .collect::<Vec<_>>()
            .join("::"),
        other => other
            .named_text()
            .map(|text| fol_types::canonical_identifier_key(&text))
            .unwrap_or_else(|| format!("{:?}", other)),
    }
}
