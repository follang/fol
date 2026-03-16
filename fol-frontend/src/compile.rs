use crate::{
    FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendConfig,
    FrontendError, FrontendErrorKind, FrontendProfile, FrontendResult, FrontendWorkspace,
};
use std::{fs, path::Path};

pub fn check_workspace_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    for member in &workspace.members {
        compile_member_workspace(workspace, config, &member.root)?;
    }

    let mut result = FrontendCommandResult::new(
        "check",
        format!("checked {} workspace package(s)", workspace.members.len()),
    );
    for member in &workspace.members {
        result.artifacts.push(FrontendArtifactSummary::new(
            FrontendArtifactKind::PackageRoot,
            member
                .root
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("package"),
            Some(member.root.clone()),
        ));
    }
    Ok(result)
}

pub fn check_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    check_workspace_with_config(workspace, &FrontendConfig::default())
}

pub fn build_workspace_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    build_workspace_for_profile_with_config(workspace, config, FrontendProfile::Debug)
}

pub fn build_workspace_for_profile_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
    profile: FrontendProfile,
) -> FrontendResult<FrontendCommandResult> {
    let mut result = FrontendCommandResult::new("build", "built 0 workspace package(s)");
    let output_root = profile_build_root(workspace, profile);
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::BuildRoot,
        format!("{:?}", profile).to_lowercase(),
        Some(output_root.clone()),
    ));

    for member in &workspace.members {
        let lowered = compile_member_workspace(workspace, config, &member.root)?;
        if lowered.entry_candidates().is_empty() {
            continue;
        }
        let backend_session = fol_backend::BackendSession::new(lowered);
        let artifact = fol_backend::emit_backend_artifact(
            &backend_session,
            &backend_config(config),
            &output_root,
        )
        .map_err(|error| FrontendError::new(FrontendErrorKind::CommandFailed, error.to_string()))?;
        let fol_backend::BackendArtifact::CompiledBinary {
            crate_root,
            binary_path,
        } = artifact
        else {
            return Err(FrontendError::new(
                FrontendErrorKind::CommandFailed,
                "build command expected a compiled backend artifact",
            ));
        };
        let label = member
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("package")
            .to_string();
        result.artifacts.push(FrontendArtifactSummary::new(
            FrontendArtifactKind::EmittedRust,
            format!("{label}-crate"),
            Some(std::path::PathBuf::from(crate_root)),
        ));
        result.artifacts.push(FrontendArtifactSummary::new(
            FrontendArtifactKind::Binary,
            label,
            Some(std::path::PathBuf::from(binary_path)),
        ));
    }

    if result.artifacts.is_empty() {
        return Err(FrontendError::new(
            FrontendErrorKind::InvalidInput,
            "build command did not find any runnable workspace packages",
        ));
    }

    let binary_count = result
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == FrontendArtifactKind::Binary)
        .count();
    result.summary = format!(
        "built {binary_count} workspace package(s) into {}",
        output_root.display()
    );
    Ok(result)
}

pub fn build_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    build_workspace_with_config(workspace, &FrontendConfig::default())
}

pub fn profile_build_root(
    workspace: &FrontendWorkspace,
    profile: FrontendProfile,
) -> std::path::PathBuf {
    workspace.build_root.join(match profile {
        FrontendProfile::Debug => "debug",
        FrontendProfile::Release => "release",
    })
}

pub fn run_workspace_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    run_workspace_with_args_and_config(workspace, config, &[])
}

pub fn run_workspace_with_args_and_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
    args: &[String],
) -> FrontendResult<FrontendCommandResult> {
    let built = build_workspace_with_config(workspace, config)?;
    let binaries = built
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == FrontendArtifactKind::Binary)
        .collect::<Vec<_>>();
    if binaries.len() != 1 {
        return Err(FrontendError::new(
            FrontendErrorKind::InvalidInput,
            format!(
                "run command requires exactly one runnable workspace package, found {}",
                binaries.len()
            ),
        ));
    }

    let binary = binaries[0]
        .path
        .as_ref()
        .cloned()
        .ok_or_else(|| FrontendError::new(FrontendErrorKind::Internal, "build result is missing a binary path"))?;
    let output = std::process::Command::new(&binary)
        .args(args)
        .output()
        .map_err(|error| FrontendError::new(FrontendErrorKind::CommandFailed, error.to_string()))?;

    if !output.status.success() {
        return Err(FrontendError::new(
            FrontendErrorKind::CommandFailed,
            format!(
                "run command failed for '{}': status {}",
                binary.display(),
                output.status
            ),
        ));
    }

    let mut result = FrontendCommandResult::new(
        "run",
        format!("ran {}", binary.display()),
    );
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::Binary,
        "binary",
        Some(binary),
    ));
    Ok(result)
}

