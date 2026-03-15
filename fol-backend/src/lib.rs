//! Backend foundations for turning lowered `V1` FOL workspaces into runnable artifacts.

mod config;
mod error;
mod model;

pub const CRATE_NAME: &str = "fol-backend";

#[derive(Debug, Default)]
pub struct Backend;

impl Backend {
    pub fn new() -> Self {
        Self
    }
}

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

pub use config::{BackendConfig, BackendMode, BackendTarget};
pub use error::{BackendError, BackendErrorKind};
pub use model::{BackendArtifact, EmittedRustFile};

pub type BackendResult<T> = Result<T, BackendError>;

#[cfg(test)]
mod tests {
    use super::{
        Backend, BackendArtifact, BackendConfig, BackendError, BackendErrorKind, BackendMode,
        BackendResult, BackendTarget, EmittedRustFile,
    };

    #[test]
    fn backend_foundation_public_surface_is_constructible() {
        let backend = Backend::new();
        let config = BackendConfig::default();
        let result: BackendResult<()> = Ok(());
        let artifact = BackendArtifact::RustSourceCrate {
            root: "target/fol-backend/demo".to_string(),
            files: vec![EmittedRustFile {
                path: "src/main.rs".to_string(),
                module_name: "main".to_string(),
            }],
        };
        let error = BackendError::new(BackendErrorKind::Unsupported, "not implemented yet");

        assert_eq!(format!("{backend:?}"), "Backend");
        assert_eq!(config.target, BackendTarget::Rust);
        assert_eq!(config.mode, BackendMode::BuildArtifact);
        assert!(result.is_ok());
        assert!(matches!(artifact, BackendArtifact::RustSourceCrate { .. }));
        assert_eq!(error.to_string(), "BackendUnsupported: not implemented yet");
    }
}
