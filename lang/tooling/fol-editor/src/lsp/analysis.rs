use crate::{
    diagnostic_to_lsp, materialize_analysis_overlay, EditorDocument,
    EditorError, EditorErrorKind, EditorResult, EditorWorkspaceMapping, LspDiagnostic,
};
use fol_diagnostics::Diagnostic;
use fol_diagnostics::ToDiagnostic;
use fol_package::{PackageError, PackageSession, PackageSourceKind};
use fol_parser::ast::{AstParser, ParseError};
use fol_resolver::{Resolver, ResolverError};
use fol_stream::{FileStream, Source, SourceType};
use fol_typecheck::{TypecheckError, Typechecker};
use std::collections::HashSet;
use std::path::Path;

use super::semantic::SemanticSnapshot;

pub(super) fn analyze_document(
    document: &EditorDocument,
    mapping: &EditorWorkspaceMapping,
) -> EditorResult<Vec<LspDiagnostic>> {
    let snapshot = analyze_document_semantics(document, mapping)?;
    Ok(dedup_diagnostics(snapshot.diagnostics))
}

/// Deduplicate LSP diagnostics by (line, code), keeping only the first
/// diagnostic for each unique pair. This prevents cascade errors from
/// flooding the editor with redundant markers on the same line.
///
/// This is the editor-layer dedup. The compiler-layer dedup in
/// `DiagnosticReport::add_diagnostic` handles consecutive same-code +
/// same-line suppression and a hard cap at 50. Both layers are intentional:
/// the compiler catches cascades at production time, the editor catches
/// cross-stage duplicates after conversion.
pub(crate) fn dedup_diagnostics(diagnostics: Vec<LspDiagnostic>) -> Vec<LspDiagnostic> {
    let mut seen = HashSet::new();
    diagnostics
        .into_iter()
        .filter(|d| seen.insert((d.range.start.line, d.code.clone())))
        .collect()
}

pub(super) fn analyze_document_semantics(
    document: &EditorDocument,
    mapping: &EditorWorkspaceMapping,
) -> EditorResult<SemanticSnapshot> {
    let overlay = materialize_analysis_overlay(mapping, document)?;
    if let Some(package_root) = overlay.package_root() {
        let parser_diags = parse_directory_diagnostics(package_root)?
            .into_iter()
            .filter(|diagnostic| diagnostic_targets_path(diagnostic, overlay.document_path()))
            .map(|diagnostic| diagnostic_to_lsp(&diagnostic))
            .collect::<Vec<_>>();
        if !parser_diags.is_empty() {
            return Ok(SemanticSnapshot {
                analyzed_path: Some(overlay.document_path().to_path_buf()),
                source_document_path: mapping.document_path.clone(),
                source_package_root: mapping.package_root.clone(),
                diagnostics: parser_diags,
                resolved_workspace: None,
                typed_workspace: None,
            });
        }

        let mut package_session = PackageSession::new();
        let prepared =
            match package_session.load_directory_package(package_root, PackageSourceKind::Entry) {
                Ok(prepared) => prepared,
                Err(error) => {
                    return Ok(SemanticSnapshot {
                        analyzed_path: Some(overlay.document_path().to_path_buf()),
                        source_document_path: mapping.document_path.clone(),
                        source_package_root: mapping.package_root.clone(),
                        diagnostics: vec![diagnostic_to_lsp(&error.to_diagnostic())],
                        resolved_workspace: None,
                        typed_workspace: None,
                    })
                }
            };

        let mut resolver = Resolver::new();
        let resolved = match resolver.resolve_prepared_workspace(prepared) {
            Ok(resolved) => resolved,
            Err(errors) => {
                return Ok(SemanticSnapshot {
                    analyzed_path: Some(overlay.document_path().to_path_buf()),
                    source_document_path: mapping.document_path.clone(),
                    source_package_root: mapping.package_root.clone(),
                    diagnostics: errors
                        .iter()
                        .map(|error| error.to_diagnostic())
                        .filter(|diagnostic| {
                            diagnostic_targets_path(diagnostic, overlay.document_path())
                        })
                        .map(|diagnostic| diagnostic_to_lsp(&diagnostic))
                        .collect(),
                    resolved_workspace: None,
                    typed_workspace: None,
                })
            }
        };

        let mut typechecker = Typechecker::new();
        match typechecker.check_resolved_workspace(resolved.clone()) {
            Ok(typed_workspace) => Ok(SemanticSnapshot {
                analyzed_path: Some(overlay.document_path().to_path_buf()),
                source_document_path: mapping.document_path.clone(),
                source_package_root: mapping.package_root.clone(),
                diagnostics: Vec::new(),
                resolved_workspace: Some(resolved),
                typed_workspace: Some(typed_workspace),
            }),
            Err(errors) => Ok(SemanticSnapshot {
                analyzed_path: Some(overlay.document_path().to_path_buf()),
                source_document_path: mapping.document_path.clone(),
                source_package_root: mapping.package_root.clone(),
                diagnostics: errors
                    .iter()
                    .map(|error| error.to_diagnostic())
                    .filter(|diagnostic| {
                        diagnostic_targets_path(diagnostic, overlay.document_path())
                    })
                    .map(|diagnostic| diagnostic_to_lsp(&diagnostic))
                    .collect(),
                resolved_workspace: Some(resolved),
                typed_workspace: None,
            }),
        }
    } else {
        Ok(SemanticSnapshot {
            analyzed_path: Some(mapping.document_path.clone()),
            source_document_path: mapping.document_path.clone(),
            source_package_root: mapping.package_root.clone(),
            diagnostics: parse_single_file_diagnostics(&mapping.document_path, &document.text)?
                .into_iter()
                .filter(|diagnostic| diagnostic_targets_path(diagnostic, &mapping.document_path))
                .map(|diagnostic| diagnostic_to_lsp(&diagnostic))
                .collect(),
            resolved_workspace: None,
            typed_workspace: None,
        })
    }
}

