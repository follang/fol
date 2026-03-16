use crate::{FrontendCommandResult, FrontendError, FrontendOutputConfig, OutputMode};

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

    pub fn render_human_header(&self, title: &str) -> String {
        format!("== {title} ==")
    }

    pub fn render_human_status(&self, action: &str, detail: &str) -> String {
        format!("{action}: {detail}")
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
}
