use crate::{
    FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendConfig,
    FrontendError, FrontendErrorKind, FrontendResult, FrontendWorkspace,
};
use std::path::Path;

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
    use super::check_workspace;
    use crate::{FrontendWorkspace, PackageRoot, WorkspaceRoot};
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
}
