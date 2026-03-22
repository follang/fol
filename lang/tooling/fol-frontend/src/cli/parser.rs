use crate::OutputMode;

use super::args::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AFTER_HELP: &str = "Run `fol <group> <command> --help` for command-specific usage.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendCli {
    pub input: Option<String>,
    pub output: OutputMode,
    pub json: bool,
    pub profile: Option<FrontendProfile>,
    pub debug: bool,
    pub release: bool,
    pub std_root: Option<String>,
    pub package_store_root: Option<String>,
    pub dump_lowered: bool,
    pub emit_rust: bool,
    pub keep_build_dir: bool,
    pub command: Option<FrontendCommand>,
}

impl FrontendCli {
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

// ---------------------------------------------------------------------------
// Parse error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    Help(String),
    Version,
    InvalidInput(String),
    InvalidSubcommand(String),
    Conflict(String),
    MissingValue(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
}

impl ParseError {
    fn help(text: String) -> Self {
        Self { kind: ParseErrorKind::Help(text) }
    }
    fn version() -> Self {
        Self { kind: ParseErrorKind::Version }
    }
    fn invalid(msg: impl Into<String>) -> Self {
        Self { kind: ParseErrorKind::InvalidInput(msg.into()) }
    }
    fn invalid_subcommand(msg: impl Into<String>) -> Self {
        Self { kind: ParseErrorKind::InvalidSubcommand(msg.into()) }
    }
    fn conflict(msg: impl Into<String>) -> Self {
        Self { kind: ParseErrorKind::Conflict(msg.into()) }
    }
    fn missing(msg: impl Into<String>) -> Self {
        Self { kind: ParseErrorKind::MissingValue(msg.into()) }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ParseErrorKind::Help(text) => write!(f, "{text}"),
            ParseErrorKind::Version => write!(f, "fol-frontend {VERSION}"),
            ParseErrorKind::InvalidInput(msg)
            | ParseErrorKind::InvalidSubcommand(msg)
            | ParseErrorKind::Conflict(msg)
            | ParseErrorKind::MissingValue(msg) => write!(f, "{msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Arg cursor helper
// ---------------------------------------------------------------------------

struct ArgCursor {
    args: Vec<String>,
    pos: usize,
}

impl ArgCursor {
    fn new(args: Vec<String>) -> Self {
        Self { args, pos: 0 }
    }

    fn peek(&self) -> Option<&str> {
        self.args.get(self.pos).map(|s| s.as_str())
    }

    fn advance(&mut self) -> Option<&str> {
        let val = self.args.get(self.pos).map(|s| s.as_str());
        if val.is_some() {
            self.pos += 1;
        }
        val
    }

    fn remaining_as_vec(&self) -> Vec<String> {
        self.args[self.pos..].to_vec()
    }

    fn is_done(&self) -> bool {
        self.pos >= self.args.len()
    }

    /// Take the value for a --flag. Handles both `--flag value` and `--flag=value`.
    /// The `current` is the already-consumed token. If it contains '=', return the part after '='.
    /// Otherwise consume the next token.
    fn take_value(&mut self, current: &str, flag_name: &str) -> Result<String, ParseError> {
        if let Some(eq_pos) = current.find('=') {
            Ok(current[eq_pos + 1..].to_string())
        } else {
            self.advance()
                .map(|s| s.to_string())
                .ok_or_else(|| ParseError::missing(format!("--{flag_name} requires a value")))
        }
    }
}

// ---------------------------------------------------------------------------
// Public parse interface
// ---------------------------------------------------------------------------

impl FrontendCli {
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        match Self::try_parse_from(args) {
            Ok(cli) => cli,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(match e.kind {
                    ParseErrorKind::Help(_) | ParseErrorKind::Version => 0,
                    _ => 1,
                });
            }
        }
    }

    pub fn try_parse_from<I, T>(args: I) -> Result<Self, ParseError>
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let args: Vec<String> = args.into_iter().map(|a| a.into()).collect();
        // Skip argv[0] (the binary name)
        let args = if !args.is_empty() { args[1..].to_vec() } else { vec![] };
        parse_root(args)
    }
}

// ---------------------------------------------------------------------------
// Help text
// ---------------------------------------------------------------------------

fn root_help() -> String {
    format!(
        "\
User-facing frontend for the FOL toolchain

Usage: fol [COMMAND]

Commands:
  work  [aliases: w]  Workspace management
  pack  [aliases: p]  Package management
  code  [aliases: c]  Build, run, test, check
  tool  [aliases: t]  Editor tools, LSP, completion

Options:
  -h, --help     Print help
  -V, --version  Print version

{AFTER_HELP}"
    )
}

fn work_help() -> String {
    "\
Usage: fol work [OPTIONS] <COMMAND>

Commands:
  init    Initialize a new package or workspace
  new     Create a new project
  info    Show workspace info
  list    List workspace members
  deps    Show dependency tree
  status  Show workspace status

Options:
  --output <MODE>  Select output mode [human|plain|json]
  --path <PATH>    Override the workspace or package root
  -h, --help       Print help"
        .to_string()
}

fn pack_help() -> String {
    "\
Usage: fol pack [OPTIONS] <COMMAND>

Commands:
  fetch   [aliases: f, sync]     Fetch dependencies
  update  [aliases: u, upgrade]  Update dependencies

Options:
  --output <MODE>  Select output mode [human|plain|json]
  -h, --help       Print help"
        .to_string()
}

fn code_help() -> String {
    "\
Usage: fol code [OPTIONS] <COMMAND>

Commands:
  build  [aliases: b, make]     Build the project
  run    [aliases: r]           Build and run the project
  test   [aliases: t]           Run tests
  check  [aliases: c, verify]   Check without building
  emit   [aliases: e, gen]      Emit intermediate representations

Options:
  --output <MODE>    Select output mode [human|plain|json]
  --profile <PROF>   Select build profile [debug|release]
  --debug            Force the debug profile
  --release          Force the release profile
  -h, --help         Print help"
        .to_string()
}

fn tool_help() -> String {
    "\
Usage: fol tool [OPTIONS] <COMMAND>

Commands:
  lsp              Start the LSP server
  format           Format a source file
  parse            Parse and dump a source file
  highlight        Highlight a source file
  symbols          List symbols in a source file
  references       Find references to a symbol
  rename           Rename a symbol
  complete         Get completions at a position
  semantic-tokens  Get semantic tokens for a file
  tree             Tree-sitter commands
  clean            [aliases: cl, purge]  Clean build artifacts
  completion       [aliases: completions, comp]  Generate shell completion

Options:
  --output <MODE>  Select output mode [human|plain|json]
  -h, --help       Print help"
        .to_string()
}

fn emit_help() -> String {
    "\
Usage: fol code emit <COMMAND>

Commands:
  rust     Emit generated Rust code
  lowered  Emit lowered IR

Options:
  -h, --help  Print help"
        .to_string()
}

fn tree_help() -> String {
    "\
Usage: fol tool tree <COMMAND>

Commands:
  generate  Generate tree-sitter grammar

Options:
  -h, --help  Print help"
        .to_string()
}

// ---------------------------------------------------------------------------
// Root parser
// ---------------------------------------------------------------------------

fn parse_root(args: Vec<String>) -> Result<FrontendCli, ParseError> {
    let mut cursor = ArgCursor::new(args);

    let mut cli = FrontendCli {
        input: None,
        output: env_output_mode(),
        json: false,
        profile: env_profile(),
        debug: false,
        release: false,
        std_root: None,
        package_store_root: None,
        dump_lowered: false,
        emit_rust: false,
        keep_build_dir: false,
        command: None,
    };

    // Consume root-level flags and find the command
    while let Some(token) = cursor.peek() {
        if token == "--help" || token == "-h" {
            return Err(ParseError::help(root_help()));
        }
        if token == "--version" || token == "-V" {
            return Err(ParseError::version());
        }
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            parse_root_flag(&mut cli, &token, &mut cursor)?;
            continue;
        }
        // Not a flag — must be a command or direct input
        break;
    }

