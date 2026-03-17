//! User-facing frontend foundations for the FOL toolchain.
//!
//! `fol-frontend` will become the canonical command-line/workspace entrypoint
//! above `fol-package` and the compiler pipeline.

mod config;
mod cli;
mod clean;
mod compile;
mod completion;
mod direct;
mod discovery;
mod editor;
mod errors;
mod fetch;
mod output;
mod result;
mod scaffold;
mod ui;
mod work;
mod workspace;

pub use cli::{
    BuildCommand, CheckCommand, CodeCommand, CodeSubcommand, CompleteCommand, CompletionCommand,
    CompletionShellArg, EditorPathCommand, EmitCommand, EmitLoweredCommand, EmitRustCommand,
    EmitSubcommand, FetchCommand, FrontendCli,
    FrontendCommand, FrontendProfile, InitCommand, NewCommand, PackCommand, PackSubcommand,
    RunCommand, TestCommand, ToolCommand, ToolSubcommand, UnitCommand, UpdateCommand,
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
pub use direct::{
    run_direct_compile, run_direct_compile_with_io, DirectCompileConfig, DirectCompileMode,
};
pub use editor::{
    editor_highlight_command, editor_lsp_command, editor_lsp_stdio, editor_parse_command, editor_symbols_command,
    editor_tree_generate_command,
};
pub use errors::{FrontendError, FrontendErrorKind, FrontendResult};
pub use fetch::{
    fetch_workspace, fetch_workspace_with_config, prepare_workspace_packages,
    select_package_store_root, update_workspace, update_workspace_with_config,
    FrontendPackagePreparation, FrontendPreparedPackage,
};
pub use discovery::{
    discover_root_from_explicit_path, discover_root_upward, require_discovered_root,
    DiscoveredRoot, PackageRoot, WorkspaceRoot, PACKAGE_FILE_NAME, WORKSPACE_FILE_NAME,
};
pub use output::{FrontendOutputConfig, OutputMode};
pub use result::{FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult};
pub use scaffold::{
    init_current_dir, init_package_root, init_root, init_workspace_root, new_project,
    new_project_with_mode, package_target_kind, PackageTargetKind,
};
pub use ui::FrontendOutput;
pub use work::{work_deps, work_info, work_list, work_status};
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

pub fn run_from_args<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    run_from_args_with_io(args, &mut std::io::stdout(), &mut std::io::stderr())
}

