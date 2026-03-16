use crate::{ColorPolicy, OutputMode};
use clap::{Args, CommandFactory, Parser, Subcommand};

const AFTER_HELP: &str = "\
Workflow Commands:
  init, new, fetch, check, build, run, test, emit, clean

Workspace Commands:
  work

Shell Commands:
  completion

Examples:
  fol init --bin
  fol new demo --lib
  fol fetch
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
    Fetch(UnitCommand),
    #[command(visible_aliases = ["b", "make"])]
    Build(UnitCommand),
    #[command(visible_aliases = ["r"])]
    Run(UnitCommand),
    #[command(visible_aliases = ["t"])]
    Test(UnitCommand),
    #[command(visible_aliases = ["c", "verify"])]
    Check(UnitCommand),
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

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct WorkCommand {
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
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum EmitSubcommand {
    Rust(UnitCommand),
    Lowered(UnitCommand),
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
    #[arg(long, global = true, env = "FOL_OUTPUT", value_enum, default_value_t = OutputMode::Human)]
    pub output: OutputMode,

    #[arg(long, global = true, env = "FOL_COLOR", value_enum, default_value_t = ColorPolicy::Auto)]
    pub color: ColorPolicy,

    #[arg(long, global = true, env = "FOL_PROFILE", value_enum)]
    pub profile: Option<FrontendProfile>,

    #[arg(long, global = true, conflicts_with_all = ["release", "profile"])]
    pub debug: bool,

    #[arg(long, global = true, conflicts_with_all = ["debug", "profile"])]
    pub release: bool,

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

    pub fn command() -> clap::Command {
        <Self as CommandFactory>::command().help_template(
            "\
{about-section}
Usage: {usage}

Commands:
{subcommands}

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
}

#[cfg(test)]
mod tests {
    use super::{
        CompletionCommand, CompletionShellArg, EmitCommand, EmitSubcommand, FrontendCli,
        FrontendCommand, FrontendProfile, InitCommand, NewCommand, UnitCommand, WorkCommand,
        WorkSubcommand, CompleteCommand,
    };
    use crate::{ColorPolicy, OutputMode};

    #[test]
    fn derive_root_parser_accepts_empty_invocation() {
        let cli = FrontendCli::parse_from(["fol"]);

        assert_eq!(cli.output, OutputMode::Human);
        assert_eq!(cli.color, ColorPolicy::Auto);
        assert_eq!(cli.selected_profile(), FrontendProfile::Debug);
        assert_eq!(cli.command, None);
    }

    #[test]
    fn root_command_families_parse_through_derive_tree() {
        let cli = FrontendCli::parse_from(["fol", "build"]);

        assert_eq!(cli.command, Some(FrontendCommand::Build(UnitCommand)));
    }

    #[test]
    fn emit_subcommands_parse_through_derive_tree() {
        let rust = FrontendCli::parse_from(["fol", "emit", "rust"]);
        let lowered = FrontendCli::parse_from(["fol", "emit", "lowered"]);

        assert_eq!(
            rust.command,
            Some(FrontendCommand::Emit(EmitCommand {
                command: EmitSubcommand::Rust(UnitCommand),
            }))
        );
        assert_eq!(
            lowered.command,
            Some(FrontendCommand::Emit(EmitCommand {
                command: EmitSubcommand::Lowered(UnitCommand),
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
        let emit = FrontendCli::parse_from(["fol", "gen", "rust"]);
        let clean = FrontendCli::parse_from(["fol", "purge"]);

        assert_eq!(build.command, Some(FrontendCommand::Build(UnitCommand)));
        assert_eq!(check.command, Some(FrontendCommand::Check(UnitCommand)));
        assert_eq!(fetch.command, Some(FrontendCommand::Fetch(UnitCommand)));
        assert_eq!(
            emit.command,
            Some(FrontendCommand::Emit(EmitCommand {
                command: EmitSubcommand::Rust(UnitCommand),
            }))
        );
        assert_eq!(clean.command, Some(FrontendCommand::Clean(UnitCommand)));
        assert_eq!(
            work.command,
            Some(FrontendCommand::Work(WorkCommand {
                command: WorkSubcommand::Info(UnitCommand),
            }))
        );
    }

    #[test]
    fn output_flag_parses_global_output_mode() {
        let cli = FrontendCli::parse_from(["fol", "--output", "json", "build"]);

        assert_eq!(cli.output, OutputMode::Json);
        assert_eq!(cli.command, Some(FrontendCommand::Build(UnitCommand)));
    }

    #[test]
    fn color_flag_parses_global_color_policy() {
        let cli = FrontendCli::parse_from(["fol", "--color", "never", "build"]);

        assert_eq!(cli.color, ColorPolicy::Never);
        assert_eq!(cli.command, Some(FrontendCommand::Build(UnitCommand)));
    }

    #[test]
    fn profile_flags_normalize_to_frontend_profile_selection() {
        let profile = FrontendCli::parse_from(["fol", "--profile", "release", "build"]);
        let release = FrontendCli::parse_from(["fol", "--release", "build"]);

        assert_eq!(profile.selected_profile(), FrontendProfile::Release);
        assert_eq!(release.selected_profile(), FrontendProfile::Release);
    }

    #[test]
    fn cli_env_values_feed_output_color_and_profile_defaults() {
        unsafe {
            std::env::set_var("FOL_OUTPUT", "plain");
            std::env::set_var("FOL_COLOR", "never");
            std::env::set_var("FOL_PROFILE", "release");
        }

        let cli = FrontendCli::parse_from(["fol", "build"]);

        assert_eq!(cli.output, OutputMode::Plain);
        assert_eq!(cli.color, ColorPolicy::Never);
        assert_eq!(cli.selected_profile(), FrontendProfile::Release);

        unsafe {
            std::env::remove_var("FOL_OUTPUT");
            std::env::remove_var("FOL_COLOR");
            std::env::remove_var("FOL_PROFILE");
        }
    }

    #[test]
    fn explicit_flags_override_env_values() {
        unsafe {
            std::env::set_var("FOL_OUTPUT", "plain");
            std::env::set_var("FOL_COLOR", "never");
            std::env::set_var("FOL_PROFILE", "release");
        }

        let cli = FrontendCli::parse_from([
            "fol",
            "--output",
            "json",
            "--color",
            "always",
            "--debug",
            "build",
        ]);

        assert_eq!(cli.output, OutputMode::Json);
        assert_eq!(cli.color, ColorPolicy::Always);
        assert_eq!(cli.selected_profile(), FrontendProfile::Debug);

        unsafe {
            std::env::remove_var("FOL_OUTPUT");
            std::env::remove_var("FOL_COLOR");
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
    }

    #[test]
    fn help_output_keeps_global_mode_flags_visible() {
        let help = FrontendCli::command().render_long_help().to_string();

        assert!(help.contains("--output"));
        assert!(help.contains("--color"));
        assert!(help.contains("--profile"));
        assert!(help.contains("--debug"));
        assert!(help.contains("--release"));
    }

    #[test]
    fn help_output_mentions_visible_aliases() {
        let help = FrontendCli::command().render_long_help().to_string();

        assert!(help.contains("build"));
        assert!(help.contains("make"));
        assert!(help.contains("check"));
        assert!(help.contains("verify"));
        assert!(help.contains("sync"));
        assert!(help.contains("purge"));
        assert!(help.contains("gen"));
        assert!(help.contains("completion"));
        assert!(help.contains("completions"));
    }

    #[test]
    fn work_subcommands_parse_for_info_and_list() {
        let info = FrontendCli::parse_from(["fol", "work", "info"]);
        let list = FrontendCli::parse_from(["fol", "work", "list"]);

        assert_eq!(
            info.command,
            Some(FrontendCommand::Work(WorkCommand {
                command: WorkSubcommand::Info(UnitCommand),
            }))
        );
        assert_eq!(
            list.command,
            Some(FrontendCommand::Work(WorkCommand {
                command: WorkSubcommand::List(UnitCommand),
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
}
