use clap::{Args, CommandFactory, Parser, Subcommand};

const AFTER_HELP: &str = "\
Workflow Commands:
  init, new, fetch, check, build, run, test, emit, clean

Workspace Commands:
  work

Shell Commands:
  completion
";

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum FrontendCommand {
    #[command(visible_aliases = ["i"])]
    Init(UnitCommand),
    #[command(visible_aliases = ["n"])]
    New(UnitCommand),
    #[command(visible_aliases = ["w", "ws", "workspace"])]
    Work(UnitCommand),
    Fetch(UnitCommand),
    #[command(visible_aliases = ["b", "make"])]
    Build(UnitCommand),
    #[command(visible_aliases = ["r"])]
    Run(UnitCommand),
    #[command(visible_aliases = ["t"])]
    Test(UnitCommand),
    #[command(visible_aliases = ["c", "verify"])]
    Check(UnitCommand),
    Emit(UnitCommand),
    Clean(UnitCommand),
    #[command(visible_aliases = ["completions", "comp"])]
    Completion(UnitCommand),
    #[command(hide = true, name = "_complete")]
    Complete(UnitCommand),
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct UnitCommand;

#[derive(Debug, Clone, Parser, PartialEq, Eq)]
#[command(
    name = "fol",
    version,
    about = "User-facing frontend for the FOL toolchain",
    disable_help_subcommand = true
)]
pub struct FrontendCli {
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
}

#[cfg(test)]
mod tests {
    use super::{FrontendCli, FrontendCommand, UnitCommand};

    #[test]
    fn derive_root_parser_accepts_empty_invocation() {
        let cli = FrontendCli::parse_from(["fol"]);

        assert_eq!(cli.command, None);
    }

    #[test]
    fn root_command_families_parse_through_derive_tree() {
        let cli = FrontendCli::parse_from(["fol", "build"]);

        assert_eq!(cli.command, Some(FrontendCommand::Build(UnitCommand)));
    }

    #[test]
    fn visible_aliases_parse_to_the_same_root_commands() {
        let build = FrontendCli::parse_from(["fol", "b"]);
        let check = FrontendCli::parse_from(["fol", "verify"]);
        let work = FrontendCli::parse_from(["fol", "workspace"]);

        assert_eq!(build.command, Some(FrontendCommand::Build(UnitCommand)));
        assert_eq!(check.command, Some(FrontendCommand::Check(UnitCommand)));
        assert_eq!(work.command, Some(FrontendCommand::Work(UnitCommand)));
    }

    #[test]
    fn help_output_groups_commands_by_workflow_sections() {
        let help = FrontendCli::command().render_long_help().to_string();

        assert!(help.contains("Workflow Commands:"));
        assert!(help.contains("Workspace Commands:"));
        assert!(help.contains("Shell Commands:"));
    }
}
