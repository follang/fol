use crate::{
    FrontendCommandResult, FrontendConfig, FrontendError, FrontendErrorKind, FrontendResult,
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

pub fn editor_lsp_command() -> FrontendResult<FrontendCommandResult> {
    fol_editor::editor_lsp_entrypoint()
        .map(editor_summary_to_result)
        .map_err(lower_editor_error)
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

#[cfg(test)]
mod tests {
    use super::{
        editor_highlight_command, editor_lsp_command, editor_parse_command, editor_symbols_command,
    };
    use crate::{FrontendConfig, FrontendErrorKind};

    #[test]
    fn editor_commands_round_trip_into_frontend_results() {
        let config = FrontendConfig::default();
        let path = "test/apps/fixtures/record_flow/main.fol";

        let lsp = editor_lsp_command().expect("lsp command should succeed");
        let parse = editor_parse_command(path, &config).expect("parse command should succeed");
        let highlight =
            editor_highlight_command(path, &config).expect("highlight command should succeed");
        let symbols =
            editor_symbols_command(path, &config).expect("symbols command should succeed");

        assert_eq!(lsp.command, "lsp");
        assert!(lsp.summary.contains("fol editor lsp"));
        assert_eq!(parse.command, "parse");
        assert!(parse.summary.contains("path=test/apps/fixtures/record_flow/main.fol"));
        assert_eq!(highlight.command, "highlight");
        assert!(highlight.summary.contains("keyword_hits="));
        assert_eq!(symbols.command, "symbols");
        assert!(symbols.summary.contains("symbol_candidates="));
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
}
