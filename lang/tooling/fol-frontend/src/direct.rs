use crate::{
    FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendConfig,
    FrontendError, FrontendErrorKind, FrontendResult, OutputMode,
};
use fol_backend::{
    emit_backend_artifact, summarize_emitted_artifact, BackendConfig, BackendMode, BackendSession,
};
use fol_diagnostics::{DiagnosticLocation, DiagnosticReport, OutputFormat, ToDiagnostic};
use fol_lower::{render_lowered_workspace, LoweredWorkspace, Lowerer, LoweringError};
use fol_package::{PackageConfig, PackageError, PackageSession};
use fol_parser::ast::{AstParser, ParseError};
use fol_resolver::{ResolverConfig, ResolverError};
use fol_stream::FileStream;
use fol_typecheck::{TypecheckError, Typechecker};
use fol_types::{BasicError, Glitch};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectCompileConfig {
    pub input: String,
    pub std_root: Option<String>,
    pub package_store_root: Option<String>,
    pub mode: DirectCompileMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectCompileMode {
    Auto {
        dump_lowered: bool,
        emit_rust: bool,
        keep_build_dir: bool,
    },
    Check,
    Build {
        keep_build_dir: bool,
    },
    Run {
        keep_build_dir: bool,
        args: Vec<String>,
    },
    EmitRust {
        keep_build_dir: bool,
    },
    EmitLowered,
}

pub fn run_direct_compile(
    config: &DirectCompileConfig,
    frontend_config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    let mut diagnostics = DiagnosticReport::new();
    let lowered = compile_file(
        &config.input,
        &ResolverConfig {
            std_root: config.std_root.clone(),
            package_store_root: config.package_store_root.clone(),
        },
        &mut diagnostics,
    )
    .map_err(|()| {
        FrontendError::new(
            FrontendErrorKind::CommandFailed,
            render_direct_diagnostics(&diagnostics, frontend_config.output.mode),
        )
    })?;

    if diagnostics.has_errors() {
        return Err(FrontendError::new(
            FrontendErrorKind::CommandFailed,
            render_direct_diagnostics(&diagnostics, frontend_config.output.mode),
        ));
    }

    match &config.mode {
        DirectCompileMode::Auto {
            dump_lowered,
            emit_rust,
            keep_build_dir,
        } => {
            let mut result =
                FrontendCommandResult::new("compile", format!("compiled {}", config.input));
            if *dump_lowered {
                let lowered_root = frontend_config
                    .working_directory
                    .join("target")
                    .join("lowered");
                std::fs::create_dir_all(&lowered_root).map_err(|error| {
                    FrontendError::new(
                        FrontendErrorKind::CommandFailed,
                        format!(
                            "failed to create lowered output root '{}': {error}",
                            lowered_root.display()
                        ),
                    )
                })?;
                let stem = Path::new(&config.input)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("input");
                let snapshot_path = lowered_root.join(format!("{stem}.lowered.txt"));
                std::fs::write(&snapshot_path, render_lowered_workspace(&lowered)).map_err(
                    |error| {
                        FrontendError::new(
                            FrontendErrorKind::CommandFailed,
                            format!(
                                "failed to write lowered snapshot '{}': {error}",
                                snapshot_path.display()
                            ),
                        )
                    },
                )?;
                result.artifacts.push(FrontendArtifactSummary::new(
                    FrontendArtifactKind::LoweredSnapshot,
                    "lowered-snapshot",
                    Some(snapshot_path),
                ));
            }

            if lowered.entry_candidates().is_empty() {
                result.summary = format!("compiled {} without runnable entrypoint", config.input);
                return Ok(result);
            }

            let backend_session = BackendSession::new(lowered);
            let output_root = frontend_config.working_directory.join("target");
            let artifact = emit_backend_artifact(
                &backend_session,
                &BackendConfig {
                    mode: if *emit_rust {
                        BackendMode::EmitSource
                    } else {
                        BackendMode::BuildArtifact
                    },
                    keep_build_dir: *keep_build_dir,
                    ..BackendConfig::default()
                },
                &output_root,
            )
            .map_err(|error| {
                FrontendError::new(FrontendErrorKind::CommandFailed, error.to_string())
            })?;

            result.summary = summarize_emitted_artifact(&artifact);
            match artifact {
                fol_backend::BackendArtifact::RustSourceCrate { root, .. } => {
                    result.artifacts.push(FrontendArtifactSummary::new(
                        FrontendArtifactKind::EmittedRust,
                        "emitted-rust",
                        Some(PathBuf::from(root)),
                    ));
                }
                fol_backend::BackendArtifact::CompiledBinary {
                    crate_root,
                    binary_path,
                } => {
                    result.artifacts.push(FrontendArtifactSummary::new(
                        FrontendArtifactKind::EmittedRust,
                        "backend-crate",
                        Some(PathBuf::from(crate_root)),
                    ));
                    result.artifacts.push(FrontendArtifactSummary::new(
                        FrontendArtifactKind::Binary,
                        "binary",
                        Some(PathBuf::from(binary_path)),
                    ));
                }
            }
            Ok(result)
        }
        DirectCompileMode::Check => {
            let mut result =
                FrontendCommandResult::new("check", format!("checked {}", config.input));
            if lowered.entry_candidates().is_empty() {
                result.summary = format!("checked {} without runnable entrypoint", config.input);
            }
            Ok(result)
        }
        DirectCompileMode::EmitLowered => {
            let lowered_root = frontend_config
                .working_directory
                .join("target")
                .join("lowered");
            std::fs::create_dir_all(&lowered_root).map_err(|error| {
                FrontendError::new(
                    FrontendErrorKind::CommandFailed,
                    format!(
                        "failed to create lowered output root '{}': {error}",
                        lowered_root.display()
                    ),
                )
            })?;
            let stem = Path::new(&config.input)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("input");
            let snapshot_path = lowered_root.join(format!("{stem}.lowered.txt"));
            std::fs::write(&snapshot_path, render_lowered_workspace(&lowered)).map_err(
                |error| {
                    FrontendError::new(
                        FrontendErrorKind::CommandFailed,
                        format!(
                            "failed to write lowered snapshot '{}': {error}",
                            snapshot_path.display()
                        ),
                    )
                },
            )?;
            let mut result = FrontendCommandResult::new(
                "emit lowered",
                format!("emitted lowered snapshot for {}", config.input),
            );
            result.artifacts.push(FrontendArtifactSummary::new(
                FrontendArtifactKind::LoweredSnapshot,
                "lowered-snapshot",
                Some(snapshot_path),
            ));
            Ok(result)
        }
        DirectCompileMode::Build { keep_build_dir }
        | DirectCompileMode::Run { keep_build_dir, .. }
        | DirectCompileMode::EmitRust { keep_build_dir } => {
            if lowered.entry_candidates().is_empty() {
                return Err(FrontendError::new(
                    FrontendErrorKind::InvalidInput,
                    format!("{} does not contain a runnable entrypoint", config.input),
                ));
            }

            let backend_session = BackendSession::new(lowered);
            let output_root = frontend_config.working_directory.join("target");
            let backend_mode = match config.mode {
                DirectCompileMode::EmitRust { .. } => BackendMode::EmitSource,
                _ => BackendMode::BuildArtifact,
            };
            let artifact = emit_backend_artifact(
                &backend_session,
                &BackendConfig {
                    mode: backend_mode,
                    keep_build_dir: *keep_build_dir,
                    ..BackendConfig::default()
                },
                &output_root,
            )
            .map_err(|error| {
                FrontendError::new(FrontendErrorKind::CommandFailed, error.to_string())
            })?;

            match (&config.mode, artifact) {
                (
                    DirectCompileMode::EmitRust { .. },
                    fol_backend::BackendArtifact::RustSourceCrate { root, .. },
                ) => {
                    let mut result = FrontendCommandResult::new(
                        "emit rust",
                        format!("emitted Rust backend for {}", config.input),
                    );
                    result.artifacts.push(FrontendArtifactSummary::new(
                        FrontendArtifactKind::EmittedRust,
                        "emitted-rust",
                        Some(PathBuf::from(root)),
                    ));
                    Ok(result)
                }
                (
                    DirectCompileMode::Build { .. },
                    fol_backend::BackendArtifact::CompiledBinary {
                        crate_root,
                        binary_path,
                    },
                ) => {
                    let mut result = FrontendCommandResult::new(
                        "build",
                        summarize_emitted_artifact(&fol_backend::BackendArtifact::CompiledBinary {
                            crate_root: crate_root.clone(),
                            binary_path: binary_path.clone(),
                        }),
                    );
                    result.artifacts.push(FrontendArtifactSummary::new(
                        FrontendArtifactKind::EmittedRust,
                        "backend-crate",
                        Some(PathBuf::from(crate_root)),
                    ));
                    result.artifacts.push(FrontendArtifactSummary::new(
                        FrontendArtifactKind::Binary,
                        "binary",
                        Some(PathBuf::from(binary_path)),
                    ));
                    Ok(result)
                }
                (
                    DirectCompileMode::Run { args, .. },
                    fol_backend::BackendArtifact::CompiledBinary {
                        crate_root,
                        binary_path,
                    },
                ) => {
                    let output = std::process::Command::new(&binary_path)
                        .args(args)
                        .output()
                        .map_err(|error| {
                            FrontendError::new(FrontendErrorKind::CommandFailed, error.to_string())
                        })?;
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if !stderr.is_empty() {
                            eprint!("{stderr}");
                        }
                        return Err(FrontendError::new(
                            FrontendErrorKind::CommandFailed,
                            format!(
                                "run command failed for '{}': status {}",
                                binary_path, output.status
                            ),
                        ));
                    }
                    let mut result =
                        FrontendCommandResult::new("run", format!("ran {}", binary_path));
                    result.artifacts.push(FrontendArtifactSummary::new(
                        FrontendArtifactKind::EmittedRust,
                        "backend-crate",
                        Some(PathBuf::from(crate_root)),
                    ));
                    result.artifacts.push(FrontendArtifactSummary::new(
                        FrontendArtifactKind::Binary,
                        "binary",
                        Some(PathBuf::from(binary_path)),
                    ));
                    Ok(result)
                }
                _ => Err(FrontendError::new(
                    FrontendErrorKind::Internal,
                    "direct compile mode received an unexpected backend artifact",
                )),
            }
        }
    }
}

