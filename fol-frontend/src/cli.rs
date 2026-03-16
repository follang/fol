use crate::OutputMode;
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap::builder::styling::{AnsiColor, Effects, Styles};

const AFTER_HELP: &str = "\
Workflow Commands:
  init, new, fetch, update, check, build, run, test, emit, clean

Workspace Commands:
  work

Shell Commands:
  completion

Examples:
  fol init --bin
  fol new demo --lib
  fol fetch
  fol update
  fol build --release
  fol run
  fol emit rust
  fol completion bash
";

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

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum FrontendCommand {
    #[command(visible_aliases = ["i", "bootstrap"])]
    Init(InitCommand),
    #[command(visible_aliases = ["n", "create"])]
    New(NewCommand),
    #[command(visible_aliases = ["w", "ws", "workspace"])]
    Work(WorkCommand),
    #[command(visible_aliases = ["f", "sync"])]
    Fetch(FetchCommand),
    #[command(visible_aliases = ["u", "upgrade"])]
    Update(UpdateCommand),
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
    #[command(visible_aliases = ["cl", "purge"])]
    Clean(UnitCommand),
    #[command(visible_aliases = ["completions", "comp"])]
    Completion(CompletionCommand),
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
    pub roots: CompileRootArgs,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct BuildCommand {
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
    #[arg(long, value_name = "PATH", help = "Override the workspace or package root")]
    pub path: Option<String>,

    #[arg(long, help = "Require the existing fol.lock to match the manifest")]
    pub locked: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct CheckCommand {
    #[command(flatten)]
    pub target: DirectTargetArg,

    #[command(flatten)]
    pub roots: CompileRootArgs,

    #[arg(long, help = "Require the existing fol.lock to match the manifest")]
    pub locked: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct WorkCommand {
    #[arg(long, value_name = "PATH", help = "Override the workspace or package root")]
    pub path: Option<String>,

    #[command(subcommand)]
    pub command: WorkSubcommand,
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
    Info(UnitCommand),
    List(UnitCommand),
    Deps(UnitCommand),
    Status(UnitCommand),
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum EmitSubcommand {
    Rust(EmitRustCommand),
    Lowered(EmitLoweredCommand),
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct EmitRustCommand {
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
        global = true,
        env = "FOL_OUTPUT",
        value_enum,
        default_value_t = OutputMode::Human,
        help = "Select frontend output mode"
    )]
    pub output: OutputMode,

    #[arg(long, global = true, hide = true, action = clap::ArgAction::SetTrue)]
    pub json: bool,

    #[arg(
        long,
        global = true,
        env = "FOL_PROFILE",
        value_enum,
        help = "Select the build profile"
    )]
    pub profile: Option<FrontendProfile>,

    #[arg(
        long,
        global = true,
        conflicts_with_all = ["release", "profile"],
        help = "Force the debug profile"
    )]
    pub debug: bool,

    #[arg(
        long,
        global = true,
        conflicts_with_all = ["debug", "profile"],
        help = "Force the release profile"
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
        .color(clap::ColorChoice::Always)
        .styles(
            Styles::styled()
                .header(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
                .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
                .literal(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
                .placeholder(AnsiColor::Yellow.on_default())
        )
        .help_template(
            "\
{about-section}
Usage: {usage}

Commands:
{subcommands}

Arguments:
{positionals}

Options:
{options}

        {after-help}",
        )
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

    pub fn render_root_help() -> String {
        let heading = |text: &str| format!("\x1b[1;36m{text}\x1b[0m");
        let flag = |text: &str| format!("\x1b[1;33m{text}\x1b[0m");
        let command = |text: &str| format!("\x1b[1;32m{text}\x1b[0m");

        [
            "User-facing frontend for the FOL toolchain".to_string(),
            String::new(),
            format!("{} {}", heading("Usage:"), "fol [OPTIONS] [FILE_OR_FOLDER] [COMMAND]"),
            String::new(),
            heading("Commands:"),
            format!("  {:<12} [aliases: i, bootstrap]", command("init")),
            format!("  {:<12} [aliases: n, create]", command("new")),
            format!("  {:<12} [aliases: w, ws, workspace]", command("work")),
            format!("  {:<12} [aliases: f, sync]", command("fetch")),
            format!("  {:<12} [aliases: b, make]", command("build")),
            format!("  {:<12} [aliases: r]", command("run")),
            format!("  {:<12} [aliases: t]", command("test")),
            format!("  {:<12} [aliases: c, verify]", command("check")),
            format!("  {:<12} [aliases: e, gen]", command("emit")),
            format!("  {:<12} [aliases: cl, purge]", command("clean")),
            format!("  {:<12} [aliases: completions, comp]", command("completion")),
            String::new(),
            heading("Arguments:"),
            format!("  {:<18} Input FOL file or folder to build directly", flag("[FILE_OR_FOLDER]")),
            String::new(),
            heading("Options:"),
            format!("  {:<26} Select frontend output mode [env: FOL_OUTPUT=] [default: human] [possible values: human, plain, json]", flag("--output <OUTPUT>")),
            format!("  {:<26} Select the build profile [env: FOL_PROFILE=] [possible values: debug, release]", flag("--profile <PROFILE>")),
            format!("  {:<26} Force the debug profile", flag("--debug")),
            format!("  {:<26} Force the release profile", flag("--release")),
            format!("  {:<26} Print help", flag("-h, --help")),
            format!("  {:<26} Print version", flag("-V, --version")),
            String::new(),
            heading("Workflow Commands:"),
            "  init, new, fetch, update, check, build, run, test, emit, clean".to_string(),
            String::new(),
            heading("Workspace Commands:"),
            "  work".to_string(),
            String::new(),
            heading("Shell Commands:"),
            "  completion".to_string(),
            String::new(),
            heading("Examples:"),
            "  fol init --bin".to_string(),
            "  fol new demo --lib".to_string(),
            "  fol fetch".to_string(),
            "  fol update".to_string(),
            "  fol build --release".to_string(),
            "  fol run".to_string(),
            "  fol emit rust".to_string(),
            "  fol completion bash".to_string(),
        ]
        .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildCommand, CheckCommand, CompleteCommand, CompletionCommand, CompletionShellArg,
        CompileRootArgs, DirectTargetArg, EmitCommand, EmitLoweredCommand, EmitRustCommand,
        EmitSubcommand, FetchCommand, FrontendCli, FrontendCommand, FrontendProfile,
        InitCommand, NewCommand, RunCommand, TestCommand, UnitCommand, UpdateCommand,
        WorkCommand, WorkSubcommand,
    };
    use crate::OutputMode;

    #[test]
    fn derive_root_parser_accepts_empty_invocation() {
        let cli = FrontendCli::parse_from(["fol"]);

        assert_eq!(cli.output, OutputMode::Human);
        assert_eq!(cli.selected_profile(), FrontendProfile::Debug);
        assert_eq!(cli.command, None);
    }

    #[test]
    fn root_command_families_parse_through_derive_tree() {
        let cli = FrontendCli::parse_from(["fol", "build"]);

        assert_eq!(cli.command, Some(FrontendCommand::Build(BuildCommand::default())));
    }

    #[test]
    fn run_command_preserves_passthrough_args() {
        let cli = FrontendCli::parse_from(["fol", "run", "--", "--flag", "value"]);

        assert_eq!(
            cli.command,
            Some(FrontendCommand::Run(RunCommand {
                target: DirectTargetArg::default(),
                roots: CompileRootArgs::default(),
                locked: false,
                keep_build_dir: false,
                args: vec!["--flag".to_string(), "value".to_string()],
            }))
        );
    }

    #[test]
    fn emit_subcommands_parse_through_derive_tree() {
        let rust = FrontendCli::parse_from(["fol", "emit", "rust"]);
        let lowered = FrontendCli::parse_from(["fol", "emit", "lowered"]);

        assert_eq!(
            rust.command,
            Some(FrontendCommand::Emit(EmitCommand {
                command: EmitSubcommand::Rust(EmitRustCommand::default()),
            }))
        );
        assert_eq!(
            lowered.command,
            Some(FrontendCommand::Emit(EmitCommand {
                command: EmitSubcommand::Lowered(EmitLoweredCommand::default()),
            }))
        );
    }

    #[test]
    fn completion_command_parses_requested_shell() {
        let cli = FrontendCli::parse_from(["fol", "completion", "bash"]);

        assert_eq!(
            cli.command,
            Some(FrontendCommand::Completion(CompletionCommand {
                shell: CompletionShellArg::Bash,
            }))
        );
    }

    #[test]
    fn internal_complete_command_parses_optional_current_token() {
        let cli = FrontendCli::parse_from(["fol", "_complete", "emit", "ru"]);

        assert_eq!(
            cli.command,
            Some(FrontendCommand::Complete(CompleteCommand {
                tokens: vec!["emit".to_string(), "ru".to_string()],
            }))
        );
    }

    #[test]
    fn visible_aliases_parse_to_the_same_root_commands() {
        let build = FrontendCli::parse_from(["fol", "b"]);
        let check = FrontendCli::parse_from(["fol", "verify"]);
        let work = FrontendCli::parse_from(["fol", "workspace", "info"]);
        let fetch = FrontendCli::parse_from(["fol", "sync"]);
        let update = FrontendCli::parse_from(["fol", "upgrade"]);
        let emit = FrontendCli::parse_from(["fol", "gen", "rust"]);
        let clean = FrontendCli::parse_from(["fol", "purge"]);

        assert_eq!(build.command, Some(FrontendCommand::Build(BuildCommand::default())));
        assert_eq!(check.command, Some(FrontendCommand::Check(CheckCommand::default())));
        assert_eq!(fetch.command, Some(FrontendCommand::Fetch(FetchCommand::default())));
        assert_eq!(update.command, Some(FrontendCommand::Update(UpdateCommand::default())));
        assert_eq!(
            emit.command,
            Some(FrontendCommand::Emit(EmitCommand {
                command: EmitSubcommand::Rust(EmitRustCommand::default()),
            }))
        );
        assert_eq!(clean.command, Some(FrontendCommand::Clean(UnitCommand)));
        assert_eq!(
            work.command,
            Some(FrontendCommand::Work(WorkCommand {
                path: None,
                command: WorkSubcommand::Info(UnitCommand),
            }))
        );
    }

    #[test]
    fn output_flag_parses_global_output_mode() {
        let cli = FrontendCli::parse_from(["fol", "--output", "json", "build"]);

        assert_eq!(cli.output, OutputMode::Json);
        assert_eq!(cli.command, Some(FrontendCommand::Build(BuildCommand::default())));
    }

    #[test]
    fn profile_flags_normalize_to_frontend_profile_selection() {
        let profile = FrontendCli::parse_from(["fol", "--profile", "release", "build"]);
        let release = FrontendCli::parse_from(["fol", "--release", "build"]);

        assert_eq!(profile.selected_profile(), FrontendProfile::Release);
        assert_eq!(release.selected_profile(), FrontendProfile::Release);
    }

    #[test]
    fn cli_env_values_feed_output_and_profile_defaults() {
        unsafe {
            std::env::set_var("FOL_OUTPUT", "plain");
            std::env::set_var("FOL_PROFILE", "release");
        }

        let cli = FrontendCli::parse_from(["fol", "build"]);

        assert_eq!(cli.output, OutputMode::Plain);
        assert_eq!(cli.selected_profile(), FrontendProfile::Release);

        unsafe {
            std::env::remove_var("FOL_OUTPUT");
            std::env::remove_var("FOL_PROFILE");
        }
    }

    #[test]
    fn explicit_flags_override_env_values() {
        unsafe {
            std::env::set_var("FOL_OUTPUT", "plain");
            std::env::set_var("FOL_PROFILE", "release");
        }

        let cli = FrontendCli::parse_from(["fol", "--output", "json", "--debug", "build"]);

        assert_eq!(cli.output, OutputMode::Json);
        assert_eq!(cli.selected_profile(), FrontendProfile::Debug);

        unsafe {
            std::env::remove_var("FOL_OUTPUT");
            std::env::remove_var("FOL_PROFILE");
        }
    }

    #[test]
    fn help_output_groups_commands_by_workflow_sections() {
        let help = FrontendCli::command().render_long_help().to_string();

        assert!(help.contains("Workflow Commands:"));
        assert!(help.contains("Workspace Commands:"));
        assert!(help.contains("Shell Commands:"));
        assert!(help.contains("Examples:"));
        assert!(help.contains("fol build --release"));
        assert!(help.contains("fol update"));
    }

    #[test]
    fn help_output_keeps_global_mode_flags_visible() {
        let help = FrontendCli::command().render_long_help().to_string();

        assert!(help.contains("--output"));
        assert!(help.contains("--profile"));
        assert!(help.contains("--debug"));
        assert!(help.contains("--release"));
        assert!(!help.contains("--dump-lowered"));
        assert!(!help.contains("--emit-rust"));
        assert!(!help.contains("--keep-build-dir"));
        assert!(help.contains("Arguments:"));
        assert!(help.contains("FILE_OR_FOLDER"));
    }

    #[test]
    fn help_output_mentions_visible_aliases() {
        let help = FrontendCli::command().render_long_help().to_string();

        assert!(help.contains("build"));
        assert!(help.contains("make"));
        assert!(help.contains("check"));
        assert!(help.contains("verify"));
        assert!(help.contains("sync"));
        assert!(help.contains("upgrade"));
        assert!(help.contains("purge"));
        assert!(help.contains("gen"));
        assert!(help.contains("completion"));
        assert!(help.contains("completions"));
    }

    #[test]
    fn work_subcommands_parse_for_info_and_list() {
        let info = FrontendCli::parse_from(["fol", "work", "info"]);
        let list = FrontendCli::parse_from(["fol", "work", "list"]);
        let deps = FrontendCli::parse_from(["fol", "work", "deps"]);
        let status = FrontendCli::parse_from(["fol", "work", "status"]);

        assert_eq!(
            info.command,
            Some(FrontendCommand::Work(WorkCommand {
                path: None,
                command: WorkSubcommand::Info(UnitCommand),
            }))
        );
        assert_eq!(
            list.command,
            Some(FrontendCommand::Work(WorkCommand {
                path: None,
                command: WorkSubcommand::List(UnitCommand),
            }))
        );
        assert_eq!(
            deps.command,
            Some(FrontendCommand::Work(WorkCommand {
                path: None,
                command: WorkSubcommand::Deps(UnitCommand),
            }))
        );
        assert_eq!(
            status.command,
            Some(FrontendCommand::Work(WorkCommand {
                path: None,
                command: WorkSubcommand::Status(UnitCommand),
            }))
        );
    }

    #[test]
    fn workspace_flags_parse_for_init_and_new_commands() {
        let init = FrontendCli::parse_from(["fol", "init", "--workspace"]);
        let new = FrontendCli::parse_from(["fol", "new", "demo", "--workspace"]);

        assert_eq!(
            init.command,
            Some(FrontendCommand::Init(InitCommand { workspace: true, bin: false, lib: false }))
        );
        assert_eq!(
            new.command,
            Some(FrontendCommand::New(NewCommand {
                name: "demo".to_string(),
                workspace: true,
                bin: false,
                lib: false,
            }))
        );
    }

    #[test]
    fn bin_flags_parse_for_init_and_new_commands() {
        let init = FrontendCli::parse_from(["fol", "init", "--bin"]);
        let new = FrontendCli::parse_from(["fol", "new", "demo", "--bin"]);

        assert_eq!(
            init.command,
            Some(FrontendCommand::Init(InitCommand { workspace: false, bin: true, lib: false }))
        );
        assert_eq!(
            new.command,
            Some(FrontendCommand::New(NewCommand {
                name: "demo".to_string(),
                workspace: false,
                bin: true,
                lib: false,
            }))
        );
    }

    #[test]
    fn duplicate_lib_flags_parse_for_init_and_new_commands() {
        let init = FrontendCli::parse_from(["fol", "init", "--lib"]);
        let new = FrontendCli::parse_from(["fol", "new", "demo", "--lib"]);

        assert_eq!(
            init.command,
            Some(FrontendCommand::Init(InitCommand { workspace: false, bin: false, lib: true }))
        );
        assert_eq!(
            new.command,
            Some(FrontendCommand::New(NewCommand {
                name: "demo".to_string(),
                workspace: false,
                bin: false,
                lib: true,
            }))
        );
    }

    #[test]
    fn lib_flags_parse_for_init_and_new_commands() {
        let init = FrontendCli::parse_from(["fol", "init", "--lib"]);
        let new = FrontendCli::parse_from(["fol", "new", "demo", "--lib"]);

        assert_eq!(
            init.command,
            Some(FrontendCommand::Init(InitCommand { workspace: false, bin: false, lib: true }))
        );
        assert_eq!(
            new.command,
            Some(FrontendCommand::New(NewCommand {
                name: "demo".to_string(),
                workspace: false,
                bin: false,
                lib: true,
            }))
        );
    }

    #[test]
    fn build_command_owns_direct_compile_flags() {
        let cli = FrontendCli::parse_from([
            "fol",
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
            Some(FrontendCommand::Build(BuildCommand {
                target: DirectTargetArg {
                    input: Some("demo".to_string()),
                },
                roots: CompileRootArgs {
                    std_root: Some("/tmp/std".to_string()),
                    package_store_root: Some("/tmp/pkg".to_string()),
                },
                locked: false,
                keep_build_dir: true,
            }))
        );
    }

    #[test]
    fn fetch_and_locked_workflow_flags_parse_on_commands() {
        let fetch = FrontendCli::parse_from(["fol", "fetch", "--locked", "--offline", "--refresh"]);
        let build = FrontendCli::parse_from(["fol", "build", "--locked"]);
        let run = FrontendCli::parse_from(["fol", "run", "--locked"]);
        let test = FrontendCli::parse_from(["fol", "test", "--locked"]);
        let check = FrontendCli::parse_from(["fol", "check", "--locked"]);

        assert_eq!(
            fetch.command,
            Some(FrontendCommand::Fetch(FetchCommand {
                roots: CompileRootArgs::default(),
                locked: true,
                offline: true,
                refresh: true,
            }))
        );
        assert_eq!(
            build.command,
            Some(FrontendCommand::Build(BuildCommand {
                target: DirectTargetArg::default(),
                roots: CompileRootArgs::default(),
                locked: true,
                keep_build_dir: false,
            }))
        );
        assert_eq!(
            run.command,
            Some(FrontendCommand::Run(RunCommand {
                target: DirectTargetArg::default(),
                roots: CompileRootArgs::default(),
                locked: true,
                keep_build_dir: false,
                args: Vec::new(),
            }))
        );
        assert_eq!(
            test.command,
            Some(FrontendCommand::Test(TestCommand {
                path: None,
                locked: true,
            }))
        );
        assert_eq!(
            check.command,
            Some(FrontendCommand::Check(CheckCommand {
                target: DirectTargetArg::default(),
                roots: CompileRootArgs::default(),
                locked: true,
            }))
        );
    }

    #[test]
    fn emit_subcommands_own_their_specific_flags() {
        let rust = FrontendCli::parse_from([
            "fol",
            "emit",
            "rust",
            "--keep-build-dir",
            "demo",
        ]);
        let lowered = FrontendCli::parse_from([
            "fol",
            "emit",
            "lowered",
            "--std-root",
            "/tmp/std",
            "demo",
        ]);

        assert_eq!(
            rust.command,
            Some(FrontendCommand::Emit(EmitCommand {
                command: EmitSubcommand::Rust(EmitRustCommand {
                    target: DirectTargetArg {
                        input: Some("demo".to_string()),
                    },
                    roots: CompileRootArgs::default(),
                    keep_build_dir: true,
                }),
            }))
        );
        assert_eq!(
            lowered.command,
            Some(FrontendCommand::Emit(EmitCommand {
                command: EmitSubcommand::Lowered(EmitLoweredCommand {
                    target: DirectTargetArg {
                        input: Some("demo".to_string()),
                    },
                    roots: CompileRootArgs {
                        std_root: Some("/tmp/std".to_string()),
                        package_store_root: None,
                    },
                }),
            }))
        );
    }
}
