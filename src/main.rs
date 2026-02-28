// FOL Compiler entrypoint.
//
// Core compiler functionality is implemented in workspace crates:
// - fol-stream
// - fol-lexer
// - fol-parser
// - fol-diagnostics
// - fol-types
use clap::{Arg, Command};
use fol_diagnostics::{DiagnosticLocation, DiagnosticReport, OutputFormat};
use fol_parser::ast::AstParser;
use fol_stream::FileStream;
use std::path::Path;

fn main() {
    let matches = Command::new("fol")
        .version(env!("CARGO_PKG_VERSION"))
        .about("FOL Programming Language Compiler")
        .arg(
            Arg::new("file")
                .help(
                    "Input FOL file or folder to compile (.mod directories are handled specially)",
                )
                .value_name("FILE_OR_FOLDER")
                .index(1),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Output diagnostics in JSON format")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let file_path = matches
        .get_one::<String>("file")
        .map(|s| s.as_str())
        .unwrap_or("./test/main/main.fol");

    let json_output = matches.get_flag("json");
    let output_format = if json_output {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    };

    // Initialize diagnostic report
    let mut diagnostics = DiagnosticReport::new();

    if !json_output {
        println!("=== FOL Compiler (Modular) ===");
        println!("Compiling: {}", file_path);
    }

    // Try to compile the file
    match compile_file(file_path, &mut diagnostics) {
        Ok(_) => {
            if !json_output && !diagnostics.has_errors() {
                println!("✓ Compilation successful!");
            }
        }
        Err(_) => {
            // Errors are already added to diagnostics
        }
    }

    // Output diagnostics
    let output = diagnostics.output(output_format);
    if !output.trim().is_empty() {
        println!("{}", output);
    }

    // Exit with error code if there were compilation errors
    if diagnostics.has_errors() {
        std::process::exit(1);
    }
}

fn compile_file(file_path: &str, diagnostics: &mut DiagnosticReport) -> Result<(), ()> {
    // Check if file exists
    let path = Path::new(file_path);
    if !path.exists() {
        let error = fol_types::BasicError {
            message: format!("File not found: {}", file_path),
        };
        diagnostics.add_error(&error, None);
        return Err(());
    }

    // 1. Create stream (supports both files and folders with .mod handling)
    let mut file_stream = if path.is_dir() {
        match FileStream::from_folder(file_path) {
            Ok(stream) => stream,
            Err(e) => {
                report_input_error(diagnostics, e.as_ref(), file_path);
                return Err(());
            }
        }
    } else {
        match FileStream::from_file(file_path) {
            Ok(stream) => stream,
            Err(e) => {
                report_input_error(diagnostics, e.as_ref(), file_path);
                return Err(());
            }
        }
    };

    // 2. Lexical Analysis
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut file_stream);

    // 3. Parse with new AST parser
    let mut ast_parser = AstParser::new();
    match ast_parser.parse(&mut lexer) {
        Ok(_ast) => {
            // Successfully parsed AST
            if !diagnostics.has_errors() {
                // Could add semantic analysis, type checking, etc. here
                return Ok(());
            }
        }
        Err(parse_errors) => {
            // Add parse errors to diagnostics
            for error in parse_errors {
                diagnostics.add_error(error.as_ref(), None);
            }
            return Err(());
        }
    }

    Ok(())
}

fn report_input_error(
    diagnostics: &mut DiagnosticReport,
    error: &dyn fol_types::Glitch,
    file: &str,
) {
    diagnostics.add_error(
        error,
        Some(DiagnosticLocation {
            file: Some(file.to_string()),
            line: 1,
            column: 1,
            length: None,
        }),
    );
}

#[test]
fn compile_missing_file_reports_error() {
    let mut diagnostics = DiagnosticReport::new();
    let result = compile_file("./test/does-not-exist.fol", &mut diagnostics);

    assert!(result.is_err(), "Missing file should fail compilation");
    assert!(
        diagnostics.has_errors(),
        "Missing file should emit diagnostics"
    );
}
