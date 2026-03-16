use crate::{
    FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendResult,
    WORKSPACE_FILE_NAME,
};
use std::fs;
use std::path::Path;

pub fn init_current_dir(root: &Path) -> FrontendResult<FrontendCommandResult> {
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/main.fol"), "")?;
    fs::write(root.join("package.yaml"), "")?;
    fs::write(root.join("build.fol"), "")?;

    Ok(FrontendCommandResult::new("init", "initialized current directory").with_artifact(
        FrontendArtifactSummary::new(
            FrontendArtifactKind::PackageRoot,
            "current-directory",
            Some(root.to_path_buf()),
        ),
    ))
}

pub fn init_workspace_root(root: &Path) -> FrontendResult<FrontendCommandResult> {
    std::fs::create_dir_all(root)?;
    std::fs::write(root.join(WORKSPACE_FILE_NAME), "")?;

    Ok(FrontendCommandResult::new("init", "initialized workspace root").with_artifact(
        FrontendArtifactSummary::new(
            FrontendArtifactKind::WorkspaceRoot,
            "workspace-root",
            Some(root.to_path_buf()),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::{init_current_dir, init_workspace_root};
    use crate::FrontendArtifactKind;
    use std::{fs, path::PathBuf};

    #[test]
    fn init_shell_returns_current_directory_summary() {
        let root = PathBuf::from("/tmp/demo");
        let result = init_current_dir(&root).unwrap();

        assert_eq!(result.command, "init");
        assert_eq!(result.summary, "initialized current directory");
        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.artifacts[0].kind, FrontendArtifactKind::PackageRoot);
        assert_eq!(result.artifacts[0].path.as_ref(), Some(&root));
    }

    #[test]
    fn init_shell_creates_package_scaffold_in_current_directory() {
        let root = std::env::temp_dir().join(format!("fol_frontend_init_pkg_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        init_current_dir(&root).unwrap();

        assert!(root.join("src").is_dir());
        assert!(root.join("src/main.fol").is_file());
        assert!(root.join("package.yaml").is_file());
        assert!(root.join("build.fol").is_file());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_init_creates_workspace_root_file() {
        let root = std::env::temp_dir().join(format!("fol_frontend_init_ws_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        let result = init_workspace_root(&root).unwrap();

        assert_eq!(result.artifacts[0].kind, FrontendArtifactKind::WorkspaceRoot);
        assert!(root.join("fol.work.yaml").is_file());

        fs::remove_dir_all(root).ok();
    }
}
