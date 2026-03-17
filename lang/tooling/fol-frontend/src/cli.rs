use crate::OutputMode;
use clap::{Args, CommandFactory, Parser, Subcommand};

const AFTER_HELP: &str = "Run `fol <group> <command> --help` for command-specific usage.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum FrontendProfile {
    Debug,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CompletionShellArg {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct FrontendOutputArgs {
    #[arg(
        long,
        env = "FOL_OUTPUT",
        value_enum,
        default_value_t = OutputMode::Human,
        help = "Select frontend output mode"
    )]
    pub output: OutputMode,
}

impl Default for FrontendOutputArgs {
    fn default() -> Self {
        Self {
            output: OutputMode::Human,
        }
    }
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct FrontendProfileArgs {
    #[arg(
        long,
        env = "FOL_PROFILE",
        value_enum,
        help = "Select the build profile"
    )]
    pub profile: Option<FrontendProfile>,

    #[arg(
        long,
        conflicts_with_all = ["release", "profile"],
        help = "Force the debug profile"
    )]
    pub debug: bool,

    #[arg(
        long,
        conflicts_with_all = ["debug", "profile"],
        help = "Force the release profile"
    )]
    pub release: bool,
}

impl FrontendProfileArgs {
    pub fn selected_profile(&self) -> FrontendProfile {
        if self.release {
            FrontendProfile::Release
        } else if self.debug {
            FrontendProfile::Debug
        } else {
            self.profile.unwrap_or(FrontendProfile::Debug)
        }
    }
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum FrontendCommand {
    #[command(visible_aliases = ["w", "ws", "workspace"])]
    Work(WorkCommand),
    #[command(visible_aliases = ["pkg", "package"])]
    Pack(PackCommand),
    Code(CodeCommand),
    Tool(ToolCommand),
    #[command(hide = true, name = "_complete")]
    Complete(CompleteCommand),
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct UnitCommand;

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct CompileRootArgs {
    #[arg(long, value_name = "DIR", help = "Override the standard-library root")]
    pub std_root: Option<String>,

    #[arg(
        long,
        value_name = "DIR",
        help = "Override the installed package-store root"
    )]
    pub package_store_root: Option<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct DirectTargetArg {
    #[arg(value_name = "PATH")]
    pub input: Option<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct FetchCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub roots: CompileRootArgs,

    #[arg(long, help = "Require the existing fol.lock to match the manifest")]
    pub locked: bool,

    #[arg(long, help = "Use only already-cached git sources")]
    pub offline: bool,

    #[arg(long, help = "Force a fresh git fetch for remote dependencies")]
    pub refresh: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct UpdateCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub roots: CompileRootArgs,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct BuildCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub profile: FrontendProfileArgs,

    #[command(flatten)]
    pub target: DirectTargetArg,

    #[command(flatten)]
    pub roots: CompileRootArgs,

    #[arg(long, help = "Require the existing fol.lock to match the manifest")]
    pub locked: bool,

    #[arg(long, help = "Keep the generated backend crate directory")]
    pub keep_build_dir: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct RunCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub profile: FrontendProfileArgs,

    #[command(flatten)]
    pub target: DirectTargetArg,

    #[command(flatten)]
    pub roots: CompileRootArgs,

    #[arg(long, help = "Require the existing fol.lock to match the manifest")]
    pub locked: bool,

    #[arg(long, help = "Keep the generated backend crate directory")]
    pub keep_build_dir: bool,

    #[arg(last = true, trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct TestCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub profile: FrontendProfileArgs,

    #[arg(long, value_name = "PATH", help = "Override the workspace or package root")]
    pub path: Option<String>,

    #[arg(long, help = "Require the existing fol.lock to match the manifest")]
    pub locked: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct CheckCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub profile: FrontendProfileArgs,

    #[command(flatten)]
    pub target: DirectTargetArg,

    #[command(flatten)]
    pub roots: CompileRootArgs,

    #[arg(long, help = "Require the existing fol.lock to match the manifest")]
    pub locked: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct WorkCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[arg(long, value_name = "PATH", help = "Override the workspace or package root")]
    pub path: Option<String>,

    #[command(subcommand)]
    pub command: WorkSubcommand,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct PackCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(subcommand)]
    pub command: PackSubcommand,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct CodeCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub profile: FrontendProfileArgs,

