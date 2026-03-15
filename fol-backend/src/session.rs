use fol_lower::{LoweredEntryCandidate, LoweredWorkspace};
use fol_resolver::PackageIdentity;

/// Compiler-owned backend session for one lowered workspace translation run.
#[derive(Debug, Clone)]
pub struct BackendSession {
    workspace: LoweredWorkspace,
    entry_identity: PackageIdentity,
    package_graph: Vec<PackageIdentity>,
    entry_candidates: Vec<LoweredEntryCandidate>,
}

impl BackendSession {
    pub fn new(workspace: LoweredWorkspace) -> Self {
        let entry_identity = workspace.entry_identity().clone();
        let package_graph = workspace
            .packages()
            .map(|package| package.identity.clone())
            .collect();
        let entry_candidates = workspace.entry_candidates().to_vec();
        Self {
            workspace,
            entry_identity,
            package_graph,
            entry_candidates,
        }
    }

    pub fn workspace(&self) -> &LoweredWorkspace {
        &self.workspace
    }

    pub fn entry_identity(&self) -> &PackageIdentity {
        &self.entry_identity
    }

    pub fn package_graph(&self) -> &[PackageIdentity] {
        &self.package_graph
    }

    pub fn entry_candidates(&self) -> &[LoweredEntryCandidate] {
        &self.entry_candidates
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
        let expected_candidates = workspace.entry_candidates().to_vec();

        let session = BackendSession::new(workspace.clone());

        assert_eq!(session.workspace().package_count(), expected_packages);
        assert_eq!(session.workspace().entry_identity().display_name, expected_entry);
        assert_eq!(session.entry_identity().display_name, expected_entry);
        assert_eq!(session.package_graph().len(), expected_packages);
        assert_eq!(session.entry_candidates(), expected_candidates.as_slice());
        assert_eq!(session.into_workspace().package_count(), workspace.package_count());
    }
}
