use crate::{
    errors::symbol_kind_label,
    model::{ResolvedProgram, ResolvedSymbol, ScopeKind, SymbolKind},
    ResolverError, ResolverErrorKind, ScopeId, SymbolId,
};
use fol_parser::ast::ParsedDeclVisibility;
use fol_parser::ast::QualifiedPath;

pub fn resolve_visible_symbol(
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

pub fn resolve_lexical_symbol_of_kinds(
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

pub fn resolve_imported_symbol_of_kinds(
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

pub fn resolve_visible_or_imported_symbol_of_kinds(
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

pub fn resolve_visible_symbol_of_kinds(
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

pub fn resolve_qualified_symbol(
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

pub fn error_with_optional_origin(
    kind: ResolverErrorKind,
    message: String,
    origin: Option<fol_parser::ast::SyntaxOrigin>,
) -> ResolverError {
    match origin {
        Some(origin) => ResolverError::with_origin(kind, message, origin),
        None => ResolverError::new(kind, message),
    }
}

pub fn ambiguity_error_with_optional_origin(
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

pub fn qualified_path_origin(
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
    use crate::errors::format_origin_brief;
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