    if cursor.is_done() {
        return Ok(cli);
    }

    let token = cursor.peek().unwrap().to_string();
    match resolve_command_group(&token) {
        Some(group) => {
            cursor.advance();
            cli.command = Some(parse_command_group(group, &mut cursor)?);
        }
        None if token == "_complete" => {
            cursor.advance();
            cli.command = Some(FrontendCommand::Complete(CompleteCommand {
                tokens: cursor.remaining_as_vec(),
            }));
        }
        None => {
            // Treat as direct file/folder input
            cli.input = Some(cursor.advance().unwrap().to_string());
            // Consume any remaining root flags after input
            while let Some(token) = cursor.peek() {
                if token.starts_with("--") {
                    let token = cursor.advance().unwrap().to_string();
                    parse_root_flag(&mut cli, &token, &mut cursor)?;
                } else {
                    break;
                }
            }
        }
    }

    Ok(cli)
}

fn parse_root_flag(cli: &mut FrontendCli, token: &str, cursor: &mut ArgCursor) -> Result<(), ParseError> {
    let (key, _) = split_eq(token);
    match key {
        "--output" => cli.output = parse_output_mode(&cursor.take_value(token, "output")?)?,
        "--json" => cli.json = true,
        "--profile" => cli.profile = Some(parse_profile(&cursor.take_value(token, "profile")?)?),
        "--debug" => cli.debug = true,
        "--release" => cli.release = true,
        "--std-root" => cli.std_root = Some(cursor.take_value(token, "std-root")?),
        "--package-store-root" => cli.package_store_root = Some(cursor.take_value(token, "package-store-root")?),
        "--dump-lowered" => cli.dump_lowered = true,
        "--emit-rust" => cli.emit_rust = true,
        "--keep-build-dir" => cli.keep_build_dir = true,
        _ => return Err(ParseError::invalid(format!("unknown flag: {key}"))),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Command group parsing
// ---------------------------------------------------------------------------

enum CommandGroup {
    Work,
    Pack,
    Code,
    Tool,
}

fn resolve_command_group(token: &str) -> Option<CommandGroup> {
    match token {
        "work" | "w" => Some(CommandGroup::Work),
        "pack" | "p" => Some(CommandGroup::Pack),
        "code" | "c" => Some(CommandGroup::Code),
        "tool" | "t" => Some(CommandGroup::Tool),
        _ => None,
    }
}

fn parse_command_group(group: CommandGroup, cursor: &mut ArgCursor) -> Result<FrontendCommand, ParseError> {
    match group {
        CommandGroup::Work => parse_work_command(cursor),
        CommandGroup::Pack => parse_pack_command(cursor),
        CommandGroup::Code => parse_code_command(cursor),
        CommandGroup::Tool => parse_tool_command(cursor),
    }
}

// ---------------------------------------------------------------------------
// work
// ---------------------------------------------------------------------------

fn parse_work_command(cursor: &mut ArgCursor) -> Result<FrontendCommand, ParseError> {
    let mut output = FrontendOutputArgs::default();
    let mut path: Option<String> = None;

    // Consume group-level flags
    while let Some(token) = cursor.peek() {
        if token == "--help" || token == "-h" {
            return Err(ParseError::help(work_help()));
        }
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--output" => output.output = parse_output_mode(&cursor.take_value(&token, "output")?)?,
                "--path" => path = Some(cursor.take_value(&token, "path")?),
                _ => return Err(ParseError::invalid(format!("unknown flag for work: {key}"))),
            }
            continue;
        }
        break;
    }

    let sub = cursor.advance().ok_or_else(|| ParseError::help(work_help()))?;
    let subcommand = match sub {
        "init" => WorkSubcommand::Init(parse_init_command(cursor)?),
        "new" => WorkSubcommand::New(parse_new_command(cursor)?),
        "info" => WorkSubcommand::Info(UnitCommand),
        "list" => WorkSubcommand::List(UnitCommand),
        "deps" => WorkSubcommand::Deps(UnitCommand),
        "status" => WorkSubcommand::Status(UnitCommand),
        _ => return Err(ParseError::invalid_subcommand(format!("unknown work subcommand: {sub}"))),
    };

    Ok(FrontendCommand::Work(WorkCommand { output, path, command: subcommand }))
}

