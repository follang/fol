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

pub fn init_root(root: &Path, workspace: bool) -> FrontendResult<FrontendCommandResult> {
    if workspace {
        init_workspace_root(root)
    } else {
        init_current_dir(root)
    }
}

pub fn new_project(parent: &Path, name: &str) -> FrontendResult<FrontendCommandResult> {
    new_project_with_mode(parent, name, false)
}

pub fn new_project_with_mode(
    parent: &Path,
    name: &str,
    workspace: bool,
) -> FrontendResult<FrontendCommandResult> {
    let root = parent.join(name);
    init_root(&root, workspace).map(|result| FrontendCommandResult {
        command: "new".to_string(),
        summary: format!("created project '{}'", name),
        artifacts: result.artifacts,
    })
}

#[cfg(test)]
mod tests {
    use super::{init_current_dir, init_root, init_workspace_root, new_project, new_project_with_mode};
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

    #[test]
    fn new_project_creates_named_directory_and_package_scaffold() {
        let parent = std::env::temp_dir().join(format!("fol_frontend_new_parent_{}", std::process::id()));
        fs::create_dir_all(&parent).unwrap();

        let result = new_project(&parent, "demo").unwrap();
        let root = parent.join("demo");

        assert_eq!(result.command, "new");
        assert!(root.join("src").is_dir());
        assert!(root.join("package.yaml").is_file());

        fs::remove_dir_all(parent).ok();
    }

    #[test]
    fn workspace_mode_switches_init_and_new_into_workspace_roots() {
        let init_root_dir = std::env::temp_dir().join(format!("fol_frontend_init_mode_{}", std::process::id()));
        let new_parent = std::env::temp_dir().join(format!("fol_frontend_new_mode_{}", std::process::id()));
        fs::create_dir_all(&init_root_dir).unwrap();
        fs::create_dir_all(&new_parent).unwrap();

        let init_result = init_root(&init_root_dir, true).unwrap();
        let new_result = new_project_with_mode(&new_parent, "demo", true).unwrap();

        assert_eq!(init_result.artifacts[0].kind, FrontendArtifactKind::WorkspaceRoot);
        assert_eq!(new_result.artifacts[0].kind, FrontendArtifactKind::WorkspaceRoot);
        assert!(init_root_dir.join("fol.work.yaml").is_file());
        assert!(new_parent.join("demo").join("fol.work.yaml").is_file());

        fs::remove_dir_all(init_root_dir).ok();
        fs::remove_dir_all(new_parent).ok();
    }
}
