use crate::{
    fol_tree_sitter_corpus, fol_tree_sitter_grammar, fol_tree_sitter_highlights_query,
    fol_tree_sitter_locals_query, fol_tree_sitter_query_snapshots, fol_tree_sitter_symbols_query,
    EditorError, EditorErrorKind, EditorResult,
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
        "ready to serve editor requests through `fol tool lsp`",
    )
    .with_detail("transport/runtime wiring lands in the LSP foundation phase"))
}

fn source_line_count(source: &str) -> usize {
    source.lines().count()
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
    .with_detail(format!("lines={}", source_line_count(&source)))
    .with_detail(format!("bytes={}", source.len()))
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
    .with_detail(format!("lines={}", source_line_count(&source)))
    .with_detail(format!("query_bytes={}", query.len()))
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
    .with_detail(format!("lines={}", source_line_count(&source)))
    .with_detail(format!("symbol_candidates={symbol_count}"))
    .with_detail(format!("query_snapshots={}", fol_tree_sitter_query_snapshots().len())))
}

pub fn editor_tree_generate_bundle(path: &Path) -> EditorResult<EditorCommandSummary> {
    if path.exists() && !path.is_dir() {
        return Err(EditorError::new(
            EditorErrorKind::InvalidDocumentPath,
            format!("tree output root '{}' is not a directory", path.display()),
        ));
    }

    let queries_root = path.join("queries").join("fol");
    let corpus_root = path.join("test").join("corpus");
    std::fs::create_dir_all(&queries_root).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to create query root '{}': {error}", queries_root.display()),
        )
    })?;
    std::fs::create_dir_all(&corpus_root).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to create corpus root '{}': {error}", corpus_root.display()),
        )
    })?;

    write_bundle_file(&path.join("grammar.js"), fol_tree_sitter_grammar())?;
    write_bundle_file(
        &queries_root.join("highlights.scm"),
        fol_tree_sitter_highlights_query(),
    )?;
    write_bundle_file(&queries_root.join("locals.scm"), fol_tree_sitter_locals_query())?;
    write_bundle_file(&queries_root.join("symbols.scm"), fol_tree_sitter_symbols_query())?;
    write_bundle_file(&path.join("package.json"), TREE_SITTER_PACKAGE_JSON)?;

    for case in fol_tree_sitter_corpus() {
        write_bundle_file(&corpus_root.join(format!("{}.txt", case.name)), case.source)?;
    }

    let mut summary = EditorCommandSummary::new(
        "tree generate",
        format!("tree-sitter bundle ready at {}", path.display()),
    )
    .with_detail(format!("root={}", path.display()))
    .with_detail(format!("query_files={}", fol_tree_sitter_query_snapshots().len()))
    .with_detail(format!("corpus_files={}", fol_tree_sitter_corpus().len()))
    .with_detail(format!("grammar_bytes={}", fol_tree_sitter_grammar().len()));

    match std::process::Command::new("tree-sitter")
        .arg("generate")
        .current_dir(path)
        .status()
    {
        Ok(status) if status.success() => {
            let parser_path = path.join("src").join("parser.c");
            summary = summary.with_detail(format!("parser_generated={}", parser_path.is_file()));
            if parser_path.is_file() {
                summary = summary.with_detail(format!("parser={}", parser_path.display()));
            }
        }
        Ok(status) => {
            summary = summary
                .with_detail("parser_generated=false")
                .with_detail(format!("tree_sitter_status={status}"));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            summary = summary
                .with_detail("parser_generated=false")
                .with_detail("tree_sitter_cli=missing");
        }
        Err(error) => {
            summary = summary
                .with_detail("parser_generated=false")
                .with_detail(format!("tree_sitter_error={error}"));
        }
    }

    Ok(summary)
}

const TREE_SITTER_PACKAGE_JSON: &str = r#"{
  "name": "tree-sitter-fol",
  "version": "0.1.0",
  "private": true,
  "grammars": [
    {
      "name": "fol",
      "scope": "source.fol",
      "file-types": ["fol"]
    }
  ]
}
"#;

