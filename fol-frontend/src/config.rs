use crate::{ColorPolicy, FrontendOutputConfig, FrontendProfile, OutputMode};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendConfig {
    pub working_directory: PathBuf,
    pub output: FrontendOutputConfig,
    pub profile_override: Option<FrontendProfile>,
    pub std_root_override: Option<PathBuf>,
    pub package_store_root_override: Option<PathBuf>,
    pub build_root_override: Option<PathBuf>,
    pub cache_root_override: Option<PathBuf>,
    pub keep_build_dir: bool,
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            output: FrontendOutputConfig::default(),
            profile_override: None,
            std_root_override: None,
            package_store_root_override: None,
            build_root_override: None,
            cache_root_override: None,
            keep_build_dir: false,
        }
    }
}

impl FrontendConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        config.output.mode = match std::env::var("FOL_OUTPUT").ok().as_deref() {
            Some("plain") => OutputMode::Plain,
            Some("json") => OutputMode::Json,
            _ => OutputMode::Human,
        };
        config.output.color = match std::env::var("FOL_COLOR").ok().as_deref() {
            Some("always") => ColorPolicy::Always,
            Some("never") => ColorPolicy::Never,
            _ => ColorPolicy::Auto,
        };
        config.profile_override = match std::env::var("FOL_PROFILE").ok().as_deref() {
            Some("release") => Some(FrontendProfile::Release),
            Some("debug") => Some(FrontendProfile::Debug),
            _ => None,
        };
        config.std_root_override = std::env::var_os("FOL_STD_ROOT").map(PathBuf::from);
        config.package_store_root_override =
            std::env::var_os("FOL_PACKAGE_STORE_ROOT").map(PathBuf::from);
        config.build_root_override = std::env::var_os("FOL_BUILD_ROOT").map(PathBuf::from);
        config.cache_root_override = std::env::var_os("FOL_CACHE_ROOT").map(PathBuf::from);
        config.keep_build_dir = std::env::var_os("FOL_KEEP_BUILD_DIR")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        config
    }
}

#[cfg(test)]
mod tests {
    use super::FrontendConfig;
    use crate::{ColorPolicy, FrontendProfile, OutputMode};

    #[test]
    fn frontend_config_defaults_to_current_working_defaults() {
        let config = FrontendConfig::default();

        assert_eq!(config.output.mode, OutputMode::Human);
        assert_eq!(config.output.color, ColorPolicy::Auto);
        assert!(config.profile_override.is_none());
        assert!(config.std_root_override.is_none());
        assert!(config.package_store_root_override.is_none());
        assert!(config.build_root_override.is_none());
        assert!(config.cache_root_override.is_none());
        assert!(!config.keep_build_dir);
    }

    #[test]
    fn frontend_config_reads_root_overrides_from_environment() {
        unsafe {
            std::env::set_var("FOL_STD_ROOT", "/tmp/std");
            std::env::set_var("FOL_PACKAGE_STORE_ROOT", "/tmp/pkg");
            std::env::set_var("FOL_BUILD_ROOT", "/tmp/build");
            std::env::set_var("FOL_CACHE_ROOT", "/tmp/cache");
            std::env::set_var("FOL_KEEP_BUILD_DIR", "true");
            std::env::set_var("FOL_OUTPUT", "json");
            std::env::set_var("FOL_COLOR", "never");
            std::env::set_var("FOL_PROFILE", "release");
        }

        let config = FrontendConfig::from_env();

        assert_eq!(config.output.mode, OutputMode::Json);
        assert_eq!(config.output.color, ColorPolicy::Never);
        assert_eq!(config.profile_override, Some(FrontendProfile::Release));
        assert_eq!(config.std_root_override, Some(std::path::PathBuf::from("/tmp/std")));
        assert_eq!(
            config.package_store_root_override,
            Some(std::path::PathBuf::from("/tmp/pkg"))
        );
        assert_eq!(config.build_root_override, Some(std::path::PathBuf::from("/tmp/build")));
        assert_eq!(config.cache_root_override, Some(std::path::PathBuf::from("/tmp/cache")));
        assert!(config.keep_build_dir);

        unsafe {
            std::env::remove_var("FOL_STD_ROOT");
            std::env::remove_var("FOL_PACKAGE_STORE_ROOT");
            std::env::remove_var("FOL_BUILD_ROOT");
            std::env::remove_var("FOL_CACHE_ROOT");
            std::env::remove_var("FOL_KEEP_BUILD_DIR");
            std::env::remove_var("FOL_OUTPUT");
            std::env::remove_var("FOL_COLOR");
            std::env::remove_var("FOL_PROFILE");
        }
    }
}
