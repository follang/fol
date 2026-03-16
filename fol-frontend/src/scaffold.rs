use crate::{FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendResult};
use std::path::Path;

pub fn init_current_dir(root: &Path) -> FrontendResult<FrontendCommandResult> {
    Ok(FrontendCommandResult::new("init", "initialized current directory").with_artifact(
        FrontendArtifactSummary::new(
            FrontendArtifactKind::PackageRoot,
            "current-directory",
            Some(root.to_path_buf()),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::init_current_dir;
    use crate::FrontendArtifactKind;
    use std::path::PathBuf;

    #[test]
    fn init_shell_returns_current_directory_summary() {
        let root = PathBuf::from("/tmp/demo");
        let result = init_current_dir(&root).unwrap();

        assert_eq!(result.command, "init");
        assert_eq!(result.summary, "initialized current directory");
        assert_eq!(result.artifacts.len(), 1);
        assert_eq!(result.artifacts[0].kind, FrontendArtifactKind::PackageRoot);
        assert_eq!(result.artifacts[0].path.as_ref(), Some(&root));
    }
}
