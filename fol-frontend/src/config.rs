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

impl FrontendConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        config.std_root_override = std::env::var_os("FOL_STD_ROOT").map(PathBuf::from);
        config.package_store_root_override =
            std::env::var_os("FOL_PACKAGE_STORE_ROOT").map(PathBuf::from);
        config.build_root_override = std::env::var_os("FOL_BUILD_ROOT").map(PathBuf::from);
        config.cache_root_override = std::env::var_os("FOL_CACHE_ROOT").map(PathBuf::from);
        config
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

    #[test]
    fn frontend_config_reads_root_overrides_from_environment() {
        unsafe {
            std::env::set_var("FOL_STD_ROOT", "/tmp/std");
            std::env::set_var("FOL_PACKAGE_STORE_ROOT", "/tmp/pkg");
            std::env::set_var("FOL_BUILD_ROOT", "/tmp/build");
            std::env::set_var("FOL_CACHE_ROOT", "/tmp/cache");
        }

        let config = FrontendConfig::from_env();

        assert_eq!(config.std_root_override, Some(std::path::PathBuf::from("/tmp/std")));
        assert_eq!(
            config.package_store_root_override,
            Some(std::path::PathBuf::from("/tmp/pkg"))
        );
        assert_eq!(config.build_root_override, Some(std::path::PathBuf::from("/tmp/build")));
        assert_eq!(config.cache_root_override, Some(std::path::PathBuf::from("/tmp/cache")));

        unsafe {
            std::env::remove_var("FOL_STD_ROOT");
            std::env::remove_var("FOL_PACKAGE_STORE_ROOT");
            std::env::remove_var("FOL_BUILD_ROOT");
            std::env::remove_var("FOL_CACHE_ROOT");
        }
    }
}
