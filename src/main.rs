// FOL Compiler entrypoint.
//
// Core compiler functionality is implemented in workspace crates:
// - fol-stream
// - fol-lexer
// - fol-parser
// - fol-diagnostics
// - fol-types
mod compiler_diagnostics;

use clap::{Arg, Command};
use fol_diagnostics::{DiagnosticLocation, DiagnosticReport, OutputFormat};
use fol_intrinsics as _;
use fol_lower::{render_lowered_workspace, LoweredWorkspace, Lowerer};
use fol_package::{PackageConfig, PackageSession};
use fol_parser::ast::AstParser;
use fol_stream::FileStream;
use fol_typecheck::Typechecker;
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
        .arg(
            Arg::new("dump-lowered")
                .long("dump-lowered")
                .help("Print a deterministic lowered-workspace snapshot after a successful compile")
                .action(clap::ArgAction::SetTrue),
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
    let dump_lowered = matches.get_flag("dump-lowered");
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
        Ok(lowered) => {
            if dump_lowered && !json_output && !diagnostics.has_errors() {
                println!("{}", render_lowered_workspace(&lowered));
            }
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
) -> Result<LoweredWorkspace, ()> {
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
                    compiler_diagnostics::add_compiler_glitch(diagnostics, &error);
                    return Err(());
                }
            };
            match fol_resolver::resolve_prepared_workspace_with_config(
                prepared,
                resolver_config.clone(),
            ) {
                Ok(resolved) => match Typechecker::new().check_resolved_workspace(resolved) {
                    Ok(typed) => match Lowerer::new().lower_typed_workspace(typed) {
                        Ok(lowered) => {
                            if !diagnostics.has_errors() {
                                return Ok(lowered);
                            }
                        }
                        Err(lowering_errors) => {
                            for error in lowering_errors {
                                compiler_diagnostics::add_compiler_glitch(diagnostics, &error);
                            }
                            return Err(());
                        }
                    },
                    Err(typecheck_errors) => {
                        for error in typecheck_errors {
                            compiler_diagnostics::add_compiler_glitch(diagnostics, &error);
                        }
                        return Err(());
                    }
                },
                Err(resolve_errors) => {
                    for error in resolve_errors {
                        compiler_diagnostics::add_compiler_glitch(diagnostics, &error);
                    }
                    return Err(());
                }
            }
        }
        Err(parse_errors) => {
            for error in parse_errors {
                compiler_diagnostics::add_compiler_glitch(diagnostics, error.as_ref());
            }
            return Err(());
        }
    }

    Err(())
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
        "Successful compilation through lowering should not emit diagnostics",
    );
}

#[test]
fn compile_typecheck_failures_surface_diagnostics() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("fol_compile_typecheck_failure_{stamp}.fol"));
    fs::write(
        &path,
        "fun[] bad(): int = {\n    return true;\n}\n",
    )
    .expect("typecheck fixture should be writable");

    let mut diagnostics = DiagnosticReport::new();
    let result = compile_file(
        path.to_str()
            .expect("typecheck fixture path should be valid UTF-8"),
        &fol_resolver::ResolverConfig::default(),
        &mut diagnostics,
    );

    assert!(result.is_err(), "Typecheck failures should fail compilation");
    assert!(
        diagnostics.has_errors(),
        "Typecheck failures should emit diagnostics",
    );

    fs::remove_file(path).ok();
}

#[cfg(test)]
fn unique_compile_fixture_dir(prefix: &str) -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("fol_compile_{prefix}_{stamp}"))
}

