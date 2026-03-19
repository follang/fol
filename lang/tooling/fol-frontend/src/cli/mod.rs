pub mod args;
pub mod parser;

#[cfg(test)]
mod tests;

pub use args::{
    BuildCommand, BuildOptionArgs, BuildStepArgs, CheckCommand, CodeCommand, CodeSubcommand,
    CompileRootArgs, CompleteCommand, CompletionCommand, CompletionShellArg,
    EditorPathCommand, EmitCommand, EmitLoweredCommand, EmitRustCommand, EmitSubcommand,
    FetchCommand, FrontendCommand, FrontendProfile,
    InitCommand, NewCommand, PackCommand, PackSubcommand, RunCommand, TestCommand, ToolCommand,
    ToolSubcommand, TreeSubcommand, UnitCommand, UpdateCommand,
    WorkSubcommand,
};
pub use parser::FrontendCli;