pub fn run_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    run_workspace_with_config(workspace, &FrontendConfig::default())
}

pub fn test_workspace_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    test_workspace_selected_with_config(workspace, config, None)
}

pub fn test_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    test_workspace_with_config(workspace, &FrontendConfig::default())
}

pub fn test_package_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
    package_name: &str,
) -> FrontendResult<FrontendCommandResult> {
    test_workspace_selected_with_config(workspace, config, Some(package_name))
}

pub fn test_package(
    workspace: &FrontendWorkspace,
    package_name: &str,
) -> FrontendResult<FrontendCommandResult> {
    test_package_with_config(workspace, &FrontendConfig::default(), package_name)
}

pub fn emit_rust_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    let output_root = workspace.build_root.join("emit").join("rust");
    let mut result = FrontendCommandResult::new("emit rust", "emitted 0 Rust crate(s)");
    let mut emitted = 0usize;
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::BuildRoot,
        "emit-rust-root",
        Some(output_root.clone()),
    ));

    for member in &workspace.members {
        let lowered = compile_member_workspace(workspace, config, &member.root)?;
        let backend_session = fol_backend::BackendSession::new(lowered);
        let artifact = fol_backend::emit_backend_artifact(
            &backend_session,
            &fol_backend::BackendConfig {
                mode: fol_backend::BackendMode::EmitSource,
                keep_build_dir: true,
                ..backend_config(config)
            },
            &output_root,
        )
        .map_err(|error| FrontendError::new(FrontendErrorKind::CommandFailed, error.to_string()))?;
        let fol_backend::BackendArtifact::RustSourceCrate { root, .. } = artifact else {
            return Err(FrontendError::new(
                FrontendErrorKind::Internal,
                "emit rust expected a backend source artifact",
            ));
        };
        emitted += 1;
        result.artifacts.push(FrontendArtifactSummary::new(
            FrontendArtifactKind::EmittedRust,
            member
                .root
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("package"),
            Some(std::path::PathBuf::from(root)),
        ));
    }

    result.summary = format!("emitted {emitted} Rust crate(s) into {}", output_root.display());
    Ok(result)
}

pub fn emit_rust(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    emit_rust_with_config(workspace, &FrontendConfig::default())
}

pub fn emit_lowered_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    let output_root = workspace.build_root.join("emit").join("lowered");
    fs::create_dir_all(&output_root).map_err(|error| {
        FrontendError::new(
            FrontendErrorKind::CommandFailed,
            format!("failed to create lowered emit root '{}': {error}", output_root.display()),
        )
    })?;

    let mut result = FrontendCommandResult::new("emit lowered", "emitted 0 lowered snapshot(s)");
    let mut emitted = 0usize;
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::BuildRoot,
        "emit-lowered-root",
        Some(output_root.clone()),
    ));

    for member in &workspace.members {
        let lowered = compile_member_workspace(workspace, config, &member.root)?;
        let rendered = fol_lower::render_lowered_workspace(&lowered);
        let label = member
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("package");
        let snapshot_path = output_root.join(format!("{label}.lowered.txt"));
        fs::write(&snapshot_path, rendered).map_err(|error| {
            FrontendError::new(
                FrontendErrorKind::CommandFailed,
                format!(
                    "failed to write lowered snapshot '{}': {error}",
                    snapshot_path.display()
                ),
            )
        })?;
        emitted += 1;
        result.artifacts.push(FrontendArtifactSummary::new(
            FrontendArtifactKind::LoweredSnapshot,
            label,
            Some(snapshot_path),
        ));
    }

    result.summary = format!(
        "emitted {emitted} lowered snapshot(s) into {}",
        output_root.display()
    );
    Ok(result)
}

pub fn emit_lowered(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    emit_lowered_with_config(workspace, &FrontendConfig::default())
}

pub fn compile_member_workspace(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
    package_root: &Path,
) -> FrontendResult<fol_lower::LoweredWorkspace> {
    let package_root_str = package_root.to_string_lossy().to_string();
    let mut file_stream = fol_stream::FileStream::from_folder(&package_root_str)
        .map_err(|error| FrontendError::new(FrontendErrorKind::CommandFailed, error.to_string()))?;
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut file_stream);
    let mut parser = fol_parser::ast::AstParser::new();
    let syntax = parser
        .parse_package(&mut lexer)
        .map_err(|errors| {
            FrontendError::new(
                FrontendErrorKind::CommandFailed,
                errors
                    .into_iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        })?;
    let prepared = fol_package::PackageSession::with_config(fol_package::PackageConfig::default())
        .prepare_entry_package(syntax)
        .map_err(FrontendError::from)?;

    let resolved = fol_resolver::resolve_prepared_workspace_with_config(
        prepared,
        resolver_config(workspace, config),
    )
    .map_err(lower_resolver_errors)?;
    let typed = fol_typecheck::Typechecker::new()
        .check_resolved_workspace(resolved)
        .map_err(lower_typecheck_errors)?;
    fol_lower::Lowerer::new()
        .lower_typed_workspace(typed)
        .map_err(lower_lowering_errors)
}

