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
mod errors;
mod fetch;
mod output;
mod result;
mod scaffold;
mod ui;
mod work;
mod workspace;

pub use cli::{
    BuildCommand, CheckCommand, CompleteCommand, CompletionCommand, CompletionShellArg,
    EmitCommand, EmitLoweredCommand, EmitRustCommand, EmitSubcommand, FetchCommand,
    FrontendCli, FrontendCommand, FrontendProfile, InitCommand, NewCommand, RunCommand,
    TestCommand, UnitCommand, UpdateCommand,
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

    if wants_help(&args) {
        return match writeln!(stdout, "{}", FrontendCli::render_root_help()) {
            Ok(()) => 0,
            Err(error) => {
                let _ = writeln!(stderr, "FrontendInternal: {error}");
                1
            }
        };
    }
    if wants_version(&args) {
        return match writeln!(stdout, "{}", env!("CARGO_PKG_VERSION")) {
            Ok(()) => 0,
            Err(error) => {
                let _ = writeln!(stderr, "FrontendInternal: {error}");
                1
            }
        };
    }

    match FrontendCli::try_parse_from(args.clone()) {
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
        Ok(cli) if cli.command.is_none() && cli.input.is_none() => {
            match writeln!(stdout, "{}", FrontendCli::render_root_help()) {
                Ok(()) => 0,
                Err(error) => {
                    let _ = writeln!(stderr, "FrontendInternal: {error}");
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

fn wants_help(args: &[std::ffi::OsString]) -> bool {
    args.iter().skip(1).any(|arg| arg == "-h" || arg == "--help")
}

fn wants_version(args: &[std::ffi::OsString]) -> bool {
    args.iter().skip(1).any(|arg| arg == "-V" || arg == "--version")
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
    config.output.mode = if cli.json { OutputMode::Json } else { cli.output };
    config.profile_override = Some(cli.selected_profile());
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
        Some(FrontendCommand::Fetch(command)) => {
            config.locked_fetch = command.locked;
            config.offline_fetch = command.offline;
            config.refresh_fetch = command.refresh;
        }
        Some(FrontendCommand::Update(_)) => {
            config.refresh_fetch = true;
        }
        Some(FrontendCommand::Build(command)) => {
            config.locked_fetch = command.locked;
        }
        Some(FrontendCommand::Run(command)) => {
            config.locked_fetch = command.locked;
        }
        Some(FrontendCommand::Test(command)) => {
            config.locked_fetch = command.locked;
        }
        Some(FrontendCommand::Check(command)) => {
            config.locked_fetch = command.locked;
        }
        _ => {}
    }
    config
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
        Some(FrontendCommand::Update(_)) => {
            let discovered = require_discovered_root(&config.working_directory)?;
            let workspace = load_frontend_workspace(&discovered, config)?;
            update_workspace_with_config(&workspace, config)
        }
        Some(FrontendCommand::Completion(command)) => {
            completion_command(parse_completion_shell(command.shell))
        }
        Some(FrontendCommand::Complete(command)) => {
            internal_complete_command_with_tokens(&command.tokens)
        }
        Some(FrontendCommand::Build(command)) if command.target.input.is_some() => {
            run_direct_compile(
                &DirectCompileConfig {
                    input: command.target.input.clone().unwrap_or_default(),
                    std_root: command.roots.std_root.clone(),
                    package_store_root: command.roots.package_store_root.clone(),
                    mode: DirectCompileMode::Build {
                        keep_build_dir: command.keep_build_dir,
                    },
                },
                &config_for_roots_keep_build(config, &command.roots, command.keep_build_dir),
            )
        }
        Some(FrontendCommand::Check(command)) if command.target.input.is_some() => {
            run_direct_compile(
                &DirectCompileConfig {
                    input: command.target.input.clone().unwrap_or_default(),
                    std_root: command.roots.std_root.clone(),
                    package_store_root: command.roots.package_store_root.clone(),
                    mode: DirectCompileMode::Check,
                },
                &config_for_roots(config, &command.roots),
            )
        }
        Some(FrontendCommand::Run(command)) if command.target.input.is_some() => {
            run_direct_compile(
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
            )
        }
        Some(FrontendCommand::Emit(command)) if emit_has_direct_target(command) => {
            match &command.command {
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
            }
        }
        Some(command) => {
            let discovered = discovered_root_for_command(command, &config.working_directory)?;
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
        FrontendCommand::Fetch(command) => {
            fetch_workspace_with_config(workspace, &config_for_roots(config, &command.roots))
        }
        FrontendCommand::Update(command) => {
            update_workspace_with_config(workspace, &config_for_roots(config, &command.roots))
        }
        FrontendCommand::Build(command) => {
            if let Some(input) = &command.target.input {
                return run_direct_compile(
                    &DirectCompileConfig {
                        input: input.clone(),
                        std_root: command.roots.std_root.clone(),
                        package_store_root: command.roots.package_store_root.clone(),
                        mode: DirectCompileMode::Build {
                            keep_build_dir: command.keep_build_dir,
                        },
                    },
                    &config_for_roots_keep_build(config, &command.roots, command.keep_build_dir),
                );
            }
            build_workspace_for_profile_with_config(
                workspace,
                &config_for_roots_keep_build(config, &command.roots, command.keep_build_dir),
                config.profile_override.unwrap_or(FrontendProfile::Debug),
            )
        }
        FrontendCommand::Check(command) => {
            if let Some(input) = &command.target.input {
                return run_direct_compile(
                    &DirectCompileConfig {
                        input: input.clone(),
                        std_root: command.roots.std_root.clone(),
                        package_store_root: command.roots.package_store_root.clone(),
                        mode: DirectCompileMode::Check,
                    },
                    &config_for_roots(config, &command.roots),
                );
            }
            check_workspace_with_config(workspace, &config_for_roots(config, &command.roots))
        }
        FrontendCommand::Run(command) => {
            if let Some(input) = &command.target.input {
                return run_direct_compile(
                    &DirectCompileConfig {
                        input: input.clone(),
                        std_root: command.roots.std_root.clone(),
                        package_store_root: command.roots.package_store_root.clone(),
                        mode: DirectCompileMode::Run {
                            keep_build_dir: command.keep_build_dir,
                            args: command.args.clone(),
                        },
                    },
                    &config_for_roots_keep_build(config, &command.roots, command.keep_build_dir),
                );
            }
            run_workspace_with_args_and_config(
                workspace,
                &config_for_roots_keep_build(config, &command.roots, command.keep_build_dir),
                &command.args,
            )
        }
        FrontendCommand::Test(_) => test_workspace_with_config(workspace, config),
        FrontendCommand::Emit(command) => match command.command {
            EmitSubcommand::Rust(ref emit) => {
                if let Some(input) = &emit.target.input {
                    return run_direct_compile(
                        &DirectCompileConfig {
                            input: input.clone(),
                            std_root: emit.roots.std_root.clone(),
                            package_store_root: emit.roots.package_store_root.clone(),
                            mode: DirectCompileMode::EmitRust {
                                keep_build_dir: emit.keep_build_dir,
                            },
                        },
                        &config_for_roots_keep_build(config, &emit.roots, emit.keep_build_dir),
                    );
                }
                emit_rust_with_config(
                    workspace,
                    &config_for_roots_keep_build(config, &emit.roots, emit.keep_build_dir),
                )
            }
            EmitSubcommand::Lowered(ref emit) => {
                if let Some(input) = &emit.target.input {
                    return run_direct_compile(
                        &DirectCompileConfig {
                            input: input.clone(),
                            std_root: emit.roots.std_root.clone(),
                            package_store_root: emit.roots.package_store_root.clone(),
                            mode: DirectCompileMode::EmitLowered,
                        },
                        &config_for_roots(config, &emit.roots),
                    );
                }
                emit_lowered_with_config(workspace, &config_for_roots(config, &emit.roots))
            }
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
        FrontendCommand::Test(command) => command.path.as_deref(),
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

        let (_, result) = run_command_from_args_in_dir(["fol", "check"], &root).unwrap();

        assert_eq!(result.command, "check");
        assert!(result.summary.contains("checked 1 workspace package(s)"));

        std::fs::remove_dir_all(root).ok();
    }
}
