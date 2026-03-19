use crate::OutputMode;
use clap::{CommandFactory, Parser};

use super::args::FrontendProfile;

const AFTER_HELP: &str = "Run `fol <group> <command> --help` for command-specific usage.";

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
        help = "Input FOL file or folder to build directly",
        hide = true
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

    #[arg(long, hide = true, env = "FOL_PROFILE", value_enum)]
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
    pub command: Option<super::args::FrontendCommand>,
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
