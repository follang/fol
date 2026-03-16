use crate::FrontendOutputConfig;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendConfig {
    pub working_directory: PathBuf,
    pub output: FrontendOutputConfig,
    pub std_root_override: Option<PathBuf>,
    pub package_store_root_override: Option<PathBuf>,
    pub build_root_override: Option<PathBuf>,
    pub cache_root_override: Option<PathBuf>,
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            output: FrontendOutputConfig::default(),
            std_root_override: None,
            package_store_root_override: None,
            build_root_override: None,
            cache_root_override: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FrontendConfig;
    use crate::{ColorPolicy, OutputMode};

    #[test]
    fn frontend_config_defaults_to_current_working_defaults() {
        let config = FrontendConfig::default();

        assert_eq!(config.output.mode, OutputMode::Human);
        assert_eq!(config.output.color, ColorPolicy::Auto);
        assert!(config.std_root_override.is_none());
        assert!(config.package_store_root_override.is_none());
        assert!(config.build_root_override.is_none());
        assert!(config.cache_root_override.is_none());
    }
}
