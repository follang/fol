use crate::{
    FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendResult,
    WORKSPACE_FILE_NAME,
};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageTargetKind {
    Bin,
    Lib,
}

pub fn package_target_kind(bin: bool, lib: bool) -> PackageTargetKind {
    if lib && !bin {
        PackageTargetKind::Lib
    } else {
        PackageTargetKind::Bin
    }
}

pub fn init_current_dir(root: &Path) -> FrontendResult<FrontendCommandResult> {
    init_package_root(root, PackageTargetKind::Bin)
}

pub fn init_package_root(
    root: &Path,
    target: PackageTargetKind,
) -> FrontendResult<FrontendCommandResult> {
    fs::create_dir_all(root.join("src"))?;
    let source_file = match target {
        PackageTargetKind::Bin => "main.fol",
        PackageTargetKind::Lib => "lib.fol",
    };
    fs::write(root.join("src").join(source_file), "")?;
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

pub fn init_root(
    root: &Path,
    workspace: bool,
    target: PackageTargetKind,
) -> FrontendResult<FrontendCommandResult> {
    if workspace {
        init_workspace_root(root)
    } else {
        init_package_root(root, target)
    }
}

pub fn new_project(parent: &Path, name: &str) -> FrontendResult<FrontendCommandResult> {
    new_project_with_mode(parent, name, false, PackageTargetKind::Bin)
}

pub fn new_project_with_mode(
    parent: &Path,
    name: &str,
    workspace: bool,
    target: PackageTargetKind,
) -> FrontendResult<FrontendCommandResult> {
    let root = parent.join(name);
    init_root(&root, workspace, target).map(|result| FrontendCommandResult {
        command: "new".to_string(),
        summary: format!("created project '{}'", name),
        artifacts: result.artifacts,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        init_current_dir, init_package_root, init_root, init_workspace_root, new_project,
        new_project_with_mode, package_target_kind, PackageTargetKind,
    };
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
    fn bin_target_scaffolding_uses_main_entry_file() {
        let root = std::env::temp_dir().join(format!("fol_frontend_bin_target_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        init_package_root(&root, PackageTargetKind::Bin).unwrap();

        assert!(root.join("src/main.fol").is_file());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn lib_target_scaffolding_uses_library_entry_file() {
        let root = std::env::temp_dir().join(format!("fol_frontend_lib_target_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        init_package_root(&root, PackageTargetKind::Lib).unwrap();

        assert!(root.join("src/lib.fol").is_file());

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

        let init_result = init_root(&init_root_dir, true, PackageTargetKind::Bin).unwrap();
        let new_result =
            new_project_with_mode(&new_parent, "demo", true, PackageTargetKind::Bin).unwrap();

        assert_eq!(init_result.artifacts[0].kind, FrontendArtifactKind::WorkspaceRoot);
        assert_eq!(new_result.artifacts[0].kind, FrontendArtifactKind::WorkspaceRoot);
        assert!(init_root_dir.join("fol.work.yaml").is_file());
        assert!(new_parent.join("demo").join("fol.work.yaml").is_file());

        fs::remove_dir_all(init_root_dir).ok();
        fs::remove_dir_all(new_parent).ok();
    }

    #[test]
    fn package_target_selection_prefers_library_when_requested() {
        assert_eq!(package_target_kind(false, false), PackageTargetKind::Bin);
        assert_eq!(package_target_kind(true, false), PackageTargetKind::Bin);
        assert_eq!(package_target_kind(false, true), PackageTargetKind::Lib);
    }

    #[test]
    fn package_mode_switches_init_and_new_into_library_roots() {
        let init_root_dir = std::env::temp_dir().join(format!("fol_frontend_init_lib_{}", std::process::id()));
        let new_parent = std::env::temp_dir().join(format!("fol_frontend_new_lib_{}", std::process::id()));
        fs::create_dir_all(&init_root_dir).unwrap();
        fs::create_dir_all(&new_parent).unwrap();

        init_root(&init_root_dir, false, PackageTargetKind::Lib).unwrap();
        new_project_with_mode(&new_parent, "demo", false, PackageTargetKind::Lib).unwrap();

        assert!(init_root_dir.join("src/lib.fol").is_file());
        assert!(new_parent.join("demo").join("src/lib.fol").is_file());

        fs::remove_dir_all(init_root_dir).ok();
        fs::remove_dir_all(new_parent).ok();
    }
}
