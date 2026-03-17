use crate::{
    fol_tree_sitter_grammar, fol_tree_sitter_highlights_query, fol_tree_sitter_query_snapshots,
    fol_tree_sitter_symbols_query, EditorError, EditorErrorKind, EditorResult,
};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorCommandSummary {
    pub command: String,
    pub summary: String,
    pub details: Vec<String>,
}

impl EditorCommandSummary {
    pub fn new(command: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            summary: summary.into(),
            details: Vec::new(),
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }
}

pub fn editor_lsp_entrypoint() -> EditorResult<EditorCommandSummary> {
    Ok(EditorCommandSummary::new(
        "lsp",
        "ready to serve editor requests through `fol editor lsp`",
    )
    .with_detail("transport/runtime wiring lands in the LSP foundation phase"))
}

pub fn editor_parse_file(path: &Path) -> EditorResult<EditorCommandSummary> {
    let source = std::fs::read_to_string(path).map_err(|error| {
        EditorError::new(
            EditorErrorKind::InvalidDocumentPath,
            format!("failed to read '{}': {error}", path.display()),
        )
    })?;
    Ok(EditorCommandSummary::new(
        "parse",
        format!("loaded {} bytes for tree-sitter parsing", source.len()),
    )
    .with_detail(format!("path={}", path.display()))
    .with_detail(format!("grammar_bytes={}", fol_tree_sitter_grammar().len())))
}

pub fn editor_highlight_file(path: &Path) -> EditorResult<EditorCommandSummary> {
    let source = std::fs::read_to_string(path).map_err(|error| {
        EditorError::new(
            EditorErrorKind::InvalidDocumentPath,
            format!("failed to read '{}': {error}", path.display()),
        )
    })?;
    let query = fol_tree_sitter_highlights_query();
    let keyword_hits = source
        .matches("fun")
        .count()
        + source.matches("typ").count()
        + source.matches("var").count();
    Ok(EditorCommandSummary::new(
        "highlight",
        format!("highlight query ready with {} bytes", query.len()),
    )
    .with_detail(format!("path={}", path.display()))
    .with_detail(format!("keyword_hits={keyword_hits}")))
}

pub fn editor_symbols_file(path: &Path) -> EditorResult<EditorCommandSummary> {
    let source = std::fs::read_to_string(path).map_err(|error| {
        EditorError::new(
            EditorErrorKind::InvalidDocumentPath,
            format!("failed to read '{}': {error}", path.display()),
        )
    })?;
    let symbol_count = source.matches("fun ").count()
        + source.matches("log ").count()
        + source.matches("typ ").count()
        + source.matches("ali ").count();
    Ok(EditorCommandSummary::new(
        "symbols",
        format!("symbol query ready with {} bytes", fol_tree_sitter_symbols_query().len()),
    )
    .with_detail(format!("path={}", path.display()))
    .with_detail(format!("symbol_candidates={symbol_count}"))
    .with_detail(format!("query_snapshots={}", fol_tree_sitter_query_snapshots().len())))
}

#[cfg(test)]
mod tests {
    use super::{
        editor_highlight_file, editor_lsp_entrypoint, editor_parse_file, editor_symbols_file,
    };
    use std::path::PathBuf;

    #[test]
    fn lsp_entrypoint_summary_is_stable() {
        let summary = editor_lsp_entrypoint().unwrap();
        assert_eq!(summary.command, "lsp");
        assert!(summary.summary.contains("fol editor lsp"));
    }

    #[test]
    fn file_backed_editor_commands_report_path_and_shape() {
        let path = PathBuf::from("test/apps/fixtures/record_flow/main.fol");
        assert!(editor_parse_file(&path).unwrap().details[0].contains("path="));
        assert!(editor_highlight_file(&path).unwrap().details[1].contains("keyword_hits="));
        assert!(editor_symbols_file(&path).unwrap().details[1].contains("symbol_candidates="));
    }
}