    #[command(subcommand)]
    pub command: CodeSubcommand,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct ToolCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(subcommand)]
    pub command: ToolSubcommand,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct CompletionCommand {
    #[arg(value_enum)]
    pub shell: CompletionShellArg,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct CompleteCommand {
    #[arg(trailing_var_arg = true)]
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct EmitCommand {
    #[command(subcommand)]
    pub command: EmitSubcommand,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum WorkSubcommand {
    Init(InitCommand),
    New(NewCommand),
    Info(UnitCommand),
    List(UnitCommand),
    Deps(UnitCommand),
    Status(UnitCommand),
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum PackSubcommand {
    #[command(visible_aliases = ["f", "sync"])]
    Fetch(FetchCommand),
    #[command(visible_aliases = ["u", "upgrade"])]
    Update(UpdateCommand),
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum CodeSubcommand {
    #[command(visible_aliases = ["b", "make"])]
    Build(BuildCommand),
    #[command(visible_aliases = ["r"])]
    Run(RunCommand),
    #[command(visible_aliases = ["t"])]
    Test(TestCommand),
    #[command(visible_aliases = ["c", "verify"])]
    Check(CheckCommand),
    #[command(visible_aliases = ["e", "gen"])]
    Emit(EmitCommand),
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum ToolSubcommand {
    #[command(visible_aliases = ["cl", "purge"])]
    Clean(UnitCommand),
    #[command(visible_aliases = ["completions", "comp"])]
    Completion(CompletionCommand),
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum EmitSubcommand {
    Rust(EmitRustCommand),
    Lowered(EmitLoweredCommand),
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct EmitRustCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub profile: FrontendProfileArgs,

    #[command(flatten)]
    pub target: DirectTargetArg,

    #[command(flatten)]
    pub roots: CompileRootArgs,

    #[arg(long, help = "Keep the generated backend crate directory")]
    pub keep_build_dir: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct EmitLoweredCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub profile: FrontendProfileArgs,

    #[command(flatten)]
    pub target: DirectTargetArg,

    #[command(flatten)]
    pub roots: CompileRootArgs,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct InitCommand {
    #[arg(long)]
    pub workspace: bool,

    #[arg(long, conflicts_with = "lib")]
    pub bin: bool,

    #[arg(long, conflicts_with = "bin")]
    pub lib: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct NewCommand {
    pub name: String,

    #[arg(long)]
    pub workspace: bool,

    #[arg(long, conflicts_with = "lib")]
    pub bin: bool,

    #[arg(long, conflicts_with = "bin")]
    pub lib: bool,
}

#[derive(Debug, Clone, Parser, PartialEq, Eq)]
#[command(
    name = "fol",
    version,
    about = "User-facing frontend for the FOL toolchain",
    disable_help_subcommand = true
)]
pub struct FrontendCli {
    #[arg(
        value_name = "FILE_OR_FOLDER",
        help = "Input FOL file or folder to build directly"
    )]
    pub input: Option<String>,

    #[arg(
        long,
        hide = true,
        env = "FOL_OUTPUT",
        value_enum,
        default_value_t = OutputMode::Human
    )]
    pub output: OutputMode,

    #[arg(long, global = true, hide = true, action = clap::ArgAction::SetTrue)]
    pub json: bool,

    #[arg(
        long,
        hide = true,
        env = "FOL_PROFILE",
        value_enum,
    )]
    pub profile: Option<FrontendProfile>,

    #[arg(
        long,
        hide = true,
        conflicts_with_all = ["release", "profile"],
    )]
    pub debug: bool,

    #[arg(
        long,
        hide = true,
        conflicts_with_all = ["debug", "profile"],
    )]
    pub release: bool,

    #[arg(long, global = true, hide = true, value_name = "DIR")]
    pub std_root: Option<String>,

    #[arg(long, global = true, hide = true, value_name = "DIR")]
    pub package_store_root: Option<String>,

    #[arg(long, global = true, hide = true)]
    pub dump_lowered: bool,

    #[arg(long, global = true, hide = true)]
    pub emit_rust: bool,

    #[arg(long, global = true, hide = true)]
    pub keep_build_dir: bool,

    #[command(subcommand)]
    pub command: Option<FrontendCommand>,
}

