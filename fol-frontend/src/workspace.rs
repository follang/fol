use crate::WorkspaceRoot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendWorkspace {
    pub root: WorkspaceRoot,
}

impl FrontendWorkspace {
    pub fn new(root: WorkspaceRoot) -> Self {
        Self { root }
    }
}

#[cfg(test)]
mod tests {
    use super::FrontendWorkspace;
    use crate::WorkspaceRoot;
    use std::path::PathBuf;

    #[test]
    fn frontend_workspace_starts_from_a_discovered_workspace_root() {
        let workspace = FrontendWorkspace::new(WorkspaceRoot::new(PathBuf::from("/tmp/demo")));

        assert_eq!(workspace.root.root, PathBuf::from("/tmp/demo"));
        assert_eq!(
            workspace.root.config_file,
            PathBuf::from("/tmp/demo/fol.work.yaml")
        );
    }
}
