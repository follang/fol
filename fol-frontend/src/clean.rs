use crate::{FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendConfig, FrontendResult, FrontendWorkspace};
use std::fs;

pub fn clean_workspace_with_config(
    workspace: &FrontendWorkspace,
    _config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    remove_dir_if_present(&workspace.build_root)?;
    remove_dir_if_present(&workspace.cache_root)?;

    let mut result = FrontendCommandResult::new(
        "clean",
        format!(
            "cleaned build root {} and cache root {}",
            workspace.build_root.display(),
            workspace.cache_root.display()
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
    Ok(result)
}

pub fn clean_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    clean_workspace_with_config(workspace, &FrontendConfig::default())
}

#[cfg(test)]
mod tests {
    use super::clean_workspace;
    use crate::{FrontendWorkspace, WorkspaceRoot};
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

        let result = clean_workspace(&workspace).unwrap();

        assert_eq!(result.command, "clean");
        assert!(result.summary.contains(&build_root.display().to_string()));
        assert!(result.summary.contains(&cache_root.display().to_string()));
        assert!(!build_root.exists());
        assert!(!cache_root.exists());

        fs::remove_dir_all(root).ok();
    }
}

fn remove_dir_if_present(path: &std::path::Path) -> FrontendResult<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}