pub fn run_from_args_with_io<I, T>(
    args: I,
    stdout: &mut impl std::io::Write,
    stderr: &mut impl std::io::Write,
) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let args = args
        .into_iter()
        .map(|arg| arg.into())
        .collect::<Vec<std::ffi::OsString>>();

    match FrontendCli::try_parse_from(args.clone()) {
        Err(error) if error.kind() == clap::error::ErrorKind::DisplayHelp => {
            match writeln!(stdout, "{error}") {
                Ok(()) => 0,
                Err(render_error) => {
                    let _ = writeln!(stderr, "FrontendInternal: {render_error}");
                    1
                }
            }
        }
        Err(error) if error.kind() == clap::error::ErrorKind::DisplayVersion => {
            match writeln!(stdout, "{error}") {
                Ok(()) => 0,
                Err(render_error) => {
                    let _ = writeln!(stderr, "FrontendInternal: {render_error}");
                    1
                }
            }
        }
        Err(error) => {
            let output = FrontendOutput::new(FrontendOutputConfig::default());
            let error = FrontendError::new(FrontendErrorKind::InvalidInput, error.to_string())
                .with_note("run `fol --help` to inspect the available workflow commands");
            match output.render_error(&error) {
                Ok(rendered) => {
                    let _ = writeln!(stderr, "{rendered}");
                }
                Err(render_error) => {
                    let _ = writeln!(stderr, "FrontendInternal: {render_error}");
                }
            }
            1
        }
        Ok(cli) if cli.input.is_some() => {
            let config = frontend_config_from_cli(&cli, None);
            run_direct_compile_with_io(
                &DirectCompileConfig {
                    input: cli.input.clone().unwrap_or_default(),
                    std_root: cli.std_root.clone(),
                    package_store_root: cli.package_store_root.clone(),
                    mode: DirectCompileMode::Auto {
                        dump_lowered: cli.dump_lowered,
                        emit_rust: cli.emit_rust,
                        keep_build_dir: cli.keep_build_dir,
                    },
                },
                &config,
                stdout,
            )
        }
        Ok(cli) if cli.command.is_none() => {
            let mut command = FrontendCli::command();
            match writeln!(stdout, "{}", command.render_long_help()) {
                Ok(()) => 0,
                Err(error) => {
                    let _ = writeln!(stderr, "FrontendInternal: {error}");
                    1
                }
            }
        }
        Ok(cli) if matches!(cli.command.as_ref(), Some(FrontendCommand::Tool(command)) if matches!(command.command, ToolSubcommand::Lsp(_))) => {
            let config = frontend_config_from_cli(&cli, None);
            match editor_lsp_stdio(&config) {
                Ok(()) => 0,
                Err(error) => {
                    let output = FrontendOutput::new(FrontendOutputConfig::default());
                    match output.render_error(&error) {
                        Ok(rendered) => {
                            let _ = writeln!(stderr, "{rendered}");
                        }
                        Err(render_error) => {
                            let _ = writeln!(stderr, "FrontendInternal: {render_error}");
                        }
                    }
                    1
                }
            }
        }
        Ok(_) => match run_command_from_args(args) {
            Ok((output, result)) => match output.render_command_summary(&result) {
                Ok(rendered) => match writeln!(stdout, "{rendered}") {
                    Ok(()) => 0,
                    Err(error) => {
                        let _ = writeln!(stderr, "FrontendInternal: {error}");
                        1
                    }
                },
                Err(error) => {
                    let _ = writeln!(stderr, "FrontendInternal: {error}");
                    1
                }
            },
            Err(error) => {
                let output = FrontendOutput::new(FrontendOutputConfig::default());
                match output.render_error(&error) {
                    Ok(rendered) => {
                        let _ = writeln!(stderr, "{rendered}");
                    }
                    Err(render_error) => {
                        let _ = writeln!(stderr, "FrontendInternal: {render_error}");
                    }
                }
                1
            }
        },
    }
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
    config.output.mode = if cli.json {
        OutputMode::Json
    } else {
        command_output_mode(cli).unwrap_or(cli.output)
    };
    config.profile_override = Some(command_profile(cli).unwrap_or_else(|| cli.selected_profile()));
    if let Some(std_root) = &cli.std_root {
        config.std_root_override = Some(std_root.into());
    }
    if let Some(package_store_root) = &cli.package_store_root {
        config.package_store_root_override = Some(package_store_root.into());
    }
    if cli.keep_build_dir {
        config.keep_build_dir = true;
    }
    match cli.command.as_ref() {
        Some(FrontendCommand::Pack(command)) => match &command.command {
            PackSubcommand::Fetch(command) => {
                config.locked_fetch = command.locked;
                config.offline_fetch = command.offline;
                config.refresh_fetch = command.refresh;
            }
            PackSubcommand::Update(_) => {
                config.refresh_fetch = true;
            }
        },
        Some(FrontendCommand::Code(command)) => match &command.command {
            CodeSubcommand::Build(command) => {
                config.locked_fetch = command.locked;
                apply_build_option_args(&mut config, &command.options);
            }
            CodeSubcommand::Run(command) => {
                config.locked_fetch = command.locked;
                apply_build_option_args(&mut config, &command.options);
            }
            CodeSubcommand::Test(command) => {
                config.locked_fetch = command.locked;
                apply_build_option_args(&mut config, &command.options);
            }
            CodeSubcommand::Check(command) => {
                config.locked_fetch = command.locked;
                apply_build_option_args(&mut config, &command.options);
            }
            CodeSubcommand::Emit(_) => {}
        },
        _ => {}
    }
    config
}

fn apply_build_option_args(config: &mut FrontendConfig, options: &cli::BuildOptionArgs) {
    if let Some(target) = &options.build_target {
        config.build_target_override = Some(target.clone());
    }
    if let Some(optimize) = &options.build_optimize {
        config.build_optimize_override = Some(optimize.clone());
    }
    if !options.build_options.is_empty() {
        config.build_option_overrides = options.build_options.clone();
    }
}

fn command_output_mode(cli: &FrontendCli) -> Option<OutputMode> {
    match cli.command.as_ref() {
        Some(FrontendCommand::Work(command)) => Some(command.output.output),
        Some(FrontendCommand::Pack(command)) => Some(command.output.output),
        Some(FrontendCommand::Code(command)) => Some(command.output.output),
        Some(FrontendCommand::Tool(command)) => Some(command.output.output),
        Some(FrontendCommand::Complete(_)) | None => None,
    }
}

