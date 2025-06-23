// Legacy modules for now (will migrate to new structure)
#[macro_use]
mod types;
mod syntax;
mod helper;
mod semantic;

use clap::{Arg, Command};
use fol_diagnostics::{DiagnosticReport, OutputFormat, DiagnosticLocation};
use crate::syntax::index;
use crate::syntax::lexer;
use crate::syntax::parse;

fn main() {
    let matches = Command::new("fol")
        .version("0.1.4")
        .about("FOL Programming Language Compiler")
        .arg(
            Arg::new("file")
                .help("Input FOL file to compile")
                .value_name("FILE")
                .index(1)
        )
        .arg(
            Arg::new("json")
                .long("json")
                .help("Output diagnostics in JSON format")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let file_path = matches.get_one::<String>("file")
        .map(|s| s.as_str())
        .unwrap_or("./test/main/main.fol");
    
    let json_output = matches.get_flag("json");
    let output_format = if json_output { OutputFormat::Json } else { OutputFormat::Human };

    // Initialize diagnostic report
    let mut diagnostics = DiagnosticReport::new();

    if !json_output {
        println!("=== FOL Compiler ===");
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
    if !std::path::Path::new(file_path).exists() {
        let error = fol_types::BasicError {
            message: format!("File not found: {}", file_path)
        };
        diagnostics.add_error(&error, None);
        return Err(());
    }

    // 1. Lexical Analysis
    let input = index::Input::Path(file_path.to_string(), index::SourceType::File);
    let mut elems = match std::panic::catch_unwind(|| {
        lexer::stage3::Elements::init(&input)
    }) {
        Ok(elems) => elems,
        Err(_) => {
            let error = fol_types::BasicError {
                message: "Failed to initialize lexer".to_string()
            };
            let location = DiagnosticLocation {
                file: Some(file_path.to_string()),
                line: 1,
                column: 1,
                length: None,
            };
            diagnostics.add_error(&error, Some(location));
            return Err(());
        }
    };

    // 2. Parsing
    let parser = parse::Parser::init(&mut elems);
    let mut node_count = 0;
    
    for node in parser.nodes() {
        node_count += 1;
        // Successfully parsed a node - could add to AST here
        // Note: parser.nodes() returns ID<Box<dyn NodeTrait>>, not Result
    }

    if node_count == 0 {
        let error = fol_types::BasicError {
            message: "No valid syntax nodes found in file".to_string()
        };
        diagnostics.add_error(&error, None);
        return Err(());
    }

    Ok(())
}

#[test]
fn it_works() {
    assert_eq!("0", "0")
}
