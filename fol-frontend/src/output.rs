#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputMode {
    Human,
    Plain,
    Json,
}

impl OutputMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Plain => "plain",
            Self::Json => "json",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorPolicy {
    Auto,
    Always,
    Never,
}

impl ColorPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Always => "always",
            Self::Never => "never",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontendOutputConfig {
    pub mode: OutputMode,
    pub color: ColorPolicy,
}

impl Default for FrontendOutputConfig {
    fn default() -> Self {
        Self {
            mode: OutputMode::Human,
            color: ColorPolicy::Auto,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ColorPolicy, FrontendOutputConfig, OutputMode};

    #[test]
    fn frontend_output_defaults_to_human_auto() {
        let config = FrontendOutputConfig::default();

        assert_eq!(config.mode, OutputMode::Human);
        assert_eq!(config.color, ColorPolicy::Auto);
        assert_eq!(config.mode.as_str(), "human");
        assert_eq!(config.color.as_str(), "auto");
    }
}
