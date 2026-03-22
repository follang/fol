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
    BuildArtifactWithCargo,
    BuildArtifactWithRustc,
}

impl BackendMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmitSource => "emit-source",
            Self::BuildArtifactWithCargo => "build-artifact-cargo",
            Self::BuildArtifactWithRustc => "build-artifact-rustc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendConfig {
    pub target: BackendTarget,
    pub build_profile: BackendBuildProfile,
    pub mode: BackendMode,
    pub keep_build_dir: bool,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            target: BackendTarget::Rust,
            build_profile: BackendBuildProfile::Release,
            mode: BackendMode::BuildArtifactWithRustc,
            keep_build_dir: false,
        }
    }
}
