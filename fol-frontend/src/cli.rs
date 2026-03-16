use clap::{Args, Parser, Subcommand};

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum FrontendCommand {
    Init(UnitCommand),
    New(UnitCommand),
    Work(UnitCommand),
    Fetch(UnitCommand),
    Build(UnitCommand),
    Run(UnitCommand),
    Test(UnitCommand),
    Check(UnitCommand),
    Emit(UnitCommand),
    Clean(UnitCommand),
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
}
