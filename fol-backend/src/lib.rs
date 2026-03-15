//! Backend foundations for turning lowered `V1` FOL workspaces into runnable artifacts.

mod config;
mod emit;
mod error;
mod identity;
mod layout;
mod mangle;
mod model;
mod session;
mod trace;
mod types;

#[cfg(test)]
mod testing;

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
pub use emit::{
    emit_cargo_toml, emit_generated_crate_skeleton, emit_main_rs,
    emit_namespace_module_shells, emit_package_module_shells,
};
pub use error::{BackendError, BackendErrorKind};
pub use identity::{stable_workspace_hash, BackendWorkspaceIdentity};
pub use layout::{
    plan_generated_crate_layout, plan_namespace_layouts, plan_package_layouts,
    GeneratedCrateLayoutPlan, NamespaceLayoutPlan, PackageLayoutPlan,
};
pub use mangle::{
    mangle_global_name, mangle_local_name, mangle_package_module_name,
    mangle_routine_name, mangle_type_name, sanitize_backend_ident,
};
pub use model::{BackendArtifact, EmittedRustFile};
pub use session::BackendSession;
pub use trace::{
    BackendEmittedSourceMap, BackendEmittedSourceMapEntry, BackendTrace, BackendTraceKind,
    BackendTraceRecord,
};
pub use types::{
    render_entry_definition, render_record_definition, render_record_trait_impl, render_rust_type,
};

pub type BackendResult<T> = Result<T, BackendError>;

#[cfg(test)]
mod tests {
    use super::{
        Backend, BackendArtifact, BackendConfig, BackendError, BackendErrorKind, BackendMode,
        BackendResult, BackendSession, BackendTarget, EmittedRustFile,
    };
    use crate::testing::sample_lowered_workspace;

    #[test]
    fn backend_foundation_public_surface_is_constructible() {
        let backend = Backend::new();
        let config = BackendConfig::default();
        let session = BackendSession::new(sample_lowered_workspace());
        let result: BackendResult<()> = Ok(());
        let artifact = BackendArtifact::RustSourceCrate {
            root: "target/fol-backend/demo".to_string(),
            files: vec![EmittedRustFile {
                path: "src/main.rs".to_string(),
                module_name: "main".to_string(),
                contents: "fn main() {}".to_string(),
            }],
        };
        let error = BackendError::new(BackendErrorKind::Unsupported, "not implemented yet");

        assert_eq!(format!("{backend:?}"), "Backend");
        assert_eq!(config.target, BackendTarget::Rust);
        assert_eq!(config.mode, BackendMode::BuildArtifact);
        assert_eq!(session.workspace().package_count(), 2);
        assert!(result.is_ok());
        assert!(matches!(artifact, BackendArtifact::RustSourceCrate { .. }));
        assert_eq!(error.to_string(), "BackendUnsupported: not implemented yet");
    }
}
