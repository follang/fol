use crate::{FrontendCommandResult, FrontendError, FrontendOutputConfig, OutputMode};
use colored::Colorize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontendOutput {
    config: FrontendOutputConfig,
}

impl FrontendOutput {
    pub fn new(config: FrontendOutputConfig) -> Self {
        colored::control::set_override(matches!(config.mode, OutputMode::Human));
        Self { config }
    }

    pub fn config(&self) -> FrontendOutputConfig {
        self.config
    }

    pub fn is_machine_readable(&self) -> bool {
        matches!(self.config.mode, OutputMode::Json)
    }

    pub fn should_use_color(&self) -> bool {
        matches!(self.config.mode, OutputMode::Human)
    }

    fn styled_section(&self, title: &str) -> String {
        if self.should_use_color() {
            title.bold().cyan().to_string()
        } else {
            title.to_string()
        }
    }

    fn styled_label(&self, label: &str, width: usize) -> String {
        let padded = format!("{label:<width$}");
        if self.should_use_color() {
            padded.bold().yellow().to_string()
        } else {
            padded
        }
    }

    fn styled_action(&self, action: &str) -> String {
        if self.should_use_color() {
            action.bold().green().to_string()
        } else {
            action.to_string()
        }
    }

    fn styled_path(&self, path: &str) -> String {
        if self.should_use_color() {
            path.cyan().to_string()
        } else {
            path.to_string()
        }
    }

    fn styled_error_prefix(&self) -> String {
        if self.should_use_color() {
            "Error:".red().bold().to_string()
        } else {
            "Error:".to_string()
        }
    }

    fn styled_note_prefix(&self) -> String {
        if self.should_use_color() {
            "Note:".blue().bold().to_string()
        } else {
            "Note:".to_string()
        }
    }

    pub fn render_human_header(&self, title: &str) -> String {
        self.styled_section(title)
    }

    pub fn render_human_status(&self, action: &str, detail: &str) -> String {
        format!(
            "{} {}",
            self.styled_label(action, 12),
            self.styled_path(detail)
        )
    }

    pub fn render_plain_section(&self, title: &str) -> String {
        format!("{title}:")
    }

    pub fn render_plain_field(&self, label: &str, value: impl std::fmt::Display) -> String {
        format!("{label}: {value}")
    }

    pub fn render_plain_status(&self, label: &str, fields: &[(&str, String)]) -> String {
        let rendered = fields
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!("{label}: {rendered}")
    }

