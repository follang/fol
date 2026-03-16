use std::path::PathBuf;

pub const WORKSPACE_FILE_NAME: &str = "fol.work.yaml";

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

#[cfg(test)]
mod tests {
    use super::{WorkspaceRoot, WORKSPACE_FILE_NAME};
    use std::path::PathBuf;

    #[test]
    fn workspace_root_model_uses_canonical_workspace_filename() {
        let root = WorkspaceRoot::new(PathBuf::from("/tmp/demo"));

        assert_eq!(WORKSPACE_FILE_NAME, "fol.work.yaml");
        assert_eq!(root.root, PathBuf::from("/tmp/demo"));
        assert_eq!(root.config_file, PathBuf::from("/tmp/demo").join("fol.work.yaml"));
    }
}