#[test]
fn compile_folder_entry_with_loc_imports_succeeds_through_workspace_lowering() {
    use std::fs;

    let root = unique_compile_fixture_dir("loc_workspace");
    let shared_root = root.join("shared");
    let app_root = root.join("app");
    fs::create_dir_all(&shared_root).expect("loc shared fixture should be creatable");
    fs::create_dir_all(&app_root).expect("loc app fixture should be creatable");
    fs::write(shared_root.join("lib.fol"), "var[exp] answer: int = 42;\n")
        .expect("loc shared fixture should be writable");
    fs::write(
        app_root.join("main.fol"),
        "use shared: loc = {\"../shared\"};\nfun[] main(): int = {\n    return answer;\n}\n",
    )
    .expect("loc app fixture should be writable");

    let mut diagnostics = DiagnosticReport::new();
    let result = compile_file(
        app_root.to_str().expect("loc app fixture path should be UTF-8"),
        &fol_resolver::ResolverConfig::default(),
        &mut diagnostics,
    );

    assert!(result.is_ok(), "folder entry packages should compile through loc imports and lowering");
    assert!(
        !diagnostics.has_errors(),
        "successful loc-import lowering should not emit diagnostics",
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn compile_folder_entry_with_std_imports_succeeds_through_workspace_lowering() {
    use std::fs;

    let root = unique_compile_fixture_dir("std_workspace");
    let std_root = root.join("std");
    let app_root = root.join("app");
    fs::create_dir_all(std_root.join("fmt")).expect("std fixture should be creatable");
    fs::create_dir_all(&app_root).expect("std app fixture should be creatable");
    fs::write(std_root.join("fmt/value.fol"), "var[exp] answer: int = 42;\n")
        .expect("std fixture should be writable");
    fs::write(
        app_root.join("main.fol"),
        "use fmt: std = {fmt};\nfun[] main(): int = {\n    return answer;\n}\n",
    )
    .expect("std app fixture should be writable");

    let mut diagnostics = DiagnosticReport::new();
    let result = compile_file(
        app_root.to_str().expect("std app fixture path should be UTF-8"),
        &fol_resolver::ResolverConfig {
            std_root: Some(
                std_root
                    .to_str()
                    .expect("std fixture path should be UTF-8")
                    .to_string(),
            ),
            package_store_root: None,
        },
        &mut diagnostics,
    );

    assert!(result.is_ok(), "folder entry packages should compile through std imports and lowering");
    assert!(
        !diagnostics.has_errors(),
        "successful std-import lowering should not emit diagnostics",
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn compile_folder_entry_with_pkg_imports_succeeds_through_workspace_lowering() {
    use std::fs;

    let root = unique_compile_fixture_dir("pkg_workspace");
    let store_root = root.join("store");
    let app_root = root.join("app");
    fs::create_dir_all(store_root.join("json/src")).expect("pkg fixture should be creatable");
    fs::create_dir_all(&app_root).expect("pkg app fixture should be creatable");
    fs::write(
        store_root.join("json/package.yaml"),
        "name: json\nversion: 1.0.0\n",
    )
    .expect("pkg metadata fixture should be writable");
    fs::write(store_root.join("json/build.fol"), "def root: loc = \"src\";\n")
        .expect("pkg build fixture should be writable");
    fs::write(store_root.join("json/src/lib.fol"), "var[exp] answer: int = 42;\n")
        .expect("pkg source fixture should be writable");
    fs::write(
        app_root.join("main.fol"),
        "use json: pkg = {json};\nfun[] main(): int = {\n    return answer;\n}\n",
    )
    .expect("pkg app fixture should be writable");

    let mut diagnostics = DiagnosticReport::new();
    let result = compile_file(
        app_root.to_str().expect("pkg app fixture path should be UTF-8"),
        &fol_resolver::ResolverConfig {
            std_root: None,
            package_store_root: Some(
                store_root
                    .to_str()
                    .expect("pkg fixture path should be UTF-8")
                    .to_string(),
            ),
        },
        &mut diagnostics,
    );

    assert!(result.is_ok(), "folder entry packages should compile through pkg imports and lowering");
    assert!(
        !diagnostics.has_errors(),
        "successful pkg-import lowering should not emit diagnostics",
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn intrinsics_crate_foundation_smoke_compiles() {
    assert_eq!(fol_intrinsics::crate_name(), "fol-intrinsics");
}

#[test]
fn backend_crate_foundation_smoke_compiles() {
    assert_eq!(fol_backend::crate_name(), "fol-backend");
}

#[test]
fn backend_public_api_shell_smoke_compiles() {
    let backend = fol_backend::Backend::new();
    assert_eq!(format!("{backend:?}"), "Backend");
}

#[test]
fn backend_error_surface_smoke_compiles() {
    let error = fol_backend::BackendError::new(
        fol_backend::BackendErrorKind::BuildFailure,
        "rust build failed",
    );

    assert_eq!(error.kind(), fol_backend::BackendErrorKind::BuildFailure);
    assert_eq!(error.message(), "rust build failed");
    assert_eq!(error.to_string(), "BackendBuildFailure: rust build failed");
}

#[test]
fn backend_config_surface_smoke_compiles() {
    let config = fol_backend::BackendConfig::default();

    assert_eq!(config.target, fol_backend::BackendTarget::Rust);
    assert_eq!(config.target.as_str(), "rust");
    assert_eq!(config.mode, fol_backend::BackendMode::BuildArtifact);
    assert_eq!(config.mode.as_str(), "build-artifact");
    assert!(!config.keep_build_dir);
}

#[test]
fn backend_artifact_surface_smoke_compiles() {
    let artifact = fol_backend::BackendArtifact::RustSourceCrate {
        root: "target/fol-backend/demo".to_string(),
        files: vec![fol_backend::EmittedRustFile {
            path: "src/main.rs".to_string(),
            module_name: "main".to_string(),
        }],
    };

    match artifact {
        fol_backend::BackendArtifact::RustSourceCrate { root, files } => {
            assert_eq!(root, "target/fol-backend/demo");
            assert_eq!(files.len(), 1);
            assert_eq!(files[0].module_name, "main");
        }
        fol_backend::BackendArtifact::CompiledBinary { .. } => {
            panic!("expected emitted rust crate artifact")
        }
    }
}

#[test]
fn intrinsics_public_model_smoke_compiles() {
    assert_eq!(fol_intrinsics::IntrinsicId::new(1).index(), 1);
    assert_eq!(
        fol_intrinsics::IntrinsicCategory::Comparison.as_str(),
        "comparison"
    );
    assert_eq!(
        fol_intrinsics::IntrinsicSurface::DotRootCall.as_str(),
        "dot-root-call"
    );
    assert_eq!(fol_intrinsics::IntrinsicAvailability::V1.as_str(), "V1");
    assert_eq!(
        fol_intrinsics::IntrinsicStatus::Implemented.as_str(),
        "implemented"
    );
}

#[test]
fn intrinsics_entry_model_smoke_compiles() {
    let entry = fol_intrinsics::IntrinsicEntry::new(
        fol_intrinsics::IntrinsicId::new(0),
        "eq",
        &["equal"],
        fol_intrinsics::IntrinsicCategory::Comparison,
        fol_intrinsics::IntrinsicSurface::DotRootCall,
        fol_intrinsics::IntrinsicAvailability::V1,
        fol_intrinsics::IntrinsicStatus::Implemented,
        fol_intrinsics::IntrinsicArity::Exactly(2),
        fol_intrinsics::IntrinsicLoweringMode::GeneralIr,
        "compare two values for equality",
    );

    assert_eq!(entry.name, "eq");
    assert_eq!(entry.aliases, &["equal"]);
    assert_eq!(entry.doc_summary, "compare two values for equality");
}

#[test]
fn intrinsics_canonical_registry_smoke_compiles() {
    let registry = fol_intrinsics::intrinsic_registry();
    assert!(registry.len() >= 8);
    assert!(registry.iter().any(|entry| entry.name == "eq"));
    assert!(registry.iter().any(|entry| entry.name == "de_alloc"));
    assert!(registry.iter().any(|entry| entry.name == "check"));
}

#[test]
fn intrinsics_lookup_api_smoke_compiles() {
    let eq = fol_intrinsics::intrinsic_by_canonical_name("eq").expect("eq should exist");
    let alias = fol_intrinsics::intrinsic_by_alias("ne").expect("alias should exist");
    let dot_calls = fol_intrinsics::intrinsics_for_surface(
        fol_intrinsics::IntrinsicSurface::DotRootCall,
    );

    assert_eq!(eq.name, "eq");
    assert_eq!(alias.name, "nq");
    assert!(dot_calls.iter().any(|entry| entry.name == "echo"));
}

#[test]
fn intrinsics_registry_validation_smoke_compiles() {
    assert!(fol_intrinsics::validate_intrinsic_registry(
        fol_intrinsics::intrinsic_registry()
    )
    .is_ok());

    let duplicate_names = [
        fol_intrinsics::IntrinsicEntry::new(
            fol_intrinsics::IntrinsicId::new(0),
            "eq",
            &[],
            fol_intrinsics::IntrinsicCategory::Comparison,
            fol_intrinsics::IntrinsicSurface::DotRootCall,
            fol_intrinsics::IntrinsicAvailability::V1,
            fol_intrinsics::IntrinsicStatus::Implemented,
            fol_intrinsics::IntrinsicArity::Exactly(2),
            fol_intrinsics::IntrinsicLoweringMode::GeneralIr,
            "compare values",
        ),
        fol_intrinsics::IntrinsicEntry::new(
            fol_intrinsics::IntrinsicId::new(1),
            "eq",
            &[],
            fol_intrinsics::IntrinsicCategory::Comparison,
            fol_intrinsics::IntrinsicSurface::DotRootCall,
            fol_intrinsics::IntrinsicAvailability::V1,
            fol_intrinsics::IntrinsicStatus::Implemented,
            fol_intrinsics::IntrinsicArity::Exactly(2),
            fol_intrinsics::IntrinsicLoweringMode::GeneralIr,
            "compare values again",
        ),
    ];
    assert!(matches!(
        fol_intrinsics::validate_intrinsic_registry(&duplicate_names),
        Err(fol_intrinsics::RegistryValidationError {
            kind: fol_intrinsics::RegistryValidationErrorKind::DuplicateCanonicalName,
            ..
        })
    ));

    let duplicate_alias = [
        fol_intrinsics::IntrinsicEntry::new(
            fol_intrinsics::IntrinsicId::new(0),
            "eq",
            &["cmp"],
            fol_intrinsics::IntrinsicCategory::Comparison,
            fol_intrinsics::IntrinsicSurface::DotRootCall,
            fol_intrinsics::IntrinsicAvailability::V1,
            fol_intrinsics::IntrinsicStatus::Implemented,
            fol_intrinsics::IntrinsicArity::Exactly(2),
            fol_intrinsics::IntrinsicLoweringMode::GeneralIr,
            "compare values",
        ),
        fol_intrinsics::IntrinsicEntry::new(
            fol_intrinsics::IntrinsicId::new(1),
            "gt",
            &["cmp"],
            fol_intrinsics::IntrinsicCategory::Comparison,
            fol_intrinsics::IntrinsicSurface::DotRootCall,
            fol_intrinsics::IntrinsicAvailability::V1,
            fol_intrinsics::IntrinsicStatus::Implemented,
            fol_intrinsics::IntrinsicArity::Exactly(2),
            fol_intrinsics::IntrinsicLoweringMode::GeneralIr,
            "compare values again",
        ),
    ];
    assert!(matches!(
        fol_intrinsics::validate_intrinsic_registry(&duplicate_alias),
        Err(fol_intrinsics::RegistryValidationError {
            kind: fol_intrinsics::RegistryValidationErrorKind::DuplicateAlias,
            offending_alias: "cmp",
            ..
        })
    ));
}

#[test]
fn intrinsics_selection_api_smoke_compiles() {
    let eq = fol_intrinsics::select_intrinsic(
        fol_intrinsics::IntrinsicSurface::DotRootCall,
        "eq",
    )
    .expect("eq should select");
    let wrong_surface = fol_intrinsics::select_intrinsic(
        fol_intrinsics::IntrinsicSurface::DotRootCall,
        "panic",
    )
    .expect_err("panic should stay keyword-only");

    assert_eq!(eq.name, "eq");
    assert_eq!(
        wrong_surface.kind,
        fol_intrinsics::IntrinsicSelectionErrorKind::WrongSurface
    );
}

#[test]
fn intrinsics_lowering_lookup_api_smoke_compiles() {
    let eq = fol_intrinsics::intrinsic_by_canonical_name("eq").expect("eq should exist");
    let runtime_hooks =
        fol_intrinsics::intrinsics_for_lowering_mode(fol_intrinsics::IntrinsicLoweringMode::RuntimeHook);

    assert_eq!(
        fol_intrinsics::lowering_mode_for_intrinsic(eq.id),
        Some(fol_intrinsics::IntrinsicLoweringMode::GeneralIr)
    );
    assert_eq!(
        fol_intrinsics::intrinsic_by_id(eq.id).map(|entry| entry.name),
        Some("eq")
    );
    assert!(runtime_hooks.iter().any(|entry| entry.name == "echo"));
}

#[test]
fn intrinsics_diagnostic_helpers_smoke_compiles() {
    let eq = fol_intrinsics::intrinsic_by_canonical_name("eq").expect("eq should exist");
    let deferred =
        fol_intrinsics::intrinsic_by_canonical_name("de_alloc").expect("de_alloc should exist");

    assert_eq!(
        fol_intrinsics::wrong_arity_message(eq, 1),
        ".eq(...) expects exactly 2 argument(s) but got 1"
    );
    assert_eq!(
        fol_intrinsics::wrong_version_message(
            deferred,
            fol_intrinsics::IntrinsicAvailability::V1
        ),
        ".de_alloc(...) belongs to V3 but the current compiler milestone is V1"
    );
}

#[test]
fn intrinsics_comparison_registry_smoke_compiles() {
    let expected = [("eq", 0usize), ("nq", 1), ("lt", 2), ("gt", 3), ("ge", 4), ("le", 5)];

    for (name, id) in expected {
        let entry =
            fol_intrinsics::intrinsic_by_canonical_name(name).expect("comparison entry should exist");
        assert_eq!(entry.id.index(), id);
        assert_eq!(entry.category, fol_intrinsics::IntrinsicCategory::Comparison);
        assert_eq!(entry.surface, fol_intrinsics::IntrinsicSurface::DotRootCall);
    }

    assert_eq!(
        fol_intrinsics::intrinsic_by_alias("ne").map(|entry| entry.name),
        Some("nq")
    );

    let eq = fol_intrinsics::intrinsic_by_canonical_name("eq").expect("eq should exist");
    let lt = fol_intrinsics::intrinsic_by_canonical_name("lt").expect("lt should exist");

    assert_eq!(
        fol_intrinsics::comparison_operand_contract(eq),
        Some(fol_intrinsics::ComparisonOperandContract::EqualityScalar)
    );
    assert_eq!(
        fol_intrinsics::comparison_operand_contract(lt),
        Some(fol_intrinsics::ComparisonOperandContract::OrderedScalar)
    );
}
