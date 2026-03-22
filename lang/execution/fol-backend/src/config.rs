#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendTarget {
    Rust,
}

impl BackendTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rust => "rust",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BackendMachineTarget {
    #[default]
    Host,
    Triple(String),
}

impl BackendMachineTarget {
    pub fn normalize_input(value: &str) -> Self {
        let trimmed = value.trim();
        if matches!(trimmed, "" | "host" | "native") {
            Self::Host
        } else {
            Self::Triple(trimmed.to_string())
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Host => "host",
            Self::Triple(triple) => triple.as_str(),
        }
    }

    pub fn is_host(&self) -> bool {
        matches!(self, Self::Host)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendBuildProfile {
    Debug,
    Release,
}

impl BackendBuildProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendMode {
    EmitSource,
    BuildArtifact,
}

impl BackendMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmitSource => "emit-source",
            Self::BuildArtifact => "build-artifact",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendConfig {
    pub target: BackendTarget,
    pub machine_target: BackendMachineTarget,
    pub build_profile: BackendBuildProfile,
    pub mode: BackendMode,
    pub keep_build_dir: bool,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            target: BackendTarget::Rust,
            machine_target: BackendMachineTarget::Host,
            build_profile: BackendBuildProfile::Release,
            mode: BackendMode::BuildArtifact,
            keep_build_dir: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BackendMachineTarget;

    #[test]
    fn machine_target_normalization_keeps_host_aliases_canonical() {
        assert_eq!(
            BackendMachineTarget::normalize_input("host"),
            BackendMachineTarget::Host
        );
        assert_eq!(
            BackendMachineTarget::normalize_input("native"),
            BackendMachineTarget::Host
        );
        assert_eq!(
            BackendMachineTarget::normalize_input("  host  "),
            BackendMachineTarget::Host
        );
    }

    #[test]
    fn machine_target_normalization_preserves_explicit_triples() {
        assert_eq!(
            BackendMachineTarget::normalize_input("aarch64-unknown-linux-gnu"),
            BackendMachineTarget::Triple("aarch64-unknown-linux-gnu".to_string())
        );
        assert_eq!(
            BackendMachineTarget::normalize_input("  x86_64-pc-windows-gnu  "),
            BackendMachineTarget::Triple("x86_64-pc-windows-gnu".to_string())
        );
    }

    #[test]
    fn machine_target_reports_host_state_and_display_name() {
        let host = BackendMachineTarget::Host;
        let triple = BackendMachineTarget::Triple("aarch64-apple-darwin".to_string());

        assert!(host.is_host());
        assert_eq!(host.display_name(), "host");
        assert!(!triple.is_host());
        assert_eq!(triple.display_name(), "aarch64-apple-darwin");
    }
}
