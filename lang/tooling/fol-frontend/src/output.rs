#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontendOutputConfig {
    pub mode: OutputMode,
}

impl Default for FrontendOutputConfig {
    fn default() -> Self {
        Self {
            mode: OutputMode::Human,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FrontendOutputConfig, OutputMode};

    #[test]
    fn frontend_output_defaults_to_human() {
        let config = FrontendOutputConfig::default();

        assert_eq!(config.mode, OutputMode::Human);
        assert_eq!(config.mode.as_str(), "human");
    }
}