fn resolver_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> fol_resolver::ResolverConfig {
    fol_resolver::ResolverConfig {
        std_root: config
            .std_root_override
            .clone()
            .or_else(|| workspace.std_root_override.clone())
            .map(|path| path.to_string_lossy().to_string()),
        package_store_root: config
            .package_store_root_override
            .clone()
            .or_else(|| workspace.package_store_root_override.clone())
            .map(|path| path.to_string_lossy().to_string()),
    }
}

fn backend_config(config: &FrontendConfig) -> fol_backend::BackendConfig {
    fol_backend::BackendConfig {
        keep_build_dir: config.keep_build_dir,
        ..fol_backend::BackendConfig::default()
    }
}

fn test_workspace_selected_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
    selected_package: Option<&str>,
) -> FrontendResult<FrontendCommandResult> {
    let selected_members = selected_workspace_members(workspace, selected_package)?;
    let mut result = FrontendCommandResult::new("test", "tested 0 workspace package(s)");
    let mut tested_count = 0usize;

    for member in selected_members {
        let member_workspace = FrontendWorkspace {
            root: workspace.root.clone(),
            members: vec![member.clone()],
            std_root_override: workspace.std_root_override.clone(),
            package_store_root_override: workspace.package_store_root_override.clone(),
            build_root: workspace.build_root.clone(),
            cache_root: workspace.cache_root.clone(),
        };
        let member_result = run_workspace_with_config(&member_workspace, config)?;
        result.artifacts.extend(member_result.artifacts);
        tested_count += 1;
    }

    result.summary = format!("tested {tested_count} workspace package(s)");
    Ok(result)
}

fn selected_workspace_members(
    workspace: &FrontendWorkspace,
    selected_package: Option<&str>,
) -> FrontendResult<Vec<crate::PackageRoot>> {
    match selected_package {
        Some(selected_package) => workspace
            .members
            .iter()
            .find(|member| {
                member
                    .root
                    .file_name()
                    .and_then(|name| name.to_str())
                    == Some(selected_package)
            })
            .cloned()
            .map(|member| vec![member])
            .ok_or_else(|| {
                FrontendError::new(
                    FrontendErrorKind::InvalidInput,
                    format!("workspace package '{selected_package}' was not found"),
                )
            }),
        None => Ok(workspace.members.clone()),
    }
}

