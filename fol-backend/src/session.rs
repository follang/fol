use fol_lower::LoweredWorkspace;

/// Compiler-owned backend session for one lowered workspace translation run.
#[derive(Debug, Clone)]
pub struct BackendSession {
    workspace: LoweredWorkspace,
}

impl BackendSession {
    pub fn new(workspace: LoweredWorkspace) -> Self {
        Self { workspace }
    }

    pub fn workspace(&self) -> &LoweredWorkspace {
        &self.workspace
    }

    pub fn into_workspace(self) -> LoweredWorkspace {
        self.workspace
    }
}

#[cfg(test)]
mod tests {
    use super::BackendSession;
    use crate::testing::sample_lowered_workspace;

    #[test]
    fn backend_session_keeps_lowered_workspace_inputs() {
        let workspace = sample_lowered_workspace();
        let expected_packages = workspace.package_count();
        let expected_entry = workspace.entry_identity().display_name.clone();

        let session = BackendSession::new(workspace.clone());

        assert_eq!(session.workspace().package_count(), expected_packages);
        assert_eq!(session.workspace().entry_identity().display_name, expected_entry);
        assert_eq!(session.into_workspace().package_count(), workspace.package_count());
    }
}
