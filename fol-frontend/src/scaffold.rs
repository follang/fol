use crate::{FrontendArtifactKind, FrontendArtifactSummary, FrontendCommandResult, FrontendResult};
use std::fs;
use std::path::Path;

pub fn init_current_dir(root: &Path) -> FrontendResult<FrontendCommandResult> {
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/main.fol"), "")?;
    fs::write(root.join("package.yaml"), "")?;
    fs::write(root.join("build.fol"), "")?;

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
    use std::{fs, path::PathBuf};

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

    #[test]
    fn init_shell_creates_package_scaffold_in_current_directory() {
        let root = std::env::temp_dir().join(format!("fol_frontend_init_pkg_{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();

        init_current_dir(&root).unwrap();

        assert!(root.join("src").is_dir());
        assert!(root.join("src/main.fol").is_file());
        assert!(root.join("package.yaml").is_file());
        assert!(root.join("build.fol").is_file());

        fs::remove_dir_all(root).ok();
    }
}
