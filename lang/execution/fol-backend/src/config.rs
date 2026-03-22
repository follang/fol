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

    pub fn rust_target_triple(&self) -> Option<String> {
        match self {
            Self::Host => None,
            Self::Triple(triple) => map_machine_target_to_rust_target(triple),
        }
    }
}

fn map_machine_target_to_rust_target(raw: &str) -> Option<String> {
    if is_known_rust_target_triple(raw) {
        return Some(raw.to_string());
    }

    match raw {
        "x86_64-linux-gnu" => Some("x86_64-unknown-linux-gnu".to_string()),
        "x86_64-linux-musl" => Some("x86_64-unknown-linux-musl".to_string()),
        "aarch64-linux-gnu" => Some("aarch64-unknown-linux-gnu".to_string()),
        "aarch64-linux-musl" => Some("aarch64-unknown-linux-musl".to_string()),
        "x86_64-windows-gnu" => Some("x86_64-pc-windows-gnu".to_string()),
        "x86_64-windows-msvc" => Some("x86_64-pc-windows-msvc".to_string()),
        "aarch64-windows-msvc" => Some("aarch64-pc-windows-msvc".to_string()),
        "x86_64-macos-gnu" => Some("x86_64-apple-darwin".to_string()),
        "aarch64-macos-gnu" => Some("aarch64-apple-darwin".to_string()),
        _ => None,
    }
}

fn is_known_rust_target_triple(raw: &str) -> bool {
    matches!(
        raw,
        "x86_64-unknown-linux-gnu"
            | "x86_64-unknown-linux-musl"
            | "aarch64-unknown-linux-gnu"
            | "aarch64-unknown-linux-musl"
            | "x86_64-pc-windows-gnu"
            | "x86_64-pc-windows-msvc"
            | "aarch64-pc-windows-msvc"
            | "x86_64-apple-darwin"
            | "aarch64-apple-darwin"
    )
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

    #[test]
    fn machine_target_maps_fol_target_spellings_to_rust_triples() {
        assert_eq!(
            BackendMachineTarget::Triple("x86_64-linux-gnu".to_string()).rust_target_triple(),
            Some("x86_64-unknown-linux-gnu".to_string())
        );
        assert_eq!(
            BackendMachineTarget::Triple("aarch64-linux-musl".to_string()).rust_target_triple(),
            Some("aarch64-unknown-linux-musl".to_string())
        );
        assert_eq!(
            BackendMachineTarget::Triple("aarch64-macos-gnu".to_string()).rust_target_triple(),
            Some("aarch64-apple-darwin".to_string())
        );
    }

    #[test]
    fn machine_target_keeps_known_rust_triples_stable() {
        assert_eq!(
            BackendMachineTarget::Triple("x86_64-unknown-linux-gnu".to_string())
                .rust_target_triple(),
            Some("x86_64-unknown-linux-gnu".to_string())
        );
        assert_eq!(
            BackendMachineTarget::Triple("aarch64-apple-darwin".to_string()).rust_target_triple(),
            Some("aarch64-apple-darwin".to_string())
        );
    }

    #[test]
    fn machine_target_rejects_unknown_target_spellings() {
        assert_eq!(
            BackendMachineTarget::Triple("sparc-linux-gnu".to_string()).rust_target_triple(),
            None
        );
        assert_eq!(
            BackendMachineTarget::Triple("aarch64-macos-msvc".to_string()).rust_target_triple(),
            None
        );
        assert_eq!(BackendMachineTarget::Host.rust_target_triple(), None);
    }
}
