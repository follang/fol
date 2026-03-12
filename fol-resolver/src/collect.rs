use crate::{
    model::{ResolvedProgram, ResolvedSymbol, SymbolKind},
    ResolverError, ResolverErrorKind, ScopeId, SourceUnitId, SymbolId,
};
use fol_parser::ast::{AstNode, BindingPattern, ParsedDeclScope, ParsedTopLevel};

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

        if let Err(error_list) =
            collect_symbols_from_top_level(program, source_unit_id, scope_id, &item, origin)
        {
            errors.extend(error_list);
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
) -> Result<(), Vec<ResolverError>> {
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
            );
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
            );
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
                );
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
            );
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
            );
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
            );
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
            );
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
            );
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
            );
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
            );
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
            );
        }
        AstNode::Comment { .. } => {}
        node => {
            return Err(vec![ResolverError::new(
                ResolverErrorKind::Unsupported,
                format!(
                    "top-level node is not a collectable declaration: {:?}",
                    node
                ),
            )]);
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
) -> SymbolId {
    let canonical_name = fol_types::canonical_identifier_key(name);
    let symbol_id = program.symbols.push(ResolvedSymbol {
        id: SymbolId(0),
        name: name.to_string(),
        canonical_name: canonical_name.clone(),
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

    symbol_id
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
