//! User-facing frontend foundations for the FOL toolchain.
//!
//! `fol-frontend` will become the canonical command-line/workspace entrypoint
//! above `fol-package` and the compiler pipeline.

mod config;
mod cli;
mod discovery;
mod errors;
mod output;
mod result;
mod ui;

pub use cli::{FrontendCli, FrontendCommand, UnitCommand};
pub use config::FrontendConfig;
pub use errors::{FrontendError, FrontendErrorKind, FrontendResult};
pub use discovery::{
    discover_root_from_explicit_path, discover_root_upward, DiscoveredRoot, PackageRoot,
    WorkspaceRoot, PACKAGE_FILE_NAME, WORKSPACE_FILE_NAME,
};
pub use output::{ColorPolicy, FrontendOutputConfig, OutputMode};
pub use result::{FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult};
pub use ui::FrontendOutput;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Frontend;

impl Frontend {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> FrontendResult<()> {
        Ok(())
    }
}

pub const CRATE_NAME: &str = "fol-frontend";

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

pub fn run() -> FrontendResult<()> {
    Frontend::new().run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_matches_frontend_identity() {
        assert_eq!(crate_name(), "fol-frontend");
    }

    #[test]
    fn public_run_shell_is_callable() {
        let frontend = Frontend::new();
        assert_eq!(frontend.run(), Ok(()));
        assert_eq!(run(), Ok(()));
    }
}
