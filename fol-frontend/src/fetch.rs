use crate::{
    FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendError,
    FrontendErrorKind, FrontendResult, FrontendWorkspace,
};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendPreparedPackage {
    pub root: PathBuf,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendPackagePreparation {
    pub package_config: fol_package::PackageConfig,
    pub packages: Vec<FrontendPreparedPackage>,
}

pub fn prepare_workspace_packages(
    workspace: &FrontendWorkspace,
) -> FrontendResult<FrontendPackagePreparation> {
    let package_config = fol_package::PackageConfig {
        std_root: workspace
            .std_root_override
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        package_store_root: workspace
            .package_store_root_override
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        package_cache_root: Some(workspace.cache_root.to_string_lossy().to_string()),
    };

    let packages = workspace
        .members
        .iter()
        .map(|member| {
            let metadata = fol_package::parse_package_metadata(&member.manifest_file).map_err(|error| {
                FrontendError::new(FrontendErrorKind::CommandFailed, error.to_string())
            })?;
            let build_path = member.root.join("build.fol");
            fol_package::parse_package_build(&build_path)
                .map_err(|error| FrontendError::new(FrontendErrorKind::CommandFailed, error.to_string()))?;

            Ok(FrontendPreparedPackage {
                root: member.root.clone(),
                name: metadata.name,
                version: metadata.version,
            })
        })
        .collect::<FrontendResult<Vec<_>>>()?;

    Ok(FrontendPackagePreparation {
        package_config,
        packages,
    })
}

pub fn fetch_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    let preparation = prepare_workspace_packages(workspace)?;
    let mut result = FrontendCommandResult::new(
        "fetch",
        format!("prepared {} workspace package(s)", preparation.packages.len()),
    );
    for package in preparation.packages {
        result.artifacts.push(FrontendArtifactSummary::new(
            FrontendArtifactKind::PackageRoot,
            package.name,
            Some(package.root),
        ));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{fetch_workspace, prepare_workspace_packages};
    use crate::{FrontendWorkspace, PackageRoot, WorkspaceRoot};
    use std::{fs, path::PathBuf};

    #[test]
    fn package_preparation_reads_formal_workspace_members() {
        let root = std::env::temp_dir().join(format!("fol_frontend_prepare_{}", std::process::id()));
        let app = root.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app.clone())],
            std_root_override: Some(root.join("std")),
            package_store_root_override: Some(root.join(".fol/pkg")),
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let preparation = prepare_workspace_packages(&workspace).unwrap();

        assert_eq!(preparation.packages.len(), 1);
        assert_eq!(preparation.packages[0].root, app);
        assert_eq!(preparation.packages[0].name, "app");
        assert_eq!(
            preparation.package_config.package_store_root,
            Some(root.join(".fol/pkg").to_string_lossy().to_string())
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn package_preparation_rejects_members_without_formal_build_files() {
        let root = std::env::temp_dir().join(format!("fol_frontend_prepare_missing_{}", std::process::id()));
        let app = root.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let error = prepare_workspace_packages(&workspace).unwrap_err();

        assert_eq!(error.kind(), crate::FrontendErrorKind::CommandFailed);
        assert!(error.message().contains("build file"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn fetch_workspace_returns_a_command_result_for_prepared_members() {
        let root = std::env::temp_dir().join(format!("fol_frontend_fetch_{}", std::process::id()));
        let app = root.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\"\n").unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app.clone())],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
        };

        let result = fetch_workspace(&workspace).unwrap();

        assert_eq!(result.command, "fetch");
        assert_eq!(result.summary, "prepared 1 workspace package(s)");
        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.artifacts[0].path, Some(app));

        fs::remove_dir_all(root).ok();
    }
}