fn write_bundle_file(path: &Path, contents: &str) -> EditorResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            EditorError::new(
                EditorErrorKind::Internal,
                format!("failed to create '{}' : {error}", parent.display()),
            )
        })?;
    }
    std::fs::write(path, contents).map_err(|error| {
        EditorError::new(
            EditorErrorKind::Internal,
            format!("failed to write '{}': {error}", path.display()),
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        editor_highlight_file, editor_lsp_entrypoint, editor_parse_file, editor_symbols_file,
        editor_tree_generate_bundle,
    };
    use crate::{fol_tree_sitter_grammar, fol_tree_sitter_query_snapshots};
    use std::path::PathBuf;

    #[test]
    fn lsp_entrypoint_summary_is_stable() {
        let summary = editor_lsp_entrypoint().unwrap();
        assert_eq!(summary.command, "lsp");
        assert!(summary.summary.contains("fol tool lsp"));
    }

    #[test]
    fn file_backed_editor_commands_report_path_and_shape() {
        let path = PathBuf::from("test/apps/fixtures/record_flow/main.fol");
        let parse = editor_parse_file(&path).unwrap();
        let highlight = editor_highlight_file(&path).unwrap();
        let symbols = editor_symbols_file(&path).unwrap();

        assert!(parse.details.iter().any(|detail| detail.contains("path=")));
        assert!(parse.details.iter().any(|detail| detail.contains("lines=")));
        assert!(highlight
            .details
            .iter()
            .any(|detail| detail.contains("keyword_hits=")));
        assert!(symbols
            .details
            .iter()
            .any(|detail| detail.contains("symbol_candidates=")));
    }

    #[test]
    fn real_fixtures_keep_editor_command_summaries_stable() {
        let showcase = PathBuf::from("test/apps/showcases/full_v1_showcase/app/main.fol");
        let package = PathBuf::from("xtra/logtiny/src/log.fol");

        let parse = editor_parse_file(&showcase).unwrap();
        let highlight = editor_highlight_file(&showcase).unwrap();
        let symbols = editor_symbols_file(&package).unwrap();

        assert_eq!(parse.command, "parse");
        assert_eq!(
            parse.details,
            vec![
                "path=test/apps/showcases/full_v1_showcase/app/main.fol".to_string(),
                "lines=98".to_string(),
                "bytes=2094".to_string(),
                format!("grammar_bytes={}", fol_tree_sitter_grammar().len()),
            ]
        );
        assert_eq!(highlight.command, "highlight");
        assert_eq!(
            highlight.details,
            vec![
                "path=test/apps/showcases/full_v1_showcase/app/main.fol".to_string(),
                "lines=98".to_string(),
                format!("query_bytes={}", crate::fol_tree_sitter_highlights_query().len()),
                "keyword_hits=19".to_string(),
            ]
        );
        assert_eq!(symbols.command, "symbols");
        assert_eq!(
            symbols.details,
            vec![
                "path=xtra/logtiny/src/log.fol".to_string(),
                "lines=52".to_string(),
                "symbol_candidates=8".to_string(),
                format!("query_snapshots={}", fol_tree_sitter_query_snapshots().len()),
            ]
        );
    }

    #[test]
    fn tree_generate_bundle_writes_editor_consumable_assets() {
        let root = std::env::temp_dir().join(format!(
            "fol_editor_tree_bundle_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ));
        let summary = editor_tree_generate_bundle(&root).unwrap();

        assert_eq!(summary.command, "tree generate");
        assert!(root.join("grammar.js").is_file());
        assert!(root.join("queries/fol/highlights.scm").is_file());
        assert!(root.join("queries/fol/locals.scm").is_file());
        assert!(root.join("queries/fol/symbols.scm").is_file());
        assert!(root.join("test/corpus/declarations.txt").is_file());
        assert!(summary.details.iter().any(|detail| detail.contains("query_files=3")));
        assert!(summary
            .details
            .iter()
            .any(|detail| detail.contains("parser_generated=")));

        std::fs::remove_dir_all(root).ok();
    }
}
