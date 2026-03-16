//! User-facing frontend foundations for the FOL toolchain.
//!
//! `fol-frontend` will become the canonical command-line/workspace entrypoint
//! above `fol-package` and the compiler pipeline.

mod config;
mod cli;
mod clean;
mod compile;
mod completion;
mod discovery;
mod errors;
mod fetch;
mod output;
mod result;
mod scaffold;
mod ui;
mod work;
mod workspace;

pub use cli::{
    CompleteCommand, CompletionCommand, CompletionShellArg, EmitCommand, EmitSubcommand,
    FrontendCli, FrontendCommand, FrontendProfile, InitCommand, NewCommand, UnitCommand,
};
pub use clean::{clean_workspace, clean_workspace_with_config};
pub use config::FrontendConfig;
pub use compile::{
    build_workspace, build_workspace_for_profile_with_config, build_workspace_with_config,
    check_workspace, check_workspace_with_config, compile_member_workspace, emit_rust,
    emit_lowered, emit_lowered_with_config, emit_rust_with_config, profile_build_root,
    run_workspace, run_workspace_with_args_and_config, run_workspace_with_config, test_workspace,
    test_package, test_package_with_config, test_workspace_with_config,
};
pub use completion::{
    completion_command, generate_bash_completion_script, generate_completion_script,
    generate_fish_completion_script, generate_zsh_completion_script, internal_complete_command,
    internal_complete_command_with_tokens, internal_complete_matches, CompletionShell,
};
pub use errors::{FrontendError, FrontendErrorKind, FrontendResult};
pub use fetch::{
    fetch_workspace, prepare_workspace_packages, select_package_store_root, FrontendPackagePreparation,
    FrontendPreparedPackage,
};
pub use discovery::{
    discover_root_from_explicit_path, discover_root_upward, require_discovered_root,
    DiscoveredRoot, PackageRoot, WorkspaceRoot, PACKAGE_FILE_NAME, WORKSPACE_FILE_NAME,
};
pub use output::{ColorPolicy, FrontendOutputConfig, OutputMode};
pub use result::{FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult};
pub use scaffold::{
    init_current_dir, init_package_root, init_root, init_workspace_root, new_project,
    new_project_with_mode, package_target_kind, PackageTargetKind,
};
pub use ui::FrontendOutput;
pub use work::{work_info, work_list};
pub use workspace::{
    enumerate_member_packages, load_workspace_config, FrontendWorkspace, FrontendWorkspaceConfig,
};

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
