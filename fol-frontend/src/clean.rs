use crate::{FrontendCommandResult, FrontendConfig, FrontendResult, FrontendWorkspace};

pub fn clean_workspace_with_config(
    workspace: &FrontendWorkspace,
    _config: &FrontendConfig,
) -> FrontendResult<FrontendCommandResult> {
    Ok(FrontendCommandResult::new(
        "clean",
        format!("cleaned workspace at {}", workspace.root.root.display()),
    ))
}

pub fn clean_workspace(workspace: &FrontendWorkspace) -> FrontendResult<FrontendCommandResult> {
    clean_workspace_with_config(workspace, &FrontendConfig::default())
}

#[cfg(test)]
mod tests {
    use super::clean_workspace;
    use crate::{FrontendWorkspace, WorkspaceRoot};
    use std::path::PathBuf;

    #[test]
    fn clean_workspace_exposes_a_stable_command_shell() {
        let workspace = FrontendWorkspace::new(WorkspaceRoot::new(PathBuf::from("/tmp/demo")));

        let result = clean_workspace(&workspace).unwrap();

        assert_eq!(result.command, "clean");
        assert!(result.summary.contains("/tmp/demo"));
    }
}
