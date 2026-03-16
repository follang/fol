//! Backend foundations for turning lowered `V1` FOL workspaces into runnable artifacts.

mod config;
mod control;
mod emit;
mod error;
mod identity;
mod instructions;
mod layout;
mod mangle;
mod model;
mod session;
mod signatures;
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
pub use control::render_terminator;
pub use emit::{
    build_generated_crate, emit_backend_artifact, emit_cargo_toml,
    emit_generated_crate_skeleton, emit_main_rs, emit_namespace_module_shells,
    emit_package_module_shells, prepare_generated_build_dir, summarize_emitted_artifact,
    write_generated_crate,
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
pub use signatures::{
    render_global_declaration, render_routine_definition, render_routine_shell,
    render_routine_signature,
};
pub use trace::{
    build_backend_trace, build_emitted_source_map, BackendEmittedSourceMap,
    BackendEmittedSourceMapEntry, BackendTrace, BackendTraceKind, BackendTraceRecord,
};
pub use instructions::{render_core_instruction, render_core_instruction_in_workspace};
pub use types::{
    render_entry_definition, render_entry_trait_impl, render_record_definition,
    render_record_trait_impl, render_rust_type, render_rust_type_in_workspace,
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