fn parse_init_command(cursor: &mut ArgCursor) -> Result<InitCommand, ParseError> {
    let mut cmd = InitCommand::default();
    while let Some(token) = cursor.peek() {
        match token {
            "--workspace" => { cursor.advance(); cmd.workspace = true; }
            "--bin" => { cursor.advance(); cmd.bin = true; }
            "--lib" => { cursor.advance(); cmd.lib = true; }
            "-h" | "--help" => return Err(ParseError::help("Usage: fol work init [--workspace] [--bin|--lib]".to_string())),
            _ => return Err(ParseError::invalid(format!("unknown flag for init: {token}"))),
        }
    }
    if cmd.bin && cmd.lib {
        return Err(ParseError::conflict("--bin and --lib cannot be used together"));
    }
    Ok(cmd)
}

fn parse_new_command(cursor: &mut ArgCursor) -> Result<NewCommand, ParseError> {
    let mut workspace = false;
    let mut bin = false;
    let mut lib = false;
    let mut name: Option<String> = None;

    while let Some(token) = cursor.peek() {
        match token {
            "--workspace" => { cursor.advance(); workspace = true; }
            "--bin" => { cursor.advance(); bin = true; }
            "--lib" => { cursor.advance(); lib = true; }
            "-h" | "--help" => return Err(ParseError::help("Usage: fol work new <NAME> [--workspace] [--bin|--lib]".to_string())),
            t if t.starts_with('-') => return Err(ParseError::invalid(format!("unknown flag for new: {t}"))),
            _ => { name = Some(cursor.advance().unwrap().to_string()); }
        }
    }
    if bin && lib {
        return Err(ParseError::conflict("--bin and --lib cannot be used together"));
    }
    let name = name.ok_or_else(|| ParseError::missing("work new requires a project name"))?;
    Ok(NewCommand { name, workspace, bin, lib })
}

// ---------------------------------------------------------------------------
// pack
// ---------------------------------------------------------------------------