pub(super) fn diagnostic_targets_path(diagnostic: &Diagnostic, path: &Path) -> bool {
    let path_text = path.to_string_lossy();
    diagnostic
        .primary_location()
        .and_then(|location| location.file.as_ref())
        .map(|file| file == &path_text)
        .or_else(|| {
            diagnostic
                .labels
                .first()
                .and_then(|label| label.location.file.as_ref())
                .map(|file| file == &path_text)
        })
        .unwrap_or(false)
}

/// Parse a single file for diagnostics when no package root is available.
///
/// Current implementation writes the file to a temp directory on disk and
/// parses via `parse_directory_diagnostics`. This is a known weakness: future
/// work (Slice 3) should replace this with in-memory source/stream construction.
pub(super) fn parse_single_file_diagnostics(
    path: &Path,
    text: &str,
) -> EditorResult<Vec<Diagnostic>> {
    let root = std::env::temp_dir().join(format!(
        "fol_editor_parse_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!(
                "failed to create parser temp root '{}': {error}",
                root.display()
            ),
        )
    })?;
    let file = root.join(
        path.file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("main.fol")),
    );
    std::fs::write(&file, text).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!(
                "failed to write parser temp file '{}': {error}",
                file.display()
            ),
        )
    })?;
    let diagnostics = parse_directory_diagnostics(&root)?;
    let _ = std::fs::remove_dir_all(&root);
    Ok(diagnostics)
}

pub(super) fn parse_directory_diagnostics(root: &Path) -> EditorResult<Vec<Diagnostic>> {
    let root_str = root.to_str().ok_or_else(|| {
        EditorError::new(
            EditorErrorKind::InvalidDocumentPath,
            format!("analysis root '{}' is not valid UTF-8", root.display()),
        )
    })?;
    let display_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("root");
    let sources =
        Source::init_with_package(root_str, SourceType::Folder, display_name).map_err(|error| {
            EditorError::new(
                EditorErrorKind::Internal,
                format!(
                    "failed to initialize analysis sources from '{}': {error}",
                    root.display()
                ),
            )
        })?;
    let mut stream = FileStream::from_sources(sources).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!(
                "failed to read analysis sources from '{}': {error}",
                root.display()
            ),
        )
    })?;
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = AstParser::new();

    match parser.parse_package(&mut lexer) {
        Ok(_) => Ok(Vec::new()),
        Err(errors) => Ok(errors
            .into_iter()
            .map(|error| glitch_to_diagnostic(error.as_ref()))
            .collect()),
    }
}

