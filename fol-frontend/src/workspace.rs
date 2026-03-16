use crate::{FrontendError, FrontendErrorKind, FrontendResult, PackageRoot, WorkspaceRoot};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendWorkspace {
    pub root: WorkspaceRoot,
    pub members: Vec<PackageRoot>,
}

impl FrontendWorkspace {
    pub fn new(root: WorkspaceRoot) -> Self {
        Self {
            root,
            members: Vec::new(),
        }
    }

    pub fn with_members(root: WorkspaceRoot, member_paths: &[PathBuf]) -> FrontendResult<Self> {
        Ok(Self {
            members: enumerate_member_packages(&root, member_paths)?,
            root,
        })
    }
}

pub fn enumerate_member_packages(
    workspace_root: &WorkspaceRoot,
    member_paths: &[PathBuf],
) -> FrontendResult<Vec<PackageRoot>> {
    member_paths
        .iter()
        .map(|member| {
            let absolute = absolute_member_root(&workspace_root.root, member);
            let manifest_file = absolute.join(crate::PACKAGE_FILE_NAME);
            if !manifest_file.is_file() {
                return Err(FrontendError::new(
                    FrontendErrorKind::InvalidInput,
                    format!(
                        "workspace member '{}' is missing '{}'",
                        absolute.display(),
                        crate::PACKAGE_FILE_NAME
                    ),
                ));
            }
            Ok(PackageRoot::new(absolute))
        })
        .collect()
}

fn absolute_member_root(workspace_root: &Path, member: &Path) -> PathBuf {
    if member.is_absolute() {
        member.to_path_buf()
    } else {
        workspace_root.join(member)
    }
}

#[cfg(test)]
mod tests {
    use super::{enumerate_member_packages, FrontendWorkspace};
    use crate::WorkspaceRoot;
    use std::{fs, path::PathBuf};

    #[test]
    fn frontend_workspace_starts_from_a_discovered_workspace_root() {
        let workspace = FrontendWorkspace::new(WorkspaceRoot::new(PathBuf::from("/tmp/demo")));

        assert_eq!(workspace.root.root, PathBuf::from("/tmp/demo"));
        assert_eq!(
            workspace.root.config_file,
            PathBuf::from("/tmp/demo/fol.work.yaml")
        );
        assert!(workspace.members.is_empty());
    }

    #[test]
    fn workspace_member_enumeration_loads_declared_package_roots() {
        let root = std::env::temp_dir().join(format!("fol_frontend_members_{}", std::process::id()));
        let app = root.join("app");
        let lib = root.join("lib");
        fs::create_dir_all(&app).unwrap();
        fs::create_dir_all(&lib).unwrap();
        fs::write(root.join("fol.work.yaml"), "members:\n  - app\n  - lib\n").unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(lib.join("package.yaml"), "name: lib\nversion: 0.1.0\n").unwrap();

        let members = enumerate_member_packages(
            &WorkspaceRoot::new(root.clone()),
            &[PathBuf::from("app"), PathBuf::from("lib")],
        )
        .unwrap();

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].root, app);
        assert_eq!(members[1].root, lib);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_member_enumeration_rejects_missing_package_roots() {
        let root = std::env::temp_dir().join(format!("fol_frontend_missing_member_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("fol.work.yaml"), "members:\n  - app\n").unwrap();

        let error = enumerate_member_packages(
            &WorkspaceRoot::new(root.clone()),
            &[PathBuf::from("app")],
        )
        .unwrap_err();

        assert_eq!(error.kind(), crate::FrontendErrorKind::InvalidInput);
        assert!(error.message().contains("missing 'package.yaml'"));

        fs::remove_dir_all(root).ok();
    }
}