fn parse_pack_command(cursor: &mut ArgCursor) -> Result<FrontendCommand, ParseError> {
    let mut output = FrontendOutputArgs::default();

    while let Some(token) = cursor.peek() {
        if token == "--help" || token == "-h" {
            return Err(ParseError::help(pack_help()));
        }
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--output" => output.output = parse_output_mode(&cursor.take_value(&token, "output")?)?,
                _ => return Err(ParseError::invalid(format!("unknown flag for pack: {key}"))),
            }
            continue;
        }
        break;
    }

    let sub = cursor.advance().ok_or_else(|| ParseError::help(pack_help()))?;
    let sub_output = env_output_args();
    let subcommand = match sub {
        "fetch" | "f" | "sync" => PackSubcommand::Fetch(parse_fetch_command(cursor, sub_output)?),
        "update" | "u" | "upgrade" => PackSubcommand::Update(parse_update_command(cursor, sub_output)?),
        _ => return Err(ParseError::invalid_subcommand(format!("unknown pack subcommand: {sub}"))),
    };

    Ok(FrontendCommand::Pack(PackCommand { output, command: subcommand }))
}

fn parse_fetch_command(cursor: &mut ArgCursor, output: FrontendOutputArgs) -> Result<FetchCommand, ParseError> {
    let mut cmd = FetchCommand { output, ..FetchCommand::default() };
    while let Some(token) = cursor.peek() {
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--output" => cmd.output.output = parse_output_mode(&cursor.take_value(&token, "output")?)?,
                "--std-root" => cmd.roots.std_root = Some(cursor.take_value(&token, "std-root")?),
                "--package-store-root" => cmd.roots.package_store_root = Some(cursor.take_value(&token, "package-store-root")?),
                "--locked" => cmd.locked = true,
                "--offline" => cmd.offline = true,
                "--refresh" => cmd.refresh = true,
                "-h" | "--help" => return Err(ParseError::help("Usage: fol pack fetch [--locked] [--offline] [--refresh]".to_string())),
                _ => return Err(ParseError::invalid(format!("unknown flag for fetch: {key}"))),
            }
        } else {
            break;
        }
    }
    Ok(cmd)
}

fn parse_update_command(cursor: &mut ArgCursor, output: FrontendOutputArgs) -> Result<UpdateCommand, ParseError> {
    let mut cmd = UpdateCommand { output, ..UpdateCommand::default() };
    while let Some(token) = cursor.peek() {
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--output" => cmd.output.output = parse_output_mode(&cursor.take_value(&token, "output")?)?,
                "--std-root" => cmd.roots.std_root = Some(cursor.take_value(&token, "std-root")?),
                "--package-store-root" => cmd.roots.package_store_root = Some(cursor.take_value(&token, "package-store-root")?),
                "-h" | "--help" => return Err(ParseError::help("Usage: fol pack update".to_string())),
                _ => return Err(ParseError::invalid(format!("unknown flag for update: {key}"))),
            }
        } else {
            break;
        }
    }
    Ok(cmd)
}

// ---------------------------------------------------------------------------
// code
// ---------------------------------------------------------------------------

fn parse_code_command(cursor: &mut ArgCursor) -> Result<FrontendCommand, ParseError> {
    let mut output = env_output_args();
    let mut profile_args = env_profile_args();

    while let Some(token) = cursor.peek() {
        if token == "--help" || token == "-h" {
            return Err(ParseError::help(code_help()));
        }
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--output" => output.output = parse_output_mode(&cursor.take_value(&token, "output")?)?,
                "--profile" => profile_args.profile = Some(parse_profile(&cursor.take_value(&token, "profile")?)?),
                "--debug" => profile_args.debug = true,
                "--release" => profile_args.release = true,
                _ => return Err(ParseError::invalid(format!("unknown flag for code: {key}"))),
            }
            continue;
        }
        break;
    }

    check_profile_conflicts(&profile_args)?;

    let sub = cursor.advance().ok_or_else(|| ParseError::help(code_help()))?;
    // Subcommands start with env defaults, not inheriting from group-level flags.
    let sub_output = env_output_args();
    let sub_profile = env_profile_args();
    let subcommand = match sub {
        "build" | "b" | "make" => CodeSubcommand::Build(parse_build_command(cursor, sub_output, sub_profile)?),
        "run" | "r" => CodeSubcommand::Run(parse_run_command(cursor, sub_output, sub_profile)?),
        "test" | "t" => CodeSubcommand::Test(parse_test_command(cursor, sub_output, sub_profile)?),
        "check" | "c" | "verify" => CodeSubcommand::Check(parse_check_command(cursor, sub_output, sub_profile)?),
        "emit" | "e" | "gen" => CodeSubcommand::Emit(parse_emit_command(cursor)?),
        _ => return Err(ParseError::invalid_subcommand(format!("unknown code subcommand: {sub}"))),
    };

    Ok(FrontendCommand::Code(CodeCommand { output, profile: profile_args, command: subcommand }))
}