pub fn run_direct_compile_with_io(
    config: &DirectCompileConfig,
    frontend_config: &FrontendConfig,
    stdout: &mut impl std::io::Write,
) -> i32 {
    let output_format = match frontend_config.output.mode {
        OutputMode::Json => OutputFormat::Json,
        _ => OutputFormat::Human,
    };
    let resolver_config = ResolverConfig {
        std_root: config.std_root.clone(),
        package_store_root: config.package_store_root.clone(),
    };
    let mut diagnostics = DiagnosticReport::new();

    if frontend_config.output.mode != OutputMode::Json {
        let _ = writeln!(stdout, "=== FOL Compiler (Modular) ===");
        let _ = writeln!(stdout, "Compiling: {}", config.input);
    }

    match compile_file(&config.input, &resolver_config, &mut diagnostics) {
        Ok(lowered) => {
            if matches!(
                config.mode,
                DirectCompileMode::EmitLowered
                    | DirectCompileMode::Auto {
                        dump_lowered: true,
                        ..
                    }
            ) && frontend_config.output.mode != OutputMode::Json
                && !diagnostics.has_errors()
            {
                let _ = writeln!(stdout, "{}", render_lowered_workspace(&lowered));
            }
            if !diagnostics.has_errors() {
                if lowered.entry_candidates().is_empty()
                    && !matches!(
                        config.mode,
                        DirectCompileMode::Check | DirectCompileMode::EmitLowered
                    )
                {
                    if matches!(config.mode, DirectCompileMode::Auto { .. }) {
                        if frontend_config.output.mode != OutputMode::Json {
                            let _ = writeln!(stdout, "✓ Compilation successful!");
                        }
                    } else {
                        diagnostics.add_error(
                            &BasicError {
                                message: format!(
                                    "{} does not contain a runnable entrypoint",
                                    config.input
                                ),
                            },
                            None,
                        );
                    }
                } else if !matches!(
                    config.mode,
                    DirectCompileMode::Check | DirectCompileMode::EmitLowered
                ) {
                    let backend_session = BackendSession::new(lowered);
                    let output_root = frontend_config.working_directory.join("target");
                    match emit_backend_artifact(
                        &backend_session,
                        &BackendConfig {
                            mode: match config.mode {
                                DirectCompileMode::Auto {
                                    emit_rust: true, ..
                                } => BackendMode::EmitSource,
                                DirectCompileMode::EmitRust { .. } => BackendMode::EmitSource,
                                _ => BackendMode::BuildArtifact,
                            },
                            keep_build_dir: match &config.mode {
                                DirectCompileMode::Auto { keep_build_dir, .. } => *keep_build_dir,
                                DirectCompileMode::Build { keep_build_dir }
                                | DirectCompileMode::Run { keep_build_dir, .. }
                                | DirectCompileMode::EmitRust { keep_build_dir } => *keep_build_dir,
                                _ => false,
                            },
                            ..BackendConfig::default()
                        },
                        &output_root,
                    ) {
                        Ok(artifact) => {
                            if frontend_config.output.mode != OutputMode::Json {
                                let _ =
                                    writeln!(stdout, "{}", summarize_emitted_artifact(&artifact));
                                let _ = writeln!(stdout, "✓ Compilation successful!");
                            }
                        }
                        Err(error) => {
                            diagnostics.add_error(
                                &BasicError {
                                    message: error.to_string(),
                                },
                                None,
                            );
                        }
                    }
                }
            }
        }
        Err(_) => {}
    }

    let rendered = diagnostics.output(output_format);
    let rendered = if frontend_config.output.mode == OutputMode::Human {
        crate::colorize::colorize_diagnostics(&rendered)
    } else {
        rendered
    };
    if !rendered.trim().is_empty() {
        let _ = writeln!(stdout, "{rendered}");
    }

    if diagnostics.has_errors() {
        1
    } else {
        0
    }
}