fn command_profile(cli: &FrontendCli) -> Option<FrontendProfile> {
    match cli.command.as_ref() {
        Some(FrontendCommand::Code(command)) => Some(command.profile.selected_profile()),
        _ => None,
    }
}

fn dispatch_cli(cli: &FrontendCli, config: &FrontendConfig) -> FrontendResult<FrontendCommandResult> {
    if let Some(input) = &cli.input {
        return run_direct_compile(
            &DirectCompileConfig {
                input: input.clone(),
                std_root: cli.std_root.clone(),
                package_store_root: cli.package_store_root.clone(),
                mode: DirectCompileMode::Auto {
                    dump_lowered: cli.dump_lowered,
                    emit_rust: cli.emit_rust,
                    keep_build_dir: cli.keep_build_dir,
                },
            },
            config,
        );
    }

    match cli.command.as_ref() {
        None => Err(FrontendError::new(
            FrontendErrorKind::InvalidInput,
            "no frontend command was provided",
        )
        .with_note("run `fol --help` to inspect the frontend workflow")),
        Some(FrontendCommand::Work(command)) => match &command.command {
            cli::WorkSubcommand::Init(command) => init_root(
                &config.working_directory,
                command.workspace,
                package_target_kind(command.bin, command.lib),
            ),
            cli::WorkSubcommand::New(command) => new_project_with_mode(
                &config.working_directory,
                &command.name,
                command.workspace,
                package_target_kind(command.bin, command.lib),
            ),
            _ => {
                let discovered = discovered_root_for_command(&cli.command.as_ref().unwrap(), &config.working_directory)?;
                let workspace = load_frontend_workspace(&discovered, config)?;
                dispatch_workspace_command(cli.command.as_ref().unwrap(), &workspace, config)
            }
        },
        Some(FrontendCommand::Pack(_)) | Some(FrontendCommand::Code(_)) => {
            let needs_direct = match cli.command.as_ref().unwrap() {
                FrontendCommand::Code(command) => code_has_direct_target(command),
                _ => false,
            };
            if needs_direct {
                dispatch_direct_grouped_command(cli.command.as_ref().unwrap(), config)
            } else {
                let discovered = discovered_root_for_command(&cli.command.as_ref().unwrap(), &config.working_directory)?;
                let workspace = load_frontend_workspace(&discovered, config)?;
                dispatch_workspace_command(cli.command.as_ref().unwrap(), &workspace, config)
            }
        }
        Some(FrontendCommand::Tool(command)) => match &command.command {
            ToolSubcommand::Lsp(_) => editor_lsp_command(config),
            ToolSubcommand::Parse(command) => editor_parse_command(&command.path, config),
            ToolSubcommand::Highlight(command) => {
                editor_highlight_command(&command.path, config)
            }
            ToolSubcommand::Symbols(command) => editor_symbols_command(&command.path, config),
            ToolSubcommand::Tree(command) => match &command.command {
                cli::TreeSubcommand::Generate(command) => {
                    editor_tree_generate_command(&command.path, config)
                }
            },
            ToolSubcommand::Completion(command) => {
                completion_command(parse_completion_shell(command.shell))
            }
            ToolSubcommand::Clean(_) => {
                let discovered = discovered_root_for_command(&cli.command.as_ref().unwrap(), &config.working_directory)?;
                let workspace = load_frontend_workspace(&discovered, config)?;
                dispatch_workspace_command(cli.command.as_ref().unwrap(), &workspace, config)
            }
        },
        Some(FrontendCommand::Complete(command)) => {
            internal_complete_command_with_tokens(&command.tokens)
        }
    }
}