fn parse_build_command(cursor: &mut ArgCursor, output: FrontendOutputArgs, profile: FrontendProfileArgs) -> Result<BuildCommand, ParseError> {
    let mut cmd = BuildCommand { output, profile, ..BuildCommand::default() };
    while let Some(token) = cursor.peek() {
        if token == "--" {
            break;
        }
        if token.starts_with("--") || token.starts_with("-D") {
            let token = cursor.advance().unwrap().to_string();
            parse_build_flag(&mut cmd.output, &mut cmd.profile, &mut cmd.roots, &mut cmd.options, &mut cmd.step, &mut cmd.locked, Some(&mut cmd.keep_build_dir), &token, cursor)?;
        } else {
            // Positional = direct target
            if cmd.target.input.is_none() {
                cmd.target.input = Some(cursor.advance().unwrap().to_string());
            } else {
                break;
            }
        }
    }
    Ok(cmd)
}

fn parse_run_command(cursor: &mut ArgCursor, output: FrontendOutputArgs, profile: FrontendProfileArgs) -> Result<RunCommand, ParseError> {
    let mut cmd = RunCommand { output, profile, ..RunCommand::default() };
    let mut hit_separator = false;
    while let Some(token) = cursor.peek() {
        if token == "--" {
            cursor.advance();
            hit_separator = true;
            break;
        }
        if token.starts_with("--") || token.starts_with("-D") {
            let token = cursor.advance().unwrap().to_string();
            parse_build_flag(&mut cmd.output, &mut cmd.profile, &mut cmd.roots, &mut cmd.options, &mut cmd.step, &mut cmd.locked, Some(&mut cmd.keep_build_dir), &token, cursor)?;
        } else {
            if cmd.target.input.is_none() {
                cmd.target.input = Some(cursor.advance().unwrap().to_string());
            } else {
                // Additional positional before -- goes to args
                cmd.args.push(cursor.advance().unwrap().to_string());
            }
        }
    }
    // Everything after -- goes to args
    if hit_separator {
        while let Some(token) = cursor.advance() {
            // After --, first token might become target.input if not set,
            // but clap's behavior with trailing_var_arg puts them all after the positional
            // Actually with clap, `fol code run -- --flag value` puts --flag into target.input
            // and value into args. Let's match that.
            if cmd.target.input.is_none() {
                cmd.target.input = Some(token.to_string());
            } else {
                cmd.args.push(token.to_string());
            }
        }
    }
    Ok(cmd)
}

fn parse_test_command(cursor: &mut ArgCursor, output: FrontendOutputArgs, profile: FrontendProfileArgs) -> Result<TestCommand, ParseError> {
    let mut cmd = TestCommand { output, profile, ..TestCommand::default() };
    while let Some(token) = cursor.peek() {
        if token.starts_with("--") || token.starts_with("-D") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--output" => cmd.output.output = parse_output_mode(&cursor.take_value(&token, "output")?)?,
                "--profile" => cmd.profile.profile = Some(parse_profile(&cursor.take_value(&token, "profile")?)?),
                "--debug" => cmd.profile.debug = true,
                "--release" => cmd.profile.release = true,
                "--path" => cmd.path = Some(cursor.take_value(&token, "path")?),
                "--locked" => cmd.locked = true,
                "--step" => cmd.step.step = Some(cursor.take_value(&token, "step")?),
                "--target" => cmd.options.build_target = Some(cursor.take_value(&token, "target")?),
                "--optimize" => cmd.options.build_optimize = Some(cursor.take_value(&token, "optimize")?),
                "--build-option" => cmd.options.build_options.push(cursor.take_value(&token, "build-option")?),
                _ if token.starts_with("-D") => {
                    let val = if token.len() > 2 { token[2..].to_string() } else { cursor.take_value(&token, "D")? };
                    cmd.options.define.push(val);
                }
                "-h" | "--help" => return Err(ParseError::help("Usage: fol code test [OPTIONS]".to_string())),
                _ => return Err(ParseError::invalid(format!("unknown flag for test: {key}"))),
            }
        } else {
            break;
        }
    }
    Ok(cmd)
}

fn parse_check_command(cursor: &mut ArgCursor, output: FrontendOutputArgs, profile: FrontendProfileArgs) -> Result<CheckCommand, ParseError> {
    let mut cmd = CheckCommand { output, profile, ..CheckCommand::default() };
    while let Some(token) = cursor.peek() {
        if token.starts_with("--") || token.starts_with("-D") {
            let token = cursor.advance().unwrap().to_string();
            parse_build_flag(&mut cmd.output, &mut cmd.profile, &mut cmd.roots, &mut cmd.options, &mut cmd.step, &mut cmd.locked, None, &token, cursor)?;
        } else {
            if cmd.target.input.is_none() {
                cmd.target.input = Some(cursor.advance().unwrap().to_string());
            } else {
                break;
            }
        }
    }
    Ok(cmd)
}

fn parse_emit_command(cursor: &mut ArgCursor) -> Result<EmitCommand, ParseError> {
    while let Some(token) = cursor.peek() {
        if token == "--help" || token == "-h" {
            return Err(ParseError::help(emit_help()));
        }
        break;
    }

    let sub = cursor.advance().ok_or_else(|| ParseError::help(emit_help()))?;
    let subcommand = match sub {
        "rust" => EmitSubcommand::Rust(parse_emit_rust_command(cursor)?),
        "lowered" => EmitSubcommand::Lowered(parse_emit_lowered_command(cursor)?),
        _ => return Err(ParseError::invalid_subcommand(format!("unknown emit subcommand: {sub}"))),
    };
    Ok(EmitCommand { command: subcommand })
}

