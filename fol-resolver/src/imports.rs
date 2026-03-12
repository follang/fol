use crate::{ResolvedProgram, ResolverError, ResolverErrorKind, ScopeId};
use crate::model::ScopeKind;
use fol_parser::ast::FolType;
use std::collections::BTreeSet;

pub fn resolve_import_targets(program: &mut ResolvedProgram) -> Result<(), Vec<ResolverError>> {
    let mut errors = Vec::new();
    let import_ids = program
        .imports
        .iter_with_ids()
        .map(|(import_id, _)| import_id)
        .collect::<Vec<_>>();

    for import_id in import_ids {
        let Some(import) = program.import(import_id).cloned() else {
            continue;
        };

        match &import.path_type {
            FolType::Location { .. } => match resolve_location_target(program, &import) {
                Ok(target_scope) => {
                    if let Some(import_slot) = program.imports.get_mut(import_id) {
                        import_slot.target_scope = Some(target_scope);
                    }
                }
                Err(error) => errors.push(error),
            },
            _ => {
                let origin = program
                    .symbol(import.alias_symbol)
                    .and_then(|symbol| symbol.origin.clone());
                let message = format!(
                    "resolver does not support '{}' imports yet",
                    import_kind_label(&import.path_type)
                );
                match origin {
                    Some(origin) => errors.push(ResolverError::with_origin(
                        ResolverErrorKind::Unsupported,
                        message,
                        origin,
                    )),
                    None => {
                        errors.push(ResolverError::new(ResolverErrorKind::Unsupported, message))
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn resolve_location_target(
    program: &ResolvedProgram,
    import: &crate::ResolvedImport,
) -> Result<ScopeId, ResolverError> {
    let source_unit = program
        .source_unit(import.source_unit)
        .expect("import source unit should exist while resolving imports");
    let package = &source_unit.package;
    let relative_suffix = import
        .path_segments
        .iter()
        .map(|segment| segment.spelling.clone())
        .collect::<Vec<_>>();
    let joined = relative_suffix.join("::");
    let mut candidate_names = BTreeSet::new();

    if !joined.is_empty() {
        candidate_names.insert(joined.clone());
        candidate_names.insert(format!("{package}::{joined}"));
    }

    let mut candidate_scopes = candidate_names
        .into_iter()
        .filter_map(|namespace| namespace_scope_if_present(program, &namespace))
        .collect::<Vec<_>>();
    candidate_scopes.sort_unstable();
    candidate_scopes.dedup();

    match candidate_scopes.as_slice() {
        [scope_id] => Ok(*scope_id),
        [] => Err(import_error(
            program,
            import.alias_symbol,
            ResolverErrorKind::UnresolvedName,
            format!("could not resolve local import target '{}'", joined),
        )),
        _ => Err(import_error(
            program,
            import.alias_symbol,
            ResolverErrorKind::AmbiguousReference,
            format!(
                "local import target '{}' is ambiguous; candidates: {}",
                joined,
                candidate_scopes
                    .iter()
                    .map(|scope_id| describe_import_target(program, *scope_id))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        )),
    }
}

fn namespace_scope_if_present(program: &ResolvedProgram, namespace: &str) -> Option<ScopeId> {
    if namespace == program.package_name() {
        Some(program.program_scope)
    } else {
        program.namespace_scope(namespace)
    }
}

fn import_error(
    program: &ResolvedProgram,
    alias_symbol: crate::SymbolId,
    kind: ResolverErrorKind,
    message: String,
) -> ResolverError {
    match program
        .symbol(alias_symbol)
        .and_then(|symbol| symbol.origin.clone())
    {
        Some(origin) => ResolverError::with_origin(kind, message, origin),
        None => ResolverError::new(kind, message),
    }
}

fn import_kind_label(path_type: &FolType) -> &'static str {
    match path_type {
        FolType::Location { .. } => "loc",
        FolType::Module { .. } => "mod",
        FolType::Standard { .. } => "std",
        FolType::Url { .. } => "url",
        _ => "unknown",
    }
}

fn describe_import_target(program: &ResolvedProgram, scope_id: ScopeId) -> String {
    match program.scope(scope_id).map(|scope| &scope.kind) {
        Some(ScopeKind::ProgramRoot { package }) => format!("package '{package}'"),
        Some(ScopeKind::NamespaceRoot { namespace }) => format!("namespace '{namespace}'"),
        Some(ScopeKind::SourceUnitRoot { path }) => format!("source unit '{path}'"),
        Some(other) => format!("scope {other:?}"),
        None => "unknown scope".to_string(),
    }
}
