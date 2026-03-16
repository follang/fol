use crate::{FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendConfig, FrontendResult, FrontendWorkspace};
use std::fs;

pub fn clean_workspace_with_config(
    workspace: &FrontendWorkspace,
    config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    remove_dir_if_present(&workspace.build_root)?;
    remove_dir_if_present(&workspace.cache_root)?;
    let package_store_root = config
        .package_store_root_override
        .clone()
        .or_else(|| workspace.package_store_root_override.clone());
    let package_store_removed = clean_workspace_package_store(workspace, package_store_root.as_deref())?;

    let mut result = FrontendCommandResult::new(
        "clean",
        format!(
            "cleaned build root {} and cache root {}{}",
            workspace.build_root.display(),
            workspace.cache_root.display(),
            package_store_root
                .as_ref()
                .map(|path| {
                    if package_store_removed {
                        format!(", and package store {}", path.display())
                    } else {
                        format!(", skipped external package store {}", path.display())
                    }
                })
                .unwrap_or_default()
        ),
    );
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::WorkspaceRoot,
        "build-root",
        Some(workspace.build_root.clone()),
    ));
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::WorkspaceRoot,
        "cache-root",
        Some(workspace.cache_root.clone()),
    ));
    if package_store_removed {
        if let Some(package_store_root) = package_store_root {
            result.artifacts.push(FrontendArtifactSummary::new(
                FrontendArtifactKind::WorkspaceRoot,
                "package-store",
                Some(package_store_root),
            ));
        }
    }
    Ok(result)
}

pub fn clean_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    clean_workspace_with_config(workspace, &FrontendConfig::default())
}

#[cfg(test)]
mod tests {
    use super::clean_workspace_with_config;
    use crate::{FrontendConfig, FrontendWorkspace, WorkspaceRoot};
    use std::{fs, path::PathBuf};

    #[test]
    fn clean_workspace_exposes_a_stable_command_shell() {
        let root = std::env::temp_dir().join(format!("fol_frontend_clean_build_{}", std::process::id()));
        let build_root = root.join(".fol/build");
        let cache_root = root.join(".fol/cache");
        fs::create_dir_all(&build_root).unwrap();
        fs::create_dir_all(&cache_root).unwrap();
        let mut workspace = FrontendWorkspace::new(WorkspaceRoot::new(root.clone()));
        workspace.build_root = build_root.clone();
        workspace.cache_root = cache_root.clone();

        let result = clean_workspace_with_config(&workspace, &FrontendConfig::default()).unwrap();

        assert_eq!(result.command, "clean");
        assert!(result.summary.contains(&build_root.display().to_string()));
        assert!(result.summary.contains(&cache_root.display().to_string()));
        assert!(!build_root.exists());
        assert!(!cache_root.exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn clean_workspace_only_removes_workspace_local_package_stores() {
        let root = std::env::temp_dir().join(format!("fol_frontend_clean_pkg_{}", std::process::id()));
        let local_store = root.join(".fol/pkg");
        let external_store = std::env::temp_dir().join(format!("fol_frontend_clean_external_pkg_{}", std::process::id()));
        fs::create_dir_all(&local_store).unwrap();
        fs::create_dir_all(&external_store).unwrap();

        let mut workspace = FrontendWorkspace::new(WorkspaceRoot::new(root.clone()));
        workspace.build_root = root.join(".fol/build");
        workspace.cache_root = root.join(".fol/cache");
        workspace.package_store_root_override = Some(local_store.clone());

        let local = clean_workspace_with_config(&workspace, &FrontendConfig::default()).unwrap();
        assert!(local.summary.contains("package store"));
        assert!(!local_store.exists());

        fs::create_dir_all(&external_store).unwrap();
        let external = clean_workspace_with_config(
            &workspace,
            &FrontendConfig {
                package_store_root_override: Some(external_store.clone()),
                ..FrontendConfig::default()
            },
        )
        .unwrap();
        assert!(external.summary.contains("skipped external package store"));
        assert!(external_store.exists());

        fs::remove_dir_all(root).ok();
        fs::remove_dir_all(external_store).ok();
    }
}

fn remove_dir_if_present(path: &std::path::Path) -> FrontendResult<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn clean_workspace_package_store(
    workspace: &FrontendWorkspace,
    package_store_root: Option<&std::path::Path>,
) -> FrontendResult<bool> {
    let Some(package_store_root) = package_store_root else {
        return Ok(false);
    };
    let workspace_local_root = workspace.root.root.join(".fol");
    if package_store_root.starts_with(&workspace_local_root) {
        remove_dir_if_present(package_store_root)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