fn parse_emit_rust_command(cursor: &mut ArgCursor) -> Result<EmitRustCommand, ParseError> {
    let mut cmd = EmitRustCommand::default();
    while let Some(token) = cursor.peek() {
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--output" => cmd.output.output = parse_output_mode(&cursor.take_value(&token, "output")?)?,
                "--profile" => cmd.profile.profile = Some(parse_profile(&cursor.take_value(&token, "profile")?)?),
                "--debug" => cmd.profile.debug = true,
                "--release" => cmd.profile.release = true,
                "--std-root" => cmd.roots.std_root = Some(cursor.take_value(&token, "std-root")?),
                "--package-store-root" => cmd.roots.package_store_root = Some(cursor.take_value(&token, "package-store-root")?),
                "--keep-build-dir" => cmd.keep_build_dir = true,
                "-h" | "--help" => return Err(ParseError::help("Usage: fol code emit rust [OPTIONS] [PATH]".to_string())),
                _ => return Err(ParseError::invalid(format!("unknown flag for emit rust: {key}"))),
            }
        } else {
            if cmd.target.input.is_none() {
                cmd.target.input = Some(cursor.advance().unwrap().to_string());
            } else {
                break;
            }
        }
    }
    Ok(cmd)
}

fn parse_emit_lowered_command(cursor: &mut ArgCursor) -> Result<EmitLoweredCommand, ParseError> {
    let mut cmd = EmitLoweredCommand::default();
    while let Some(token) = cursor.peek() {
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--output" => cmd.output.output = parse_output_mode(&cursor.take_value(&token, "output")?)?,
                "--profile" => cmd.profile.profile = Some(parse_profile(&cursor.take_value(&token, "profile")?)?),
                "--debug" => cmd.profile.debug = true,
                "--release" => cmd.profile.release = true,
                "--std-root" => cmd.roots.std_root = Some(cursor.take_value(&token, "std-root")?),
                "--package-store-root" => cmd.roots.package_store_root = Some(cursor.take_value(&token, "package-store-root")?),
                "-h" | "--help" => return Err(ParseError::help("Usage: fol code emit lowered [OPTIONS] [PATH]".to_string())),
                _ => return Err(ParseError::invalid(format!("unknown flag for emit lowered: {key}"))),
            }
        } else {
            if cmd.target.input.is_none() {
                cmd.target.input = Some(cursor.advance().unwrap().to_string());
            } else {
                break;
            }
        }
    }
    Ok(cmd)
}

// ---------------------------------------------------------------------------
// tool
// ---------------------------------------------------------------------------

fn parse_tool_command(cursor: &mut ArgCursor) -> Result<FrontendCommand, ParseError> {
    let mut output = FrontendOutputArgs::default();

    while let Some(token) = cursor.peek() {
        if token == "--help" || token == "-h" {
            return Err(ParseError::help(tool_help()));
        }
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--output" => output.output = parse_output_mode(&cursor.take_value(&token, "output")?)?,
                _ => return Err(ParseError::invalid(format!("unknown flag for tool: {key}"))),
            }
            continue;
        }
        break;
    }

    let sub = cursor.advance().ok_or_else(|| ParseError::help(tool_help()))?;
    let subcommand = match sub {
        "lsp" => ToolSubcommand::Lsp(UnitCommand),
        "format" => ToolSubcommand::Format(parse_editor_path_command(cursor)?),
        "parse" => ToolSubcommand::Parse(parse_editor_path_command(cursor)?),
        "highlight" => ToolSubcommand::Highlight(parse_editor_path_command(cursor)?),
        "symbols" => ToolSubcommand::Symbols(parse_editor_path_command(cursor)?),
        "references" => ToolSubcommand::References(parse_editor_reference_command(cursor)?),
        "rename" => ToolSubcommand::Rename(parse_editor_rename_command(cursor)?),
        "complete" => ToolSubcommand::Complete(parse_editor_completion_command(cursor)?),
        "semantic-tokens" => ToolSubcommand::SemanticTokens(parse_editor_path_command(cursor)?),
        "tree" => ToolSubcommand::Tree(parse_tree_command(cursor)?),
        "clean" | "cl" | "purge" => ToolSubcommand::Clean(UnitCommand),
        "completion" | "completions" | "comp" => ToolSubcommand::Completion(parse_completion_command(cursor)?),
        _ => return Err(ParseError::invalid_subcommand(format!("unknown tool subcommand: {sub}"))),
    };

    Ok(FrontendCommand::Tool(ToolCommand { output, command: subcommand }))
}

