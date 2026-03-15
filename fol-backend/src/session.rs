use crate::{
    identity::BackendWorkspaceIdentity,
    trace::{BackendEmittedSourceMap, BackendTrace},
};
use fol_lower::{LoweredEntryCandidate, LoweredWorkspace};
use fol_resolver::PackageIdentity;

/// Compiler-owned backend session for one lowered workspace translation run.
#[derive(Debug, Clone)]
pub struct BackendSession {
    workspace: LoweredWorkspace,
    entry_identity: PackageIdentity,
    package_graph: Vec<PackageIdentity>,
    entry_candidates: Vec<LoweredEntryCandidate>,
    workspace_identity: BackendWorkspaceIdentity,
    source_map: BackendEmittedSourceMap,
    trace: BackendTrace,
}

impl BackendSession {
    pub fn new(workspace: LoweredWorkspace) -> Self {
        let entry_identity = workspace.entry_identity().clone();
        let package_graph = workspace
            .packages()
            .map(|package| package.identity.clone())
            .collect();
        let entry_candidates = workspace.entry_candidates().to_vec();
        let workspace_identity = BackendWorkspaceIdentity::for_workspace(&workspace);
        Self {
            workspace,
            entry_identity,
            package_graph,
            entry_candidates,
            workspace_identity,
            source_map: BackendEmittedSourceMap::new(),
            trace: BackendTrace::new(),
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

    pub fn workspace_identity(&self) -> &BackendWorkspaceIdentity {
        &self.workspace_identity
    }

    pub fn source_map(&self) -> &BackendEmittedSourceMap {
        &self.source_map
    }

    pub fn source_map_mut(&mut self) -> &mut BackendEmittedSourceMap {
        &mut self.source_map
    }

    pub fn trace(&self) -> &BackendTrace {
        &self.trace
    }

    pub fn trace_mut(&mut self) -> &mut BackendTrace {
        &mut self.trace
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
        assert!(session.workspace_identity().crate_dir_name.starts_with("fol-build-app-"));
        assert!(session.source_map().is_empty());
        assert!(session.trace().is_empty());
        assert_eq!(session.into_workspace().package_count(), workspace.package_count());
    }

    #[test]
    fn backend_session_keeps_package_graph_and_hash_stable() {
        let session = BackendSession::new(sample_lowered_workspace());

        let package_names = session
            .package_graph()
            .iter()
            .map(|identity| identity.display_name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(package_names, vec!["app", "shared"]);
        assert_eq!(
            session.workspace_identity().hash,
            BackendSession::new(sample_lowered_workspace())
                .workspace_identity()
                .hash
        );
    }
}