    pub fn render_json_result(
        &self,
        result: &FrontendCommandResult,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&serde_json::json!({
            "command": result.command,
            "summary": result.summary,
            "artifacts": result
                .artifacts
                .iter()
                .map(|artifact| serde_json::json!({
                    "kind": artifact.kind.as_str(),
                    "label": artifact.label,
                    "path": artifact.path.as_ref().map(|path| path.to_string_lossy().to_string()),
                }))
                .collect::<Vec<_>>(),
        }))
    }

    pub fn render_json_error(&self, error: &FrontendError) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&serde_json::json!({
            "kind": error.kind().as_str(),
            "message": error.message(),
            "notes": error.notes(),
        }))
    }

    pub fn render_human_error(&self, error: &FrontendError) -> String {
        let mut lines = vec![format!("{} {}", self.styled_error_prefix(), error)];
        for note in error.notes() {
            lines.push(format!("{} {}", self.styled_note_prefix(), note));
        }
        lines.join("\n")
    }

    pub fn render_plain_error(&self, error: &FrontendError) -> String {
        let mut lines = vec![format!("error: {}", error)];
        for note in error.notes() {
            lines.push(format!("note: {note}"));
        }
        lines.join("\n")
    }

    pub fn render_command_summary(
        &self,
        result: &FrontendCommandResult,
    ) -> Result<String, serde_json::Error> {
        match self.config.mode {
            OutputMode::Human => {
                let mut lines = vec![self.render_human_header(&result.command)];
                lines.push(format!(
                    "{} {}",
                    self.styled_action("Done:"),
                    result.summary
                ));
                for artifact in &result.artifacts {
                    let detail = artifact
                        .path
                        .as_ref()
                        .map(|path| path.to_string_lossy().to_string())
                        .unwrap_or_else(|| artifact.label.clone());
                    lines.push(self.render_human_status(artifact.kind.as_str(), &detail));
                }
                Ok(lines.join("\n"))
            }
            OutputMode::Plain => {
                let mut lines = vec![self.render_plain_field("command", &result.command)];
                lines.push(self.render_plain_field("summary", &result.summary));
                for artifact in &result.artifacts {
                    let detail = artifact
                        .path
                        .as_ref()
                        .map(|path| path.to_string_lossy().to_string())
                        .unwrap_or_else(|| artifact.label.clone());
                    lines.push(self.render_plain_field(artifact.kind.as_str(), detail));
                }
                Ok(lines.join("\n"))
            }
            OutputMode::Json => self.render_json_result(result),
        }
    }

    pub fn render_error(&self, error: &FrontendError) -> Result<String, serde_json::Error> {
        match self.config.mode {
            OutputMode::Human => Ok(self.render_human_error(error)),
            OutputMode::Plain => Ok(self.render_plain_error(error)),
            OutputMode::Json => self.render_json_error(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FrontendOutput;
    use crate::{
        FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendError,
        FrontendErrorKind, FrontendOutputConfig, OutputMode,
    };
    use std::path::PathBuf;

    #[test]
    fn output_helper_keeps_frontend_output_config() {
        let output = FrontendOutput::new(FrontendOutputConfig::default());

        assert_eq!(output.config().mode, OutputMode::Human);
        assert!(!output.is_machine_readable());
        assert!(output.should_use_color());
    }

    #[test]
    fn human_helpers_render_colored_sections_and_rows() {
        let output = FrontendOutput::new(FrontendOutputConfig::default());

        let header = output.render_human_header("Build");
        let status = output.render_human_status("binary", "target/bin/demo");

        assert!(header.contains("\u{1b}["));
        assert!(status.contains("\u{1b}["));
        assert!(status.contains("target/bin/demo"));
    }

    #[test]
    fn plain_helpers_render_stable_script_friendly_lines() {
        let output = FrontendOutput::new(FrontendOutputConfig {
            mode: OutputMode::Plain,
        });

        assert_eq!(output.render_plain_section("build"), "build:");
        assert_eq!(
            output.render_plain_field("artifact", "target/bin/demo"),
            "artifact: target/bin/demo"
        );
        assert_eq!(
            output.render_plain_status(
                "status",
                &[("kind", "binary".to_string()), ("path", "target/bin/demo".to_string())]
            ),
            "status: kind=binary path=target/bin/demo"
        );
    }

    #[test]
    fn json_helpers_render_structured_result_and_error_payloads() {
        let output = FrontendOutput::new(FrontendOutputConfig {
            mode: OutputMode::Json,
        });
        let result = FrontendCommandResult::new("build", "built binary").with_artifact(
            FrontendArtifactSummary::new(
                FrontendArtifactKind::Binary,
                "demo",
                Some(PathBuf::from("target/bin/demo")),
            ),
        );
        let error = FrontendError::new(FrontendErrorKind::CommandFailed, "boom")
            .with_note("note one");

        let rendered_result = output.render_json_result(&result).unwrap();
        let rendered_error = output.render_json_error(&error).unwrap();

        assert!(rendered_result.contains("\"command\": \"build\""));
        assert!(rendered_result.contains("\"kind\": \"binary\""));
        assert!(rendered_error.contains("\"kind\": \"command_failed\""));
        assert!(rendered_error.contains("\"note one\""));
    }

    #[test]
    fn color_rendering_only_applies_in_human_mode() {
        let human = FrontendOutput::new(FrontendOutputConfig::default());
        let json = FrontendOutput::new(FrontendOutputConfig {
            mode: OutputMode::Json,
        });

        assert!(human.should_use_color());
        assert!(!json.should_use_color());
    }

    #[test]
    fn human_error_rendering_uses_labeled_lines() {
        let output = FrontendOutput::new(FrontendOutputConfig::default());
        let error = FrontendError::new(FrontendErrorKind::WorkspaceNotFound, "missing root")
            .with_note("run `fol work init --bin`");

        let rendered = output.render_human_error(&error);

        assert!(rendered.contains("Error:"));
        assert!(rendered.contains("Note:"));
        assert!(rendered.contains("missing root"));
    }

    #[test]
    fn human_command_summary_renders_done_and_artifacts() {
        let output = FrontendOutput::new(FrontendOutputConfig::default());
        let result = FrontendCommandResult::new("build", "built demo").with_artifact(
            FrontendArtifactSummary::new(
                FrontendArtifactKind::Binary,
                "demo",
                Some(PathBuf::from("target/bin/demo")),
            ),
        );

        let rendered = output.render_command_summary(&result).unwrap();

        assert!(rendered.contains("Done:"));
        assert!(rendered.contains("target/bin/demo"));
        assert!(rendered.contains("build"));
    }
}