fn parse_editor_path_command(cursor: &mut ArgCursor) -> Result<EditorPathCommand, ParseError> {
    let path = cursor.advance()
        .ok_or_else(|| ParseError::missing("expected a file path"))?
        .to_string();
    Ok(EditorPathCommand { path })
}

fn parse_editor_reference_command(cursor: &mut ArgCursor) -> Result<EditorReferenceCommand, ParseError> {
    let mut path: Option<String> = None;
    let mut line: Option<u32> = None;
    let mut character: Option<u32> = None;
    let mut exclude_declaration = false;

    while let Some(token) = cursor.peek() {
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--line" => line = Some(parse_u32(&cursor.take_value(&token, "line")?, "line")?),
                "--character" => character = Some(parse_u32(&cursor.take_value(&token, "character")?, "character")?),
                "--exclude-declaration" => exclude_declaration = true,
                "-h" | "--help" => return Err(ParseError::help("Usage: fol tool references <PATH> --line <N> --character <N> [--exclude-declaration]".to_string())),
                _ => return Err(ParseError::invalid(format!("unknown flag for references: {key}"))),
            }
        } else {
            if path.is_none() {
                path = Some(cursor.advance().unwrap().to_string());
            } else {
                break;
            }
        }
    }

    Ok(EditorReferenceCommand {
        path: path.ok_or_else(|| ParseError::missing("references requires a file path"))?,
        line: line.ok_or_else(|| ParseError::missing("references requires --line"))?,
        character: character.ok_or_else(|| ParseError::missing("references requires --character"))?,
        exclude_declaration,
    })
}

fn parse_editor_completion_command(cursor: &mut ArgCursor) -> Result<EditorCompletionCommand, ParseError> {
    let mut path: Option<String> = None;
    let mut line: Option<u32> = None;
    let mut character: Option<u32> = None;

    while let Some(token) = cursor.peek() {
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--line" => line = Some(parse_u32(&cursor.take_value(&token, "line")?, "line")?),
                "--character" => character = Some(parse_u32(&cursor.take_value(&token, "character")?, "character")?),
                _ => return Err(ParseError::invalid(format!("unknown flag for complete: {key}"))),
            }
        } else {
            if path.is_none() {
                path = Some(cursor.advance().unwrap().to_string());
            } else {
                break;
            }
        }
    }

    Ok(EditorCompletionCommand {
        path: path.ok_or_else(|| ParseError::missing("complete requires a file path"))?,
        line: line.ok_or_else(|| ParseError::missing("complete requires --line"))?,
        character: character.ok_or_else(|| ParseError::missing("complete requires --character"))?,
    })
}

fn parse_editor_rename_command(cursor: &mut ArgCursor) -> Result<EditorRenameCommand, ParseError> {
    let mut path: Option<String> = None;
    let mut line: Option<u32> = None;
    let mut character: Option<u32> = None;
    let mut new_name: Option<String> = None;

    while let Some(token) = cursor.peek() {
        if token.starts_with("--") {
            let token = cursor.advance().unwrap().to_string();
            let (key, _) = split_eq(&token);
            match key {
                "--line" => line = Some(parse_u32(&cursor.take_value(&token, "line")?, "line")?),
                "--character" => character = Some(parse_u32(&cursor.take_value(&token, "character")?, "character")?),
                _ => return Err(ParseError::invalid(format!("unknown flag for rename: {key}"))),
            }
        } else {
            if path.is_none() {
                path = Some(cursor.advance().unwrap().to_string());
            } else if new_name.is_none() {
                new_name = Some(cursor.advance().unwrap().to_string());
            } else {
                break;
            }
        }
    }

    Ok(EditorRenameCommand {
        path: path.ok_or_else(|| ParseError::missing("rename requires a file path"))?,
        line: line.ok_or_else(|| ParseError::missing("rename requires --line"))?,
        character: character.ok_or_else(|| ParseError::missing("rename requires --character"))?,
        new_name: new_name.ok_or_else(|| ParseError::missing("rename requires a new name"))?,
    })
}

fn parse_tree_command(cursor: &mut ArgCursor) -> Result<TreeCommand, ParseError> {
    while let Some(token) = cursor.peek() {
        if token == "--help" || token == "-h" {
            return Err(ParseError::help(tree_help()));
        }
        break;
    }
    let sub = cursor.advance().ok_or_else(|| ParseError::help(tree_help()))?;
    match sub {
        "generate" => {
            let path = cursor.advance()
                .ok_or_else(|| ParseError::missing("tree generate requires a path"))?
                .to_string();
            Ok(TreeCommand { command: TreeSubcommand::Generate(TreeGenerateCommand { path }) })
        }
        _ => Err(ParseError::invalid_subcommand(format!("unknown tree subcommand: {sub}"))),
    }
}