/// Convert a `dyn Glitch` to a `Diagnostic` via manual downcast chain.
///
/// This is a known weakness: the chain currently handles ParseError,
/// PackageError, ResolverError, and TypecheckError. Missing: LoweringError
/// and BuildEvaluationError. Unknown types fall back to `E9999`.
///
/// Future work (Slice 3) should replace this with trait-based dispatch
/// so that all `Glitch` implementors produce diagnostics at their
/// production site instead of requiring a central downcast.
pub(super) fn glitch_to_diagnostic(error: &dyn fol_types::Glitch) -> Diagnostic {
    if let Some(parse_error) = error.as_any().downcast_ref::<ParseError>() {
        return parse_error.to_diagnostic();
    }
    if let Some(package_error) = error.as_any().downcast_ref::<PackageError>() {
        return package_error.to_diagnostic();
    }
    if let Some(resolver_error) = error.as_any().downcast_ref::<ResolverError>() {
        return resolver_error.to_diagnostic();
    }
    if let Some(typecheck_error) = error.as_any().downcast_ref::<TypecheckError>() {
        return typecheck_error.to_diagnostic();
    }
    fol_diagnostics::Diagnostic::error("E9999", error.to_string())
}

pub(super) fn syntax_at_position(
    program: &fol_resolver::ResolvedProgram,
    path: &Path,
    position: crate::LspPosition,
) -> Option<fol_parser::ast::SyntaxNodeId> {
    let path_text = path.to_string_lossy();
    let mut best: Option<(fol_parser::ast::SyntaxNodeId, usize)> = None;
    for index in 0..program.syntax_index().len() {
        let syntax_id = fol_parser::ast::SyntaxNodeId(index);
        let Some(origin) = program.syntax_index().origin(syntax_id) else {
            continue;
        };
        let Some(file) = &origin.file else {
            continue;
        };
        if file != &path_text {
            continue;
        }
        let start_line = origin.line.saturating_sub(1) as u32;
        let start_character = origin.column.saturating_sub(1) as u32;
        let end_character = start_character + origin.length.max(1) as u32;
        let contains = position.line == start_line
            && position.character >= start_character
            && position.character <= end_character;
        if contains {
            match best {
                Some((_, current_len)) if current_len <= origin.length => {}
                _ => best = Some((syntax_id, origin.length)),
            }
        }
    }
    best.map(|(syntax_id, _)| syntax_id)
}

pub(super) fn nearest_scope_before_position(
    program: &fol_resolver::ResolvedProgram,
    path: &Path,
    position: crate::LspPosition,
) -> Option<fol_resolver::ScopeId> {
    let path_text = path.to_string_lossy();
    let mut best: Option<((u32, u32), fol_resolver::ScopeId)> = None;
    for index in 0..program.syntax_index().len() {
        let syntax_id = fol_parser::ast::SyntaxNodeId(index);
        let Some(scope_id) = program.scope_for_syntax(syntax_id) else {
            continue;
        };
        let Some(origin) = program.syntax_index().origin(syntax_id) else {
            continue;
        };
        let Some(file) = &origin.file else {
            continue;
        };
        if file != &path_text {
            continue;
        }
        let start = (
            origin.line.saturating_sub(1) as u32,
            origin.column.saturating_sub(1) as u32,
        );
        let cursor = (position.line, position.character);
        if start > cursor {
            continue;
        }
        match best {
            Some((best_start, _)) if best_start >= start => {}
            _ => best = Some((start, scope_id)),
        }
    }
    best.map(|(_, scope_id)| scope_id)
}
