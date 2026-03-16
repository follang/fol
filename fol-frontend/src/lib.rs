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
    FrontendCli, FrontendCommand, FrontendProfile, InitCommand, NewCommand, RunCommand,
    UnitCommand,
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
    fetch_workspace, fetch_workspace_with_config, prepare_workspace_packages,
    select_package_store_root, FrontendPackagePreparation, FrontendPreparedPackage,
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
    enumerate_member_packages, load_frontend_workspace, load_workspace_config, FrontendWorkspace,
    FrontendWorkspaceConfig,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Frontend;

impl Frontend {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> FrontendResult<()> {
        let args = std::env::args_os().collect::<Vec<_>>();
        let (output, result) = run_command_from_args(args)?;
        println!(
            "{}",
            output
                .render_command_summary(&result)
                .map_err(|error| FrontendError::new(FrontendErrorKind::Internal, error.to_string()))?
        );
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

pub fn run_command_from_args<I, T>(
    args: I,
) -> FrontendResult<(FrontendOutput, FrontendCommandResult)>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = FrontendCli::try_parse_from(args).map_err(|error| {
        FrontendError::new(FrontendErrorKind::InvalidInput, error.to_string())
            .with_note("run `fol --help` to inspect the available workflow commands")
    })?;
    let config = frontend_config_from_cli(&cli, None);
    let output = FrontendOutput::new(config.output);
    let result = dispatch_cli(&cli, &config)?;
    Ok((output, result))
}

pub fn run_command_from_args_in_dir<I, T>(
    args: I,
    working_directory: impl Into<std::path::PathBuf>,
) -> FrontendResult<(FrontendOutput, FrontendCommandResult)>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = FrontendCli::try_parse_from(args).map_err(|error| {
        FrontendError::new(FrontendErrorKind::InvalidInput, error.to_string())
            .with_note("run `fol --help` to inspect the available workflow commands")
    })?;
    let config = frontend_config_from_cli(&cli, Some(working_directory.into()));
    let output = FrontendOutput::new(config.output);
    let result = dispatch_cli(&cli, &config)?;
    Ok((output, result))
}

fn frontend_config_from_cli(
    cli: &FrontendCli,
    working_directory: Option<std::path::PathBuf>,
) -> FrontendConfig {
    let mut config = FrontendConfig::from_env();
    if let Some(working_directory) = working_directory {
        config.working_directory = working_directory;
    }
    config.output.mode = cli.output;
    config.output.color = cli.color;
    config.profile_override = Some(cli.selected_profile());
    config
}

fn dispatch_cli(cli: &FrontendCli, config: &FrontendConfig) -> FrontendResult<FrontendCommandResult> {
    match cli.command.as_ref() {
        None => Err(FrontendError::new(
            FrontendErrorKind::InvalidInput,
            "no frontend command was provided",
        )
        .with_note("run `fol --help` to inspect the frontend workflow")),
        Some(FrontendCommand::Init(command)) => init_root(
            &config.working_directory,
            command.workspace,
            package_target_kind(command.bin, command.lib),
        ),
        Some(FrontendCommand::New(command)) => new_project_with_mode(
            &config.working_directory,
            &command.name,
            command.workspace,
            package_target_kind(command.bin, command.lib),
        ),
        Some(FrontendCommand::Completion(command)) => {
            completion_command(parse_completion_shell(command.shell))
        }
        Some(FrontendCommand::Complete(command)) => {
            internal_complete_command_with_tokens(&command.tokens)
        }
        Some(command) => {
            let discovered = require_discovered_root(&config.working_directory)?;
            let workspace = load_frontend_workspace(&discovered, config)?;
            dispatch_workspace_command(command, &workspace, config)
        }
    }
}

fn dispatch_workspace_command(
    command: &FrontendCommand,
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    match command {
        FrontendCommand::Work(command) => Ok(match command.command {
            cli::WorkSubcommand::Info(_) => work_info(workspace),
            cli::WorkSubcommand::List(_) => work_list(workspace),
        }),
        FrontendCommand::Fetch(_) => fetch_workspace_with_config(workspace, config),
        FrontendCommand::Build(_) => build_workspace_for_profile_with_config(
            workspace,
            config,
            config.profile_override.unwrap_or(FrontendProfile::Debug),
        ),
        FrontendCommand::Check(_) => check_workspace_with_config(workspace, config),
        FrontendCommand::Run(command) => {
            run_workspace_with_args_and_config(workspace, config, &command.args)
        }
        FrontendCommand::Test(_) => test_workspace_with_config(workspace, config),
        FrontendCommand::Emit(command) => match command.command {
            EmitSubcommand::Rust(_) => emit_rust_with_config(workspace, config),
            EmitSubcommand::Lowered(_) => emit_lowered_with_config(workspace, config),
        },
        FrontendCommand::Clean(_) => clean_workspace_with_config(workspace, config),
        FrontendCommand::Completion(_)
        | FrontendCommand::Complete(_)
        | FrontendCommand::Init(_)
        | FrontendCommand::New(_) => Err(FrontendError::new(
            FrontendErrorKind::Internal,
            "unexpected command reached workspace dispatcher",
        )),
    }
}

fn parse_completion_shell(shell: CompletionShellArg) -> CompletionShell {
    match shell {
        CompletionShellArg::Bash => CompletionShell::Bash,
        CompletionShellArg::Zsh => CompletionShell::Zsh,
        CompletionShellArg::Fish => CompletionShell::Fish,
    }
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
        let _ = frontend;
        let _run_ptr: fn() -> FrontendResult<()> = run;
    }

    #[test]
    fn run_command_from_args_dispatches_buildable_frontend_commands() {
        let root = std::env::temp_dir().join(format!("fol_frontend_dispatch_{}", std::process::id()));
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        std::fs::write(root.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        std::fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let (_, result) = run_command_from_args_in_dir(["fol", "check"], &root).unwrap();

        assert_eq!(result.command, "check");
        assert!(result.summary.contains("checked 1 workspace package(s)"));

        std::fs::remove_dir_all(root).ok();
    }
}