fn parse_completion_command(cursor: &mut ArgCursor) -> Result<CompletionCommand, ParseError> {
    let shell = cursor.advance().ok_or_else(|| ParseError::missing("completion requires a shell (bash, zsh, fish)"))?;
    let shell = match shell {
        "bash" => CompletionShellArg::Bash,
        "zsh" => CompletionShellArg::Zsh,
        "fish" => CompletionShellArg::Fish,
        _ => return Err(ParseError::invalid(format!("unknown shell: {shell} (expected bash, zsh, fish)"))),
    };
    Ok(CompletionCommand { shell })
}

// ---------------------------------------------------------------------------
// Shared build flag parser
// ---------------------------------------------------------------------------

fn parse_build_flag(
    output: &mut FrontendOutputArgs,
    profile: &mut FrontendProfileArgs,
    roots: &mut CompileRootArgs,
    options: &mut BuildOptionArgs,
    step: &mut BuildStepArgs,
    locked: &mut bool,
    keep_build_dir: Option<&mut bool>,
    token: &str,
    cursor: &mut ArgCursor,
) -> Result<(), ParseError> {
    let (key, _) = split_eq(token);
    match key {
        "--output" => output.output = parse_output_mode(&cursor.take_value(token, "output")?)?,
        "--profile" => profile.profile = Some(parse_profile(&cursor.take_value(token, "profile")?)?),
        "--debug" => profile.debug = true,
        "--release" => profile.release = true,
        "--std-root" => roots.std_root = Some(cursor.take_value(token, "std-root")?),
        "--package-store-root" => roots.package_store_root = Some(cursor.take_value(token, "package-store-root")?),
        "--locked" => *locked = true,
        "--keep-build-dir" => {
            if let Some(k) = keep_build_dir {
                *k = true;
            }
        }
        "--step" => step.step = Some(cursor.take_value(token, "step")?),
        "--target" => options.build_target = Some(cursor.take_value(token, "target")?),
        "--optimize" => options.build_optimize = Some(cursor.take_value(token, "optimize")?),
        "--build-option" => options.build_options.push(cursor.take_value(token, "build-option")?),
        _ if token.starts_with("-D") => {
            let val = if token.len() > 2 { token[2..].to_string() } else { cursor.take_value(token, "D")? };
            options.define.push(val);
        }
        "-h" | "--help" => return Err(ParseError::help("Usage: fol code <command> [OPTIONS]".to_string())),
        _ => return Err(ParseError::invalid(format!("unknown flag: {key}"))),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn split_eq(token: &str) -> (&str, Option<&str>) {
    if let Some(pos) = token.find('=') {
        (&token[..pos], Some(&token[pos + 1..]))
    } else {
        (token, None)
    }
}

fn parse_output_mode(value: &str) -> Result<OutputMode, ParseError> {
    match value {
        "human" => Ok(OutputMode::Human),
        "plain" => Ok(OutputMode::Plain),
        "json" => Ok(OutputMode::Json),
        _ => Err(ParseError::invalid(format!("unknown output mode: {value} (expected human, plain, json)"))),
    }
}

fn parse_profile(value: &str) -> Result<FrontendProfile, ParseError> {
    match value {
        "debug" => Ok(FrontendProfile::Debug),
        "release" => Ok(FrontendProfile::Release),
        _ => Err(ParseError::invalid(format!("unknown profile: {value} (expected debug, release)"))),
    }
}

fn parse_u32(value: &str, name: &str) -> Result<u32, ParseError> {
    value.parse::<u32>().map_err(|_| ParseError::invalid(format!("--{name} must be a number, got: {value}")))
}

fn check_profile_conflicts(profile: &FrontendProfileArgs) -> Result<(), ParseError> {
    let count = [profile.debug, profile.release, profile.profile.is_some()]
        .iter()
        .filter(|&&v| v)
        .count();
    if count > 1 {
        return Err(ParseError::conflict("--debug, --release, and --profile are mutually exclusive"));
    }
    Ok(())
}

fn env_output_mode() -> OutputMode {
    match std::env::var("FOL_OUTPUT").ok().as_deref() {
        Some("plain") => OutputMode::Plain,
        Some("json") => OutputMode::Json,
        _ => OutputMode::Human,
    }
}

fn env_profile() -> Option<FrontendProfile> {
    match std::env::var("FOL_PROFILE").ok().as_deref() {
        Some("release") => Some(FrontendProfile::Release),
        Some("debug") => Some(FrontendProfile::Debug),
        _ => None,
    }
}

fn env_output_args() -> FrontendOutputArgs {
    FrontendOutputArgs { output: env_output_mode() }
}

fn env_profile_args() -> FrontendProfileArgs {
    FrontendProfileArgs {
        profile: env_profile(),
        ..FrontendProfileArgs::default()
    }
}

// ---------------------------------------------------------------------------
// Help rendering (for compatibility with old FrontendCli::command())
// ---------------------------------------------------------------------------

impl FrontendCli {
    /// Returns the root help text. Used by dispatch for `fol` with no args.
    pub fn root_help_text() -> String {
        root_help()
    }
}
