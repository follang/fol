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

pub fn work_list(workspace: &FrontendWorkspace) -> FrontendCommandResult {
    let mut result = FrontendCommandResult::new(
        "work list",
        workspace
            .members
            .iter()
            .map(|member| member.root.display().to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    );
    for member in &workspace.members {
        result.artifacts.push(FrontendArtifactSummary::new(
            FrontendArtifactKind::PackageRoot,
            member
                .root
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("package"),
            Some(member.root.clone()),
        ));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{work_info, work_list};
    use crate::{FrontendWorkspace, PackageRoot, WorkspaceRoot};
    use std::path::PathBuf;

    #[test]
    fn work_info_returns_workspace_summary_as_command_result() {
        let workspace = FrontendWorkspace::new(WorkspaceRoot::new(PathBuf::from("/tmp/demo")));
        let result = work_info(&workspace);

        assert_eq!(result.command, "work info");
        assert!(result.summary.contains("root=/tmp/demo"));
        assert_eq!(result.artifacts.len(), 1);
    }

    #[test]
    fn work_list_returns_member_packages_as_command_result() {
        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(PathBuf::from("/tmp/demo")),
            members: vec![
                PackageRoot::new(PathBuf::from("/tmp/demo/app")),
                PackageRoot::new(PathBuf::from("/tmp/demo/lib")),
            ],
            std_root_override: None,
            package_store_root_override: None,
            build_root: PathBuf::from("/tmp/demo/.fol/build"),
            cache_root: PathBuf::from("/tmp/demo/.fol/cache"),
        };
        let result = work_list(&workspace);

        assert_eq!(result.command, "work list");
        assert!(result.summary.contains("/tmp/demo/app"));
        assert!(result.summary.contains("/tmp/demo/lib"));
        assert_eq!(result.artifacts.len(), 2);
        assert_eq!(result.artifacts[0].kind, crate::FrontendArtifactKind::PackageRoot);
    }
}