fn compile_file(
    file_path: &str,
    resolver_config: &ResolverConfig,
    diagnostics: &mut DiagnosticReport,
) -> Result<LoweredWorkspace, ()> {
    let path = Path::new(file_path);
    if !path.exists() {
        diagnostics.add_error(
            &BasicError {
                message: format!("File not found: {}", file_path),
            },
            None,
        );
        return Err(());
    }

    let mut file_stream = if path.is_dir() {
        FileStream::from_folder(file_path).map_err(|e| {
            report_input_error(diagnostics, e.as_ref(), file_path);
        })?
    } else {
        FileStream::from_file(file_path).map_err(|e| {
            report_input_error(diagnostics, e.as_ref(), file_path);
        })?
    };

    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut file_stream);
    let mut ast_parser = AstParser::new();
    match ast_parser.parse_package(&mut lexer) {
        Ok(package) => {
            let package_session = PackageSession::with_config(PackageConfig {
                std_root: resolver_config.std_root.clone(),
                package_store_root: resolver_config.package_store_root.clone(),
                package_cache_root: None,
                package_git_cache_root: None,
            });
            let prepared = match package_session.prepare_entry_package(package) {
                Ok(prepared) => prepared,
                Err(error) => {
                    add_compiler_glitch(diagnostics, &error);
                    return Err(());
                }
            };
            match fol_resolver::resolve_prepared_workspace_with_config(
                prepared,
                resolver_config.clone(),
            ) {
                Ok(resolved) => match Typechecker::new().check_resolved_workspace(resolved) {
                    Ok(typed) => match Lowerer::new().lower_typed_workspace(typed) {
                        Ok(lowered) => Ok(lowered),
                        Err(errors) => {
                            for error in errors {
                                add_compiler_glitch(diagnostics, &error);
                            }
                            Err(())
                        }
                    },
                    Err(errors) => {
                        for error in errors {
                            add_compiler_glitch(diagnostics, &error);
                        }
                        Err(())
                    }
                },
                Err(errors) => {
                    for error in errors {
                        add_compiler_glitch(diagnostics, &error);
                    }
                    Err(())
                }
            }
        }
        Err(errors) => {
            for error in errors {
                add_compiler_glitch(diagnostics, error.as_ref());
            }
            Err(())
        }
    }
}

