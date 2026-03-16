use crate::{FrontendOutputConfig, OutputMode};

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
}

#[cfg(test)]
mod tests {
    use super::FrontendOutput;
    use crate::{FrontendOutputConfig, OutputMode};

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
}
