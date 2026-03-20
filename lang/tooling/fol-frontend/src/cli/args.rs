use crate::OutputMode;
use clap::{Args, Subcommand};

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
    #[command(visible_alias = "w")]
    Work(WorkCommand),
    #[command(visible_alias = "p")]
    Pack(PackCommand),
    #[command(visible_alias = "c")]
    Code(CodeCommand),
    #[command(visible_alias = "t")]
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
pub struct BuildOptionArgs {
    #[arg(
        long = "target",
        value_name = "TRIPLE",
        help = "Override the build target triple"
    )]
    pub build_target: Option<String>,

    #[arg(
        long = "optimize",
        value_name = "MODE",
        help = "Override the build optimization mode"
    )]
    pub build_optimize: Option<String>,

    #[arg(
        long = "build-option",
        value_name = "NAME=VALUE",
        help = "Override a named build option",
        action = clap::ArgAction::Append
    )]
    pub build_options: Vec<String>,

    #[arg(
        short = 'D',
        value_name = "NAME=VALUE",
        help = "Override a build option (shorthand for --build-option)",
        action = clap::ArgAction::Append
    )]
    pub define: Vec<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct BuildStepArgs {
    #[arg(long, value_name = "NAME", help = "Select a named build step")]
    pub step: Option<String>,
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

    #[command(flatten)]
    pub options: BuildOptionArgs,

    #[command(flatten)]
    pub step: BuildStepArgs,

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

    #[command(flatten)]
    pub options: BuildOptionArgs,

    #[command(flatten)]
    pub step: BuildStepArgs,

    #[arg(long, help = "Require the existing fol.lock to match the manifest")]
    pub locked: bool,

    #[arg(long, help = "Keep the generated backend crate directory")]
    pub keep_build_dir: bool,

    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub struct TestCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[command(flatten)]
    pub profile: FrontendProfileArgs,

    #[command(flatten)]
    pub options: BuildOptionArgs,

    #[command(flatten)]
    pub step: BuildStepArgs,

    #[arg(
        long,
        value_name = "PATH",
        help = "Override the workspace or package root"
    )]
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

    #[command(flatten)]
    pub options: BuildOptionArgs,

    #[command(flatten)]
    pub step: BuildStepArgs,

    #[arg(long, help = "Require the existing fol.lock to match the manifest")]
    pub locked: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct WorkCommand {
    #[command(flatten)]
    pub output: FrontendOutputArgs,

    #[arg(
        long,
        value_name = "PATH",
        help = "Override the workspace or package root"
    )]
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
pub struct TreeCommand {
    #[command(subcommand)]
    pub command: TreeSubcommand,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct TreeGenerateCommand {
    pub path: String,
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

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct EditorPathCommand {
    pub path: String,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct EditorReferenceCommand {
    pub path: String,

    #[arg(long, value_name = "LINE")]
    pub line: u32,

    #[arg(long, value_name = "CHARACTER")]
    pub character: u32,

    #[arg(long, help = "Exclude the declaration location from the result set")]
    pub exclude_declaration: bool,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct EditorRenameCommand {
    pub path: String,

    #[arg(long, value_name = "LINE")]
    pub line: u32,

    #[arg(long, value_name = "CHARACTER")]
    pub character: u32,

    pub new_name: String,
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
    Lsp(UnitCommand),
    Format(EditorPathCommand),
    Parse(EditorPathCommand),
    Highlight(EditorPathCommand),
    Symbols(EditorPathCommand),
    References(EditorReferenceCommand),
    Rename(EditorRenameCommand),
    SemanticTokens(EditorPathCommand),
    Tree(TreeCommand),
    #[command(visible_aliases = ["cl", "purge"])]
    Clean(UnitCommand),
    #[command(visible_aliases = ["completions", "comp"])]
    Completion(CompletionCommand),
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum TreeSubcommand {
    Generate(TreeGenerateCommand),
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
