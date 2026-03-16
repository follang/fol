use std::path::PathBuf;

pub const WORKSPACE_FILE_NAME: &str = "fol.work.yaml";
pub const PACKAGE_FILE_NAME: &str = "package.yaml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRoot {
    pub root: PathBuf,
    pub config_file: PathBuf,
}

impl WorkspaceRoot {
    pub fn new(root: PathBuf) -> Self {
        Self {
            config_file: root.join(WORKSPACE_FILE_NAME),
            root,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageRoot {
    pub root: PathBuf,
    pub manifest_file: PathBuf,
}

impl PackageRoot {
    pub fn new(root: PathBuf) -> Self {
        Self {
            manifest_file: root.join(PACKAGE_FILE_NAME),
            root,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PackageRoot, WorkspaceRoot, PACKAGE_FILE_NAME, WORKSPACE_FILE_NAME};
    use std::path::PathBuf;

    #[test]
    fn workspace_root_model_uses_canonical_workspace_filename() {
        let root = WorkspaceRoot::new(PathBuf::from("/tmp/demo"));

        assert_eq!(WORKSPACE_FILE_NAME, "fol.work.yaml");
        assert_eq!(root.root, PathBuf::from("/tmp/demo"));
        assert_eq!(root.config_file, PathBuf::from("/tmp/demo").join("fol.work.yaml"));
    }

    #[test]
    fn package_root_model_uses_canonical_package_filename() {
        let root = PackageRoot::new(PathBuf::from("/tmp/demo"));

        assert_eq!(PACKAGE_FILE_NAME, "package.yaml");
        assert_eq!(root.root, PathBuf::from("/tmp/demo"));
        assert_eq!(root.manifest_file, PathBuf::from("/tmp/demo").join("package.yaml"));
    }
}
