use crate::{ResolvedProgram, ResolverError, ResolverErrorKind};
use fol_parser::ast::FolType;

pub fn validate_supported_import_kinds(
    program: &ResolvedProgram,
) -> Result<(), Vec<ResolverError>> {
    let mut errors = Vec::new();

    for import in program.imports.iter() {
        if matches!(import.path_type, FolType::Location { .. }) {
            continue;
        }

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
            None => errors.push(ResolverError::new(ResolverErrorKind::Unsupported, message)),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
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