fn report_input_error(diagnostics: &mut DiagnosticReport, error: &dyn Glitch, file: &str) {
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

fn add_compiler_glitch(report: &mut DiagnosticReport, error: &dyn Glitch) {
    if let Some(diagnostic) = lower_compiler_glitch(error) {
        report.add_diagnostic(diagnostic);
    } else {
        report.add_error(error, None);
    }
}

fn lower_compiler_glitch(error: &dyn Glitch) -> Option<fol_diagnostics::Diagnostic> {
    error
        .as_any()
        .downcast_ref::<ParseError>()
        .map(ToDiagnostic::to_diagnostic)
        .or_else(|| {
            error
                .as_any()
                .downcast_ref::<PackageError>()
                .map(ToDiagnostic::to_diagnostic)
        })
        .or_else(|| {
            error
                .as_any()
                .downcast_ref::<ResolverError>()
                .map(ToDiagnostic::to_diagnostic)
        })
        .or_else(|| {
            error
                .as_any()
                .downcast_ref::<TypecheckError>()
                .map(ToDiagnostic::to_diagnostic)
        })
        .or_else(|| {
            error
                .as_any()
                .downcast_ref::<LoweringError>()
                .map(ToDiagnostic::to_diagnostic)
        })
}

fn render_direct_diagnostics(report: &DiagnosticReport, mode: OutputMode) -> String {
    let rendered = report.output(match mode {
        OutputMode::Json => OutputFormat::Json,
        _ => OutputFormat::Human,
    });
    if mode == OutputMode::Human {
        crate::colorize::colorize_diagnostics(&rendered)
    } else {
        rendered
    }
}
