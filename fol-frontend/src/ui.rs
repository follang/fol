use crate::{ColorPolicy, FrontendCommandResult, FrontendError, FrontendOutputConfig, OutputMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontendOutput {
    config: FrontendOutputConfig,
}

impl FrontendOutput {
    pub fn new(config: FrontendOutputConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> FrontendOutputConfig {
        self.config
    }

    pub fn is_machine_readable(&self) -> bool {
        matches!(self.config.mode, OutputMode::Json)
    }

    pub fn should_use_color(&self, is_tty: bool) -> bool {
        if !matches!(self.config.mode, OutputMode::Human) {
            return false;
        }

        match self.config.color {
            ColorPolicy::Always => true,
            ColorPolicy::Never => false,
            ColorPolicy::Auto => is_tty,
        }
    }

    fn human_highlight_action(&self, action: &str) -> String {
        if self.config.mode == OutputMode::Human && self.config.color == ColorPolicy::Always {
            format!("\x1b[1;32m{action}\x1b[0m")
        } else {
            action.to_string()
        }
    }

    fn human_highlight_path(&self, path: &str) -> String {
        if self.config.mode == OutputMode::Human && self.config.color == ColorPolicy::Always {
            format!("\x1b[36m{path}\x1b[0m")
        } else {
            path.to_string()
        }
    }

    pub fn render_human_header(&self, title: &str) -> String {
        format!("== {title} ==")
    }

    pub fn render_human_status(&self, action: &str, detail: &str) -> String {
        format!(
            "{}: {}",
            self.human_highlight_action(action),
            self.human_highlight_path(detail)
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
        }))
    }

    pub fn render_command_summary(
        &self,
        result: &FrontendCommandResult,
    ) -> Result<String, serde_json::Error> {
        match self.config.mode {
            OutputMode::Human => {
                let mut lines = vec![self.render_human_header(&result.command)];
                lines.push(self.render_human_status("Summary", &result.summary));
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
}

#[cfg(test)]
mod tests {
    use super::FrontendOutput;
    use crate::{
        ColorPolicy, FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult,
        FrontendError, FrontendErrorKind, FrontendOutputConfig, OutputMode,
    };
    use std::path::PathBuf;

    #[test]
    fn output_helper_keeps_frontend_output_config() {
        let output = FrontendOutput::new(FrontendOutputConfig::default());

        assert_eq!(output.config().mode, OutputMode::Human);
        assert!(!output.is_machine_readable());
        assert!(output.should_use_color(true));
        assert!(!output.should_use_color(false));
    }

    #[test]
    fn human_helpers_render_stable_plain_human_lines() {
        let output = FrontendOutput::new(FrontendOutputConfig::default());

        assert_eq!(output.render_human_header("Build"), "== Build ==");
        assert_eq!(
            output.render_human_status("Built", "target/bin/demo"),
            "Built: target/bin/demo"
        );
    }

    #[test]
    fn human_helpers_highlight_actions_and_paths_when_color_is_forced() {
        let output = FrontendOutput::new(FrontendOutputConfig {
            color: ColorPolicy::Always,
            ..FrontendOutputConfig::default()
        });

        let rendered = output.render_human_status("Built", "target/bin/demo");

        assert!(rendered.contains("\u{1b}[1;32mBuilt\u{1b}[0m"));
        assert!(rendered.contains("\u{1b}[36mtarget/bin/demo\u{1b}[0m"));
    }

    #[test]
    fn plain_helpers_render_stable_script_friendly_lines() {
        let output = FrontendOutput::new(FrontendOutputConfig::default());

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
        let output = FrontendOutput::new(FrontendOutputConfig::default());
        let result = FrontendCommandResult::new("build", "built binary").with_artifact(
            FrontendArtifactSummary::new(
                FrontendArtifactKind::Binary,
                "demo",
                Some(PathBuf::from("target/bin/demo")),
            ),
        );
        let error = FrontendError::new(FrontendErrorKind::CommandFailed, "failed");

        let rendered_result = output.render_json_result(&result).unwrap();
        let rendered_error = output.render_json_error(&error).unwrap();

        assert!(rendered_result.contains("\"command\": \"build\""));
        assert!(rendered_result.contains("\"kind\": \"binary\""));
        assert!(rendered_error.contains("\"kind\": \"FrontendCommandFailed\""));
        assert!(rendered_error.contains("\"message\": \"failed\""));
    }

    #[test]
    fn command_summary_respects_selected_output_mode() {
        let result = FrontendCommandResult::new("build", "built binary").with_artifact(
            FrontendArtifactSummary::new(
                FrontendArtifactKind::Binary,
                "demo",
                Some(PathBuf::from("target/bin/demo")),
            ),
        );

        let human = FrontendOutput::new(FrontendOutputConfig::default())
            .render_command_summary(&result)
            .unwrap();
        let plain = FrontendOutput::new(FrontendOutputConfig {
            mode: OutputMode::Plain,
            ..FrontendOutputConfig::default()
        })
        .render_command_summary(&result)
        .unwrap();
        let json = FrontendOutput::new(FrontendOutputConfig {
            mode: OutputMode::Json,
            ..FrontendOutputConfig::default()
        })
        .render_command_summary(&result)
        .unwrap();

        assert!(human.contains("== build =="));
        assert!(plain.contains("command: build"));
        assert!(json.contains("\"command\": \"build\""));
    }

    #[test]
    fn color_auto_detection_respects_mode_and_policy() {
        let always = FrontendOutput::new(FrontendOutputConfig {
            color: ColorPolicy::Always,
            ..FrontendOutputConfig::default()
        });
        let never = FrontendOutput::new(FrontendOutputConfig {
            color: ColorPolicy::Never,
            ..FrontendOutputConfig::default()
        });
        let json = FrontendOutput::new(FrontendOutputConfig {
            mode: OutputMode::Json,
            ..FrontendOutputConfig::default()
        });

        assert!(always.should_use_color(false));
        assert!(!never.should_use_color(true));
        assert!(!json.should_use_color(true));
    }

    #[test]
    fn output_compatibility_matrix_stays_stable_across_modes() {
        let result = FrontendCommandResult::new("check", "ok").with_artifact(
            FrontendArtifactSummary::new(FrontendArtifactKind::WorkspaceRoot, "root", None),
        );

        let human = FrontendOutput::new(FrontendOutputConfig::default())
            .render_command_summary(&result)
            .unwrap();
        let plain = FrontendOutput::new(FrontendOutputConfig {
            mode: OutputMode::Plain,
            ..FrontendOutputConfig::default()
        })
        .render_command_summary(&result)
        .unwrap();
        let json = FrontendOutput::new(FrontendOutputConfig {
            mode: OutputMode::Json,
            ..FrontendOutputConfig::default()
        })
        .render_command_summary(&result)
        .unwrap();

        assert_eq!(human, "== check ==\nSummary: ok\nworkspace-root: root");
        assert_eq!(plain, "command: check\nsummary: ok\nworkspace-root: root");
        assert_eq!(
            json,
            "{\n  \"artifacts\": [\n    {\n      \"kind\": \"workspace-root\",\n      \"label\": \"root\",\n      \"path\": null\n    }\n  ],\n  \"command\": \"check\",\n  \"summary\": \"ok\"\n}"
        );
    }
}