fn dispatch_direct_grouped_command(
    command: &FrontendCommand,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    match command {
        FrontendCommand::Code(command) => match &command.command {
            CodeSubcommand::Build(command) => run_direct_compile(
                &DirectCompileConfig {
                    input: command.target.input.clone().unwrap_or_default(),
                    std_root: command.roots.std_root.clone(),
                    package_store_root: command.roots.package_store_root.clone(),
                    mode: DirectCompileMode::Build {
                        keep_build_dir: command.keep_build_dir,
                    },
                },
                &config_for_roots_keep_build(config, &command.roots, command.keep_build_dir),
            ),
            CodeSubcommand::Check(command) => run_direct_compile(
                &DirectCompileConfig {
                    input: command.target.input.clone().unwrap_or_default(),
                    std_root: command.roots.std_root.clone(),
                    package_store_root: command.roots.package_store_root.clone(),
                    mode: DirectCompileMode::Check,
                },
                &config_for_roots(config, &command.roots),
            ),
            CodeSubcommand::Run(command) => run_direct_compile(
                &DirectCompileConfig {
                    input: command.target.input.clone().unwrap_or_default(),
                    std_root: command.roots.std_root.clone(),
                    package_store_root: command.roots.package_store_root.clone(),
                    mode: DirectCompileMode::Run {
                        keep_build_dir: command.keep_build_dir,
                        args: command.args.clone(),
                    },
                },
                &config_for_roots_keep_build(config, &command.roots, command.keep_build_dir),
            ),
            CodeSubcommand::Emit(command) => match &command.command {
                EmitSubcommand::Rust(emit) => run_direct_compile(
                    &DirectCompileConfig {
                        input: emit.target.input.clone().unwrap_or_default(),
                        std_root: emit.roots.std_root.clone(),
                        package_store_root: emit.roots.package_store_root.clone(),
                        mode: DirectCompileMode::EmitRust {
                            keep_build_dir: emit.keep_build_dir,
                        },
                    },
                    &config_for_roots_keep_build(config, &emit.roots, emit.keep_build_dir),
                ),
                EmitSubcommand::Lowered(emit) => run_direct_compile(
                    &DirectCompileConfig {
                        input: emit.target.input.clone().unwrap_or_default(),
                        std_root: emit.roots.std_root.clone(),
                        package_store_root: emit.roots.package_store_root.clone(),
                        mode: DirectCompileMode::EmitLowered,
                    },
                    &config_for_roots(config, &emit.roots),
                ),
            },
            CodeSubcommand::Test(_) => Err(FrontendError::new(
                FrontendErrorKind::Internal,
                "unexpected direct test dispatch",
            )),
        },
        _ => Err(FrontendError::new(
            FrontendErrorKind::Internal,
            "unexpected grouped direct dispatch",
        )),
    }
}

fn dispatch_workspace_command(
    command: &FrontendCommand,
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    match command {
        FrontendCommand::Work(command) => Ok(match command.command {
            cli::WorkSubcommand::Init(_) | cli::WorkSubcommand::New(_) => {
                return Err(FrontendError::new(
                    FrontendErrorKind::Internal,
                    "unexpected work setup command reached workspace dispatcher",
                ))
            }
            cli::WorkSubcommand::Info(_) => work_info(workspace),
            cli::WorkSubcommand::List(_) => work_list(workspace),
            cli::WorkSubcommand::Deps(_) => work_deps(workspace)?,
            cli::WorkSubcommand::Status(_) => work_status(workspace, config)?,
        }),
        FrontendCommand::Pack(command) => match &command.command {
            PackSubcommand::Fetch(command) => {
                fetch_workspace_with_config(workspace, &config_for_roots(config, &command.roots))
            }
            PackSubcommand::Update(command) => {
                update_workspace_with_config(workspace, &config_for_roots(config, &command.roots))
            }
        },
        FrontendCommand::Code(command) => match &command.command {
            CodeSubcommand::Build(command) => {
                build_workspace_for_profile_with_config(
                    workspace,
                    &config_for_roots_keep_build(config, &command.roots, command.keep_build_dir),
                    config.profile_override.unwrap_or(FrontendProfile::Debug),
                )
            }
            CodeSubcommand::Check(command) => {
                check_workspace_with_config(workspace, &config_for_roots(config, &command.roots))
            }
            CodeSubcommand::Run(command) => {
                run_workspace_with_args_and_config(
                    workspace,
                    &config_for_roots_keep_build(config, &command.roots, command.keep_build_dir),
                    &command.args,
                )
            }
            CodeSubcommand::Test(_) => test_workspace_with_config(workspace, config),
            CodeSubcommand::Emit(command) => match &command.command {
                EmitSubcommand::Rust(emit) => emit_rust_with_config(
                    workspace,
                    &config_for_roots_keep_build(config, &emit.roots, emit.keep_build_dir),
                ),
                EmitSubcommand::Lowered(emit) => {
                    emit_lowered_with_config(workspace, &config_for_roots(config, &emit.roots))
                }
            },
        },
        FrontendCommand::Tool(command) => match &command.command {
            ToolSubcommand::Clean(_) => clean_workspace_with_config(workspace, config),
            ToolSubcommand::Lsp(_)
            | ToolSubcommand::Parse(_)
            | ToolSubcommand::Highlight(_)
            | ToolSubcommand::Symbols(_)
            | ToolSubcommand::Tree(_) => Err(FrontendError::new(
                FrontendErrorKind::Internal,
                "unexpected editor command reached workspace dispatcher",
            )),
            ToolSubcommand::Completion(_) => Err(FrontendError::new(
                FrontendErrorKind::Internal,
                "unexpected completion command reached workspace dispatcher",
            )),
        },
        FrontendCommand::Complete(_) => Err(FrontendError::new(
            FrontendErrorKind::Internal,
            "unexpected command reached workspace dispatcher",
        )),
    }
}