impl FrontendCli {
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        <Self as Parser>::parse_from(args)
    }

    pub fn try_parse_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        <Self as Parser>::try_parse_from(args)
    }

    pub fn command() -> clap::Command {
        <Self as CommandFactory>::command()
        .color(clap::ColorChoice::Auto)
        .after_help(AFTER_HELP)
    }

    pub fn selected_profile(&self) -> FrontendProfile {
        if self.release {
            FrontendProfile::Release
        } else if self.debug {
            FrontendProfile::Debug
        } else {
            self.profile.unwrap_or(FrontendProfile::Debug)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildCommand, CheckCommand, CodeCommand, CodeSubcommand, CompleteCommand,
        CompletionCommand, CompletionShellArg, CompileRootArgs, DirectTargetArg, EmitCommand,
        EmitLoweredCommand, EmitRustCommand, EmitSubcommand, FetchCommand, FrontendCli,
        FrontendCommand, FrontendOutputArgs, FrontendProfile, FrontendProfileArgs, InitCommand,
        NewCommand, PackCommand, PackSubcommand, RunCommand, TestCommand, ToolCommand,
        ToolSubcommand, UnitCommand, UpdateCommand, WorkCommand, WorkSubcommand,
    };
    use crate::OutputMode;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn env_lock() -> MutexGuard<'static, ()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env test lock should not be poisoned")
    }

    fn parse_clean<const N: usize>(args: [&str; N]) -> FrontendCli {
        let _guard = env_lock();
        std::env::remove_var("FOL_OUTPUT");
        std::env::remove_var("FOL_PROFILE");
        FrontendCli::parse_from(args)
    }

    fn default_output_args() -> FrontendOutputArgs {
        FrontendOutputArgs::default()
    }

    fn default_profile_args() -> FrontendProfileArgs {
        FrontendProfileArgs::default()
    }

    #[test]
    fn derive_root_parser_accepts_empty_invocation() {
        let _guard = env_lock();
        std::env::remove_var("FOL_OUTPUT");
        std::env::remove_var("FOL_PROFILE");
        let cli = parse_clean(["fol"]);

        assert_eq!(cli.output, OutputMode::Human);
        assert_eq!(cli.selected_profile(), FrontendProfile::Debug);
        assert_eq!(cli.command, None);
    }

    #[test]
    fn root_command_families_parse_through_derive_tree() {
        let cli = parse_clean(["fol", "code", "build"]);

        assert_eq!(
            cli.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Build(BuildCommand::default()),
            }))
        );
    }

    #[test]
    fn run_command_preserves_passthrough_args() {
        let cli = parse_clean(["fol", "code", "run", "--", "--flag", "value"]);

        assert_eq!(
            cli.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Run(RunCommand {
                    output: default_output_args(),
                    profile: default_profile_args(),
                    target: DirectTargetArg::default(),
                    roots: CompileRootArgs::default(),
                    locked: false,
                    keep_build_dir: false,
                    args: vec!["--flag".to_string(), "value".to_string()],
                }),
            }))
        );
    }

    #[test]
    fn emit_subcommands_parse_through_derive_tree() {
        let rust = parse_clean(["fol", "code", "emit", "rust"]);
        let lowered = parse_clean(["fol", "code", "emit", "lowered"]);

        assert_eq!(
            rust.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Emit(EmitCommand {
                    command: EmitSubcommand::Rust(EmitRustCommand::default()),
                }),
            }))
        );
        assert_eq!(
            lowered.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Emit(EmitCommand {
                    command: EmitSubcommand::Lowered(EmitLoweredCommand::default()),
                }),
            }))
        );
    }

    #[test]
    fn completion_command_parses_requested_shell() {
        let cli = parse_clean(["fol", "tool", "completion", "bash"]);

        assert_eq!(
            cli.command,
            Some(FrontendCommand::Tool(ToolCommand {
                output: default_output_args(),
                command: ToolSubcommand::Completion(CompletionCommand {
                    shell: CompletionShellArg::Bash,
                }),
            }))
        );
    }

    #[test]
    fn internal_complete_command_parses_optional_current_token() {
        let cli = parse_clean(["fol", "_complete", "code", "emit", "ru"]);

        assert_eq!(
            cli.command,
            Some(FrontendCommand::Complete(CompleteCommand {
                tokens: vec!["code".to_string(), "emit".to_string(), "ru".to_string()],
            }))
        );
    }

    #[test]
    fn visible_aliases_parse_to_the_same_root_commands() {
        let build = parse_clean(["fol", "code", "make"]);
        let check = parse_clean(["fol", "code", "verify"]);
        let work = parse_clean(["fol", "workspace", "info"]);
        let fetch = parse_clean(["fol", "pack", "sync"]);
        let update = parse_clean(["fol", "pack", "upgrade"]);
        let emit = parse_clean(["fol", "code", "gen", "rust"]);
        let clean = parse_clean(["fol", "tool", "purge"]);

        assert_eq!(build.command, Some(FrontendCommand::Code(CodeCommand {
            output: default_output_args(),
            profile: default_profile_args(),
            command: CodeSubcommand::Build(BuildCommand::default()),
        })));
        assert_eq!(check.command, Some(FrontendCommand::Code(CodeCommand {
            output: default_output_args(),
            profile: default_profile_args(),
            command: CodeSubcommand::Check(CheckCommand::default()),
        })));
        assert_eq!(fetch.command, Some(FrontendCommand::Pack(PackCommand {
            output: default_output_args(),
            command: PackSubcommand::Fetch(FetchCommand::default()),
        })));
        assert_eq!(update.command, Some(FrontendCommand::Pack(PackCommand {
            output: default_output_args(),
            command: PackSubcommand::Update(UpdateCommand::default()),
        })));
        assert_eq!(
            emit.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Emit(EmitCommand {
                    command: EmitSubcommand::Rust(EmitRustCommand::default()),
                }),
            }))
        );
        assert_eq!(clean.command, Some(FrontendCommand::Tool(ToolCommand {
            output: default_output_args(),
            command: ToolSubcommand::Clean(UnitCommand),
        })));
        assert_eq!(
            work.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::Info(UnitCommand),
            }))
        );
    }

    #[test]
    fn output_flag_parses_global_output_mode() {
        let cli = parse_clean(["fol", "code", "--output", "json", "build"]);

        assert_eq!(cli.output, OutputMode::Human);
        assert_eq!(
            cli.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: FrontendOutputArgs { output: OutputMode::Json },
                profile: default_profile_args(),
                command: CodeSubcommand::Build(BuildCommand::default()),
            }))
        );
    }

    #[test]
    fn profile_flags_normalize_to_frontend_profile_selection() {
        let profile = parse_clean(["fol", "code", "--profile", "release", "build"]);
        let release = parse_clean(["fol", "code", "--release", "build"]);

        assert_eq!(
            profile.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: FrontendProfileArgs {
                    profile: Some(FrontendProfile::Release),
                    debug: false,
                    release: false,
                },
                command: CodeSubcommand::Build(BuildCommand::default()),
            }))
        );
        assert_eq!(
            release.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: FrontendProfileArgs {
                    profile: None,
                    debug: false,
                    release: true,
                },
                command: CodeSubcommand::Build(BuildCommand::default()),
            }))
        );
    }

    #[test]
    fn cli_env_values_feed_output_and_profile_defaults() {
        let _guard = env_lock();
        std::env::set_var("FOL_OUTPUT", "plain");
        std::env::set_var("FOL_PROFILE", "release");

        let cli = FrontendCli::parse_from(["fol", "code", "build"]);

        assert_eq!(cli.output, OutputMode::Human);
        assert_eq!(
            cli.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: FrontendOutputArgs { output: OutputMode::Plain },
                profile: FrontendProfileArgs {
                    profile: Some(FrontendProfile::Release),
                    debug: false,
                    release: false,
                },
                command: CodeSubcommand::Build(BuildCommand::default()),
            }))
        );

        std::env::remove_var("FOL_OUTPUT");
        std::env::remove_var("FOL_PROFILE");
    }

    #[test]
    fn explicit_flags_override_env_values() {
        let _guard = env_lock();
        std::env::set_var("FOL_OUTPUT", "plain");
        std::env::set_var("FOL_PROFILE", "release");

        let cli = FrontendCli::parse_from(["fol", "code", "--output", "json", "--debug", "build"]);

        assert_eq!(cli.output, OutputMode::Human);
        assert_eq!(
            cli.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: FrontendOutputArgs { output: OutputMode::Json },
                profile: FrontendProfileArgs {
                    profile: None,
                    debug: true,
                    release: false,
                },
                command: CodeSubcommand::Build(BuildCommand::default()),
            }))
        );

        std::env::remove_var("FOL_OUTPUT");
        std::env::remove_var("FOL_PROFILE");
    }

    #[test]
    fn help_output_points_users_to_subcommand_help() {
        let help = FrontendCli::command().render_long_help().to_string();

        assert!(help.contains("Run `fol <command> --help` for command-specific usage."));
        assert!(!help.contains("Workflow Commands:"));
        assert!(!help.contains("Workspace Commands:"));
        assert!(!help.contains("Shell Commands:"));
        assert!(!help.contains("Examples:"));
    }

    #[test]
    fn help_output_keeps_global_mode_flags_visible() {
        let help = FrontendCli::command().render_long_help().to_string();

        assert!(!help.contains("--output"));
        assert!(!help.contains("--profile"));
        assert!(!help.contains("--debug"));
        assert!(!help.contains("--release"));
        assert!(!help.contains("--dump-lowered"));
        assert!(!help.contains("--emit-rust"));
        assert!(!help.contains("--keep-build-dir"));
        assert!(help.contains("Arguments:"));
        assert!(help.contains("FILE_OR_FOLDER"));
    }

    #[test]
    fn help_output_mentions_visible_aliases() {
        let help = FrontendCli::command().render_long_help().to_string();

        assert!(help.contains("work"));
        assert!(help.contains("workspace"));
        assert!(help.contains("pack"));
        assert!(help.contains("package"));
        assert!(help.contains("code"));
        assert!(help.contains("tool"));
    }

    #[test]
    fn work_subcommands_parse_for_info_and_list() {
        let info = parse_clean(["fol", "work", "info"]);
        let list = parse_clean(["fol", "work", "list"]);
        let deps = parse_clean(["fol", "work", "deps"]);
        let status = parse_clean(["fol", "work", "status"]);

        assert_eq!(
            info.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::Info(UnitCommand),
            }))
        );
        assert_eq!(
            list.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::List(UnitCommand),
            }))
        );
        assert_eq!(
            deps.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::Deps(UnitCommand),
            }))
        );
        assert_eq!(
            status.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::Status(UnitCommand),
            }))
        );
    }

    #[test]
    fn workspace_flags_parse_for_init_and_new_commands() {
        let init = parse_clean(["fol", "work", "init", "--workspace"]);
        let new = parse_clean(["fol", "work", "new", "demo", "--workspace"]);

        assert_eq!(
            init.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::Init(InitCommand { workspace: true, bin: false, lib: false }),
            }))
        );
        assert_eq!(
            new.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::New(NewCommand {
                    name: "demo".to_string(),
                    workspace: true,
                    bin: false,
                    lib: false,
                }),
            }))
        );
    }

    #[test]
    fn bin_flags_parse_for_init_and_new_commands() {
        let init = parse_clean(["fol", "work", "init", "--bin"]);
        let new = parse_clean(["fol", "work", "new", "demo", "--bin"]);

        assert_eq!(
            init.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::Init(InitCommand { workspace: false, bin: true, lib: false }),
            }))
        );
        assert_eq!(
            new.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::New(NewCommand {
                    name: "demo".to_string(),
                    workspace: false,
                    bin: true,
                    lib: false,
                }),
            }))
        );
    }

    #[test]
    fn duplicate_lib_flags_parse_for_init_and_new_commands() {
        let init = parse_clean(["fol", "work", "init", "--lib"]);
        let new = parse_clean(["fol", "work", "new", "demo", "--lib"]);

        assert_eq!(
            init.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::Init(InitCommand { workspace: false, bin: false, lib: true }),
            }))
        );
        assert_eq!(
            new.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::New(NewCommand {
                    name: "demo".to_string(),
                    workspace: false,
                    bin: false,
                    lib: true,
                }),
            }))
        );
    }

    #[test]
    fn lib_flags_parse_for_init_and_new_commands() {
        let init = parse_clean(["fol", "work", "init", "--lib"]);
        let new = parse_clean(["fol", "work", "new", "demo", "--lib"]);

        assert_eq!(
            init.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::Init(InitCommand { workspace: false, bin: false, lib: true }),
            }))
        );
        assert_eq!(
            new.command,
            Some(FrontendCommand::Work(WorkCommand {
                output: default_output_args(),
                path: None,
                command: WorkSubcommand::New(NewCommand {
                    name: "demo".to_string(),
                    workspace: false,
                    bin: false,
                    lib: true,
                }),
            }))
        );
    }

    #[test]
    fn build_command_owns_direct_compile_flags() {
        let cli = parse_clean([
            "fol",
            "code",
            "build",
            "--std-root",
            "/tmp/std",
            "--package-store-root",
            "/tmp/pkg",
            "--keep-build-dir",
            "demo",
        ]);

        assert_eq!(
            cli.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Build(BuildCommand {
                    output: default_output_args(),
                    profile: default_profile_args(),
                    target: DirectTargetArg {
                        input: Some("demo".to_string()),
                    },
                    roots: CompileRootArgs {
                        std_root: Some("/tmp/std".to_string()),
                        package_store_root: Some("/tmp/pkg".to_string()),
                    },
                    locked: false,
                    keep_build_dir: true,
                }),
            }))
        );
    }

    #[test]
    fn fetch_and_locked_workflow_flags_parse_on_commands() {
        let fetch = parse_clean(["fol", "pack", "fetch", "--locked", "--offline", "--refresh"]);
        let build = parse_clean(["fol", "code", "build", "--locked"]);
        let run = parse_clean(["fol", "code", "run", "--locked"]);
        let test = parse_clean(["fol", "code", "test", "--locked"]);
        let check = parse_clean(["fol", "code", "check", "--locked"]);

        assert_eq!(
            fetch.command,
            Some(FrontendCommand::Pack(PackCommand {
                output: default_output_args(),
                command: PackSubcommand::Fetch(FetchCommand {
                    output: default_output_args(),
                    roots: CompileRootArgs::default(),
                    locked: true,
                    offline: true,
                    refresh: true,
                }),
            }))
        );
        assert_eq!(
            build.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Build(BuildCommand {
                    output: default_output_args(),
                    profile: default_profile_args(),
                    target: DirectTargetArg::default(),
                    roots: CompileRootArgs::default(),
                    locked: true,
                    keep_build_dir: false,
                }),
            }))
        );
        assert_eq!(
            run.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Run(RunCommand {
                    output: default_output_args(),
                    profile: default_profile_args(),
                    target: DirectTargetArg::default(),
                    roots: CompileRootArgs::default(),
                    locked: true,
                    keep_build_dir: false,
                    args: Vec::new(),
                }),
            }))
        );
        assert_eq!(
            test.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Test(TestCommand {
                    output: default_output_args(),
                    profile: default_profile_args(),
                    path: None,
                    locked: true,
                }),
            }))
        );
        assert_eq!(
            check.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Check(CheckCommand {
                    output: default_output_args(),
                    profile: default_profile_args(),
                    target: DirectTargetArg::default(),
                    roots: CompileRootArgs::default(),
                    locked: true,
                }),
            }))
        );
    }

    #[test]
    fn emit_subcommands_own_their_specific_flags() {
        let rust = parse_clean([
            "fol",
            "code",
            "emit",
            "rust",
            "--keep-build-dir",
            "demo",
        ]);
        let lowered = parse_clean([
            "fol",
            "code",
            "emit",
            "lowered",
            "--std-root",
            "/tmp/std",
            "demo",
        ]);

        assert_eq!(
            rust.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Emit(EmitCommand {
                    command: EmitSubcommand::Rust(EmitRustCommand {
                        output: default_output_args(),
                        profile: default_profile_args(),
                        target: DirectTargetArg {
                            input: Some("demo".to_string()),
                        },
                        roots: CompileRootArgs::default(),
                        keep_build_dir: true,
                    }),
                }),
            }))
        );
        assert_eq!(
            lowered.command,
            Some(FrontendCommand::Code(CodeCommand {
                output: default_output_args(),
                profile: default_profile_args(),
                command: CodeSubcommand::Emit(EmitCommand {
                    command: EmitSubcommand::Lowered(EmitLoweredCommand {
                        output: default_output_args(),
                        profile: default_profile_args(),
                        target: DirectTargetArg {
                            input: Some("demo".to_string()),
                        },
                        roots: CompileRootArgs {
                            std_root: Some("/tmp/std".to_string()),
                            package_store_root: None,
                        },
                    }),
                }),
            }))
        );
    }
}