fn lower_resolver_errors(errors: Vec<fol_resolver::ResolverError>) -> FrontendError {
    FrontendError::new(
        FrontendErrorKind::CommandFailed,
        errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn lower_typecheck_errors(errors: Vec<fol_typecheck::TypecheckError>) -> FrontendError {
    FrontendError::new(
        FrontendErrorKind::CommandFailed,
        errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn lower_lowering_errors(errors: Vec<fol_lower::LoweringError>) -> FrontendError {
    FrontendError::new(
        FrontendErrorKind::CommandFailed,
        errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_workspace, build_workspace_for_profile_with_config, check_workspace, emit_lowered,
        emit_rust, profile_build_root, run_workspace, run_workspace_with_args_and_config,
        test_package, test_workspace,
    };
    use crate::{FrontendProfile, FrontendWorkspace, PackageRoot, WorkspaceRoot};
    use std::{fs, path::PathBuf};

    #[test]
    fn check_workspace_runs_the_real_pipeline_for_workspace_members() {
        let root = std::env::temp_dir().join(format!("fol_frontend_check_{}", std::process::id()));
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app.clone())],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = check_workspace(&workspace).unwrap();

        assert_eq!(result.command, "check");
        assert_eq!(result.summary, "checked 1 workspace package(s)");
        assert_eq!(result.artifacts[0].path, Some(app));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn build_workspace_runs_the_backend_for_runnable_members() {
        let root = std::env::temp_dir().join(format!("fol_frontend_build_{}", std::process::id()));
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = build_workspace(&workspace).unwrap();

        assert_eq!(result.command, "build");
        assert!(result.summary.contains("built 1 workspace package(s) into "));
        assert_eq!(result.artifacts.len(), 2);
        assert_eq!(result.artifacts[0].kind, FrontendArtifactKind::EmittedRust);
        assert!(result.artifacts[1]
            .path
            .as_ref()
            .expect("binary path")
            .is_file());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn build_output_roots_are_profile_scoped() {
        let workspace = FrontendWorkspace::new(WorkspaceRoot::new(PathBuf::from("/tmp/demo")));

        assert_eq!(
            profile_build_root(&workspace, FrontendProfile::Debug),
            PathBuf::from("/tmp/demo/.fol/build/debug")
        );
        assert_eq!(
            profile_build_root(&workspace, FrontendProfile::Release),
            PathBuf::from("/tmp/demo/.fol/build/release")
        );
    }

    #[test]
    fn build_workspace_uses_profile_specific_output_roots() {
        let root = std::env::temp_dir().join(format!("fol_frontend_build_profile_{}", std::process::id()));
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = build_workspace_for_profile_with_config(
            &workspace,
            &crate::FrontendConfig::default(),
            FrontendProfile::Release,
        )
        .unwrap();

        let binary = result.artifacts[1].path.as_ref().expect("binary path");
        assert!(binary.display().to_string().contains("/.fol/build/release/"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn run_workspace_executes_a_single_runnable_member() {
        let root = std::env::temp_dir().join(format!("fol_frontend_run_{}", std::process::id()));
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = run_workspace(&workspace).unwrap();

        assert_eq!(result.command, "run");
        assert!(result.summary.contains("ran "));
        assert_eq!(result.artifacts.len(), 1);
        assert!(result.artifacts[0]
            .path
            .as_ref()
            .expect("binary path")
            .is_file());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn run_workspace_passes_through_binary_arguments() {
        let root = std::env::temp_dir().join(format!("fol_frontend_run_args_{}", std::process::id()));
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = run_workspace_with_args_and_config(
            &workspace,
            &crate::FrontendConfig::default(),
            &["--demo".to_string(), "123".to_string()],
        )
        .unwrap();

        assert_eq!(result.command, "run");
        assert_eq!(result.artifacts.len(), 1);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn build_workspace_keeps_generated_crate_dirs_when_requested() {
        let root =
            std::env::temp_dir().join(format!("fol_frontend_keep_build_dir_{}", std::process::id()));
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };
        let config = crate::FrontendConfig {
            keep_build_dir: true,
            ..crate::FrontendConfig::default()
        };

        let result = build_workspace_with_config(&workspace, &config).unwrap();
        let crate_root = result.artifacts[0].path.as_ref().unwrap();

        assert!(crate_root.exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn test_workspace_runs_single_workspace_members() {
        let root = std::env::temp_dir().join(format!("fol_frontend_test_{}", std::process::id()));
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = test_workspace(&workspace).unwrap();

        assert_eq!(result.command, "test");
        assert_eq!(result.summary, "tested 1 workspace package(s)");
        assert_eq!(result.artifacts.len(), 1);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn test_package_selects_a_single_named_workspace_member() {
        let root =
            std::env::temp_dir().join(format!("fol_frontend_test_package_{}", std::process::id()));
        let app = root.join("app");
        let lib = root.join("lib");
        for package in [&app, &lib] {
            let src = package.join("src");
            fs::create_dir_all(&src).unwrap();
            fs::write(package.join("package.yaml"), "name: pkg\nversion: 0.1.0\n").unwrap();
            fs::write(package.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
            fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();
        }

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app), PackageRoot::new(lib)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = test_package(&workspace, "lib").unwrap();

        assert_eq!(result.command, "test");
        assert_eq!(result.summary, "tested 1 workspace package(s)");
        assert_eq!(result.artifacts.len(), 1);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn emit_rust_materializes_generated_crates_for_workspace_members() {
        let root = std::env::temp_dir().join(format!("fol_frontend_emit_rust_{}", std::process::id()));
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = emit_rust(&workspace).unwrap();

        assert_eq!(result.command, "emit rust");
        assert_eq!(
            result.summary,
            format!("emitted 1 Rust crate(s) into {}", workspace.build_root.join("emit").join("rust").display())
        );
        assert_eq!(result.artifacts[0].kind, FrontendArtifactKind::EmittedRust);
        assert!(result.artifacts[0].path.as_ref().unwrap().is_dir());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn emit_lowered_materializes_rendered_workspace_snapshots() {
        let root =
            std::env::temp_dir().join(format!("fol_frontend_emit_lowered_{}", std::process::id()));
        let app = root.join("app");
        let src = app.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(src.join("main.fol"), "fun[] main(): int = {\n    return 0\n}\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = emit_lowered(&workspace).unwrap();

        assert_eq!(result.command, "emit lowered");
        assert_eq!(
            result.summary,
            format!(
                "emitted 1 lowered snapshot(s) into {}",
                workspace.build_root.join("emit").join("lowered").display()
            )
        );
        assert_eq!(result.artifacts[0].kind, FrontendArtifactKind::LoweredSnapshot);
        assert!(result.artifacts[0].path.as_ref().unwrap().is_file());

        fs::remove_dir_all(root).ok();
    }
}