fn emit_has_direct_target(command: &EmitCommand) -> bool {
    match &command.command {
        EmitSubcommand::Rust(emit) => emit.target.input.is_some(),
        EmitSubcommand::Lowered(emit) => emit.target.input.is_some(),
    }
}

fn config_for_roots(base: &FrontendConfig, roots: &cli::CompileRootArgs) -> FrontendConfig {
    let mut config = base.clone();
    if let Some(std_root) = &roots.std_root {
        config.std_root_override = Some(std_root.into());
    }
    if let Some(package_store_root) = &roots.package_store_root {
        config.package_store_root_override = Some(package_store_root.into());
    }
    config
}

fn config_for_roots_keep_build(
    base: &FrontendConfig,
    roots: &cli::CompileRootArgs,
    keep_build_dir: bool,
) -> FrontendConfig {
    let mut config = config_for_roots(base, roots);
    config.keep_build_dir = keep_build_dir;
    config
}

fn discovered_root_for_command(
    command: &FrontendCommand,
    working_directory: &std::path::Path,
) -> FrontendResult<DiscoveredRoot> {
    let explicit = match command {
        FrontendCommand::Work(command) => command.path.as_deref(),
        FrontendCommand::Code(command) => match &command.command {
            CodeSubcommand::Test(command) => command.path.as_deref(),
            _ => None,
        },
        _ => None,
    };
    if let Some(path) = explicit {
        require_discovered_root(std::path::Path::new(path))
    } else {
        require_discovered_root(working_directory)
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

        let (_, result) = run_command_from_args_in_dir(["fol", "code", "check"], &root).unwrap();

        assert_eq!(result.command, "check");
        assert!(result.summary.contains("checked 1 workspace package(s)"));

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn root_help_stays_root_owned() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = run_from_args_with_io(["fol", "--help"], &mut stdout, &mut stderr);
        let rendered = String::from_utf8(stdout).expect("help output should be utf8");

        assert_eq!(code, 0);
        assert!(stderr.is_empty());
        assert!(rendered.contains("User-facing frontend for the FOL toolchain"));
        assert!(rendered.contains("Run `fol <command> --help` for command-specific usage."));
        assert!(!rendered.contains("Workflow Commands:"));
    }

    #[test]
    fn subcommand_help_is_not_swallowed_by_root_help() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let code = run_from_args_with_io(["fol", "code", "emit", "--help"], &mut stdout, &mut stderr);
        let rendered = String::from_utf8(stdout).expect("help output should be utf8");

        assert_eq!(code, 0);
        assert!(stderr.is_empty());
        assert!(rendered.contains("Usage: fol code emit"));
        assert!(rendered.contains("Commands:"));
        assert!(rendered.contains("rust"));
        assert!(rendered.contains("lowered"));
        assert!(!rendered.contains("Run `fol <command> --help` for command-specific usage."));
    }

    #[test]
    fn frontend_config_from_cli_keeps_build_option_overrides() {
        let cli = FrontendCli::parse_from([
            "fol",
            "code",
            "build",
            "--target",
            "aarch64-macos-gnu",
            "--optimize",
            "release-fast",
            "--build-option",
            "jobs=16",
        ]);

        let config = frontend_config_from_cli(&cli, None);

        assert_eq!(
            config.build_target_override.as_deref(),
            Some("aarch64-macos-gnu")
        );
        assert_eq!(config.build_optimize_override.as_deref(), Some("release-fast"));
        assert_eq!(config.build_option_overrides, vec!["jobs=16".to_string()]);
    }
}
fn code_has_direct_target(command: &CodeCommand) -> bool {
    match &command.command {
        CodeSubcommand::Build(command) => command.target.input.is_some(),
        CodeSubcommand::Run(command) => command.target.input.is_some(),
        CodeSubcommand::Check(command) => command.target.input.is_some(),
        CodeSubcommand::Emit(command) => emit_has_direct_target(command),
        CodeSubcommand::Test(_) => false,
    }
}
