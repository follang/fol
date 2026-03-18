use crate::{
    require_discovered_root, FrontendCommandResult, FrontendConfig, FrontendError,
    FrontendErrorKind, FrontendResult,
};
use std::path::Path;

fn lower_editor_error(error: fol_editor::EditorError) -> FrontendError {
    let mut frontend = FrontendError::new(FrontendErrorKind::CommandFailed, error.message);
    for note in error.notes {
        frontend = frontend.with_note(note);
    }
    frontend
}

fn editor_summary_to_result(summary: fol_editor::EditorCommandSummary) -> FrontendCommandResult {
    let mut details = summary.details;
    details.sort();
    let rendered = if details.is_empty() {
        summary.summary
    } else {
        format!("{} ({})", summary.summary, details.join(", "))
    };
    FrontendCommandResult::new(summary.command, rendered)
}

pub fn editor_lsp_command(config: &FrontendConfig) -> FrontendResult<FrontendCommandResult> {
    require_discovered_root(&config.working_directory).map_err(|error| {
        if error.kind() == FrontendErrorKind::WorkspaceNotFound {
            error
                .with_note("start the editor inside a FOL package or workspace root")
                .with_note("or open a package directory before launching `fol tool lsp`")
        } else {
            error
        }
    })?;
    fol_editor::editor_lsp_entrypoint()
        .map(editor_summary_to_result)
        .map_err(lower_editor_error)
}

pub fn editor_lsp_stdio(config: &FrontendConfig) -> FrontendResult<()> {
    let _ = config;
    fol_editor::run_lsp_stdio(fol_editor::EditorConfig::default()).map_err(lower_editor_error)
}

pub fn editor_parse_command(
    path: &str,
    _config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    fol_editor::editor_parse_file(Path::new(path))
        .map(editor_summary_to_result)
        .map_err(lower_editor_error)
}

pub fn editor_highlight_command(
    path: &str,
    _config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    fol_editor::editor_highlight_file(Path::new(path))
        .map(editor_summary_to_result)
        .map_err(lower_editor_error)
}

pub fn editor_symbols_command(
    path: &str,
    _config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    fol_editor::editor_symbols_file(Path::new(path))
        .map(editor_summary_to_result)
        .map_err(lower_editor_error)
}

pub fn editor_tree_generate_command(
    path: &str,
    _config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    fol_editor::editor_tree_generate_bundle(Path::new(path))
        .map(editor_summary_to_result)
        .map_err(lower_editor_error)
}

#[cfg(test)]
mod tests {
    use super::{
        editor_highlight_command, editor_lsp_command, editor_parse_command, editor_symbols_command,
        editor_tree_generate_command,
    };
    use crate::{FrontendConfig, FrontendErrorKind};

    #[test]
    fn editor_commands_round_trip_into_frontend_results() {
        let config = FrontendConfig::default();
        let path = "test/apps/fixtures/record_flow/main.fol";

        let lsp = editor_lsp_command(&config).expect("lsp command should succeed");
        let parse = editor_parse_command(path, &config).expect("parse command should succeed");
        let highlight =
            editor_highlight_command(path, &config).expect("highlight command should succeed");
        let symbols =
            editor_symbols_command(path, &config).expect("symbols command should succeed");
        let tree_root = std::env::temp_dir().join("fol_frontend_editor_tree_command_smoke");
        let tree = editor_tree_generate_command(tree_root.to_str().unwrap(), &config)
            .expect("tree generate command should succeed");

        assert_eq!(lsp.command, "lsp");
        assert!(lsp.summary.contains("fol tool lsp"));
        assert_eq!(parse.command, "parse");
        assert!(parse
            .summary
            .contains("path=test/apps/fixtures/record_flow/main.fol"));
        assert_eq!(highlight.command, "highlight");
        assert!(highlight.summary.contains("capture_count="));
        assert!(highlight.summary.contains("captures="));
        assert_eq!(symbols.command, "symbols");
        assert!(symbols.summary.contains("symbol_candidates="));
        assert_eq!(tree.command, "tree generate");
        assert!(tree.summary.contains("tree-sitter bundle ready"));
        std::fs::remove_dir_all(tree_root).ok();
    }

    #[test]
    fn editor_file_commands_wrap_missing_path_failures_as_frontend_errors() {
        let error = editor_parse_command(
            "test/apps/fixtures/record_flow/missing.fol",
            &FrontendConfig::default(),
        )
        .expect_err("missing files should fail");

        assert_eq!(error.kind(), FrontendErrorKind::CommandFailed);
        assert!(error.message().contains("failed to read"));
    }

    #[test]
    fn editor_lsp_command_adds_editor_specific_workspace_guidance() {
        let config = FrontendConfig {
            working_directory: std::env::temp_dir().join(format!(
                "fol_frontend_editor_missing_root_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system time should be after epoch")
                    .as_nanos()
            )),
            ..FrontendConfig::default()
        };
        std::fs::create_dir_all(&config.working_directory).unwrap();

        let error = editor_lsp_command(&config).expect_err("missing roots should fail");

        assert_eq!(error.kind(), FrontendErrorKind::WorkspaceNotFound);
        assert!(error
            .notes()
            .iter()
            .any(|note| note.contains("start the editor inside a FOL package or workspace root")));
        assert!(error
            .notes()
            .iter()
            .any(|note| note.contains("fol tool lsp")));

        std::fs::remove_dir_all(&config.working_directory).ok();
    }
}
