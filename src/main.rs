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
use fol_package::{PackageConfig, PackageSession, PackageError};
use fol_parser::ast::{AstParser, ParseError};
use fol_stream::FileStream;
use fol_types::Glitch;
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
        .arg(
            Arg::new("std-root")
                .long("std-root")
                .value_name("DIR")
                .help("Explicit standard-library root for std imports"),
        )
        .arg(
            Arg::new("package-store-root")
                .long("package-store-root")
                .value_name("DIR")
                .help("Explicit installed package-store root for pkg imports"),
        )
        .get_matches();

    let file_path = matches
        .get_one::<String>("file")
        .map(|s| s.as_str())
        .unwrap_or("./test/main/main.fol");

    let json_output = matches.get_flag("json");
    let resolver_config = fol_resolver::ResolverConfig {
        std_root: matches.get_one::<String>("std-root").cloned(),
        package_store_root: matches.get_one::<String>("package-store-root").cloned(),
    };
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
    match compile_file(file_path, &resolver_config, &mut diagnostics) {
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

fn compile_file(
    file_path: &str,
    resolver_config: &fol_resolver::ResolverConfig,
    diagnostics: &mut DiagnosticReport,
) -> Result<(), ()> {
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

    // 3. Parse the book-aligned package shape
    let mut ast_parser = AstParser::new();
    match ast_parser.parse_package(&mut lexer) {
        Ok(package) => {
            let package_session = PackageSession::with_config(package_config_from_resolver(
                resolver_config,
            ));
            let prepared = match package_session.prepare_entry_package(package) {
                Ok(prepared) => prepared,
                Err(error) => {
                    report_compiler_glitch(diagnostics, &error);
                    return Err(());
                }
            };
            match fol_resolver::resolve_prepared_package_with_config(
                prepared,
                resolver_config.clone(),
            ) {
                Ok(_) => {
                    if !diagnostics.has_errors() {
                        return Ok(());
                    }
                }
                Err(resolve_errors) => {
                    for error in resolve_errors {
                        report_compiler_glitch(diagnostics, &error);
                    }
                    return Err(());
                }
            }
        }
        Err(parse_errors) => {
            for error in parse_errors {
                report_compiler_glitch(diagnostics, error.as_ref());
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

fn report_compiler_glitch(diagnostics: &mut DiagnosticReport, error: &dyn Glitch) {
    if let Some(parse_error) = error.as_any().downcast_ref::<ParseError>() {
        diagnostics.add_from(parse_error);
        return;
    }
    if let Some(package_error) = error.as_any().downcast_ref::<PackageError>() {
        diagnostics.add_from(package_error);
        return;
    }
    if let Some(resolver_error) = error
        .as_any()
        .downcast_ref::<fol_resolver::ResolverError>()
    {
        diagnostics.add_from(resolver_error);
        return;
    }

    diagnostics.add_error(error, None);
}

fn package_config_from_resolver(resolver_config: &fol_resolver::ResolverConfig) -> PackageConfig {
    PackageConfig {
        std_root: resolver_config.std_root.clone(),
        package_store_root: resolver_config.package_store_root.clone(),
        package_cache_root: None,
    }
}

#[test]
fn compile_missing_file_reports_error() {
    let mut diagnostics = DiagnosticReport::new();
    let result = compile_file(
        "./test/does-not-exist.fol",
        &fol_resolver::ResolverConfig::default(),
        &mut diagnostics,
    );

    assert!(result.is_err(), "Missing file should fail compilation");
    assert!(
        diagnostics.has_errors(),
        "Missing file should emit diagnostics"
    );
}

#[test]
fn compile_simple_file_succeeds_through_package_preparation_boundary() {
    let mut diagnostics = DiagnosticReport::new();
    let result = compile_file(
        "./test/parser/simple_var.fol",
        &fol_resolver::ResolverConfig::default(),
        &mut diagnostics,
    );

    assert!(result.is_ok(), "Simple files should compile through prepared entry packages");
    assert!(
        !diagnostics.has_errors(),
        "Successful prepared-package compilation should not emit diagnostics",
    );
}
