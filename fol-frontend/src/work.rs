use crate::{
    FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendWorkspace,
};

pub fn work_info(workspace: &FrontendWorkspace) -> FrontendCommandResult {
    let mut result = FrontendCommandResult::new("work info", workspace.info_summary_lines().join("\n"));
    result.artifacts.push(FrontendArtifactSummary::new(
        FrontendArtifactKind::WorkspaceRoot,
        "workspace-root",
        Some(workspace.root.root.clone()),
    ));
    result
}

#[cfg(test)]
mod tests {
    use super::work_info;
    use crate::{FrontendWorkspace, WorkspaceRoot};
    use std::path::PathBuf;

    #[test]
    fn work_info_returns_workspace_summary_as_command_result() {
        let workspace = FrontendWorkspace::new(WorkspaceRoot::new(PathBuf::from("/tmp/demo")));
        let result = work_info(&workspace);

        assert_eq!(result.command, "work info");
        assert!(result.summary.contains("root=/tmp/demo"));
        assert_eq!(result.artifacts.len(), 1);
    }
}
