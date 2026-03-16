use crate::{git_cache_path, git_store_path, PackageError, PackageErrorKind, PackageLocator};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageGitMaterialization {
    pub locator: PackageLocator,
    pub selected_revision: String,
    pub cache_root: PathBuf,
    pub store_root: PathBuf,
}

impl PackageGitMaterialization {
    pub fn new(
        cache_base_root: &Path,
        store_base_root: &Path,
        locator: PackageLocator,
        selected_revision: impl Into<String>,
    ) -> Self {
        let selected_revision = selected_revision.into();
        Self {
            cache_root: git_cache_path(cache_base_root, &locator),
            store_root: git_store_path(store_base_root, &locator, &selected_revision),
            locator,
            selected_revision,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageGitSourceSession {
    cache_base_root: PathBuf,
    store_base_root: PathBuf,
}

impl PackageGitSourceSession {
    pub fn new(cache_base_root: impl Into<PathBuf>, store_base_root: impl Into<PathBuf>) -> Self {
        Self {
            cache_base_root: cache_base_root.into(),
            store_base_root: store_base_root.into(),
        }
    }

    pub fn cache_base_root(&self) -> &Path {
        &self.cache_base_root
    }

    pub fn store_base_root(&self) -> &Path {
        &self.store_base_root
    }

    pub fn plan_materialization(
        &self,
        locator: &PackageLocator,
        selected_revision: impl Into<String>,
    ) -> Result<PackageGitMaterialization, PackageError> {
        if locator.git.is_none() {
            return Err(PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "git materialization requires a git locator, but '{}' is not remote git",
                    locator.raw
                ),
            ));
        }

        Ok(PackageGitMaterialization::new(
            &self.cache_base_root,
            &self.store_base_root,
            locator.clone(),
            selected_revision,
        ))
    }
}

pub fn wrap_git_failure(
    action: &str,
    locator: &PackageLocator,
    detail: impl Into<String>,
) -> PackageError {
    PackageError::new(
        PackageErrorKind::InvalidInput,
        format!(
            "git dependency '{}' failed while trying to {action}: {}",
            locator.raw,
            detail.into()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{wrap_git_failure, PackageGitSourceSession};
    use crate::parse_package_locator;
    use std::path::PathBuf;

    #[test]
    fn git_source_session_plans_separate_cache_and_store_roots() {
        let session = PackageGitSourceSession::new("/tmp/demo/.fol/cache/git", "/tmp/demo/.fol/pkg/git");
        let locator = parse_package_locator("https://github.com/bresilla/logtiny.git?tag=v1")
            .expect("git locator should parse");

        let materialization = session
            .plan_materialization(&locator, "abc123")
            .expect("git materialization plan should succeed");

        assert_eq!(
            materialization.cache_root,
            PathBuf::from("/tmp/demo/.fol/cache/git/github.com/bresilla/logtiny.git")
        );
        assert_eq!(
            materialization.store_root,
            PathBuf::from("/tmp/demo/.fol/pkg/git/github.com/bresilla/logtiny/rev-abc123")
        );
        assert_eq!(materialization.selected_revision, "abc123");
    }

    #[test]
    fn git_source_session_rejects_non_git_locators() {
        let session = PackageGitSourceSession::new("/tmp/demo/.fol/cache/git", "/tmp/demo/.fol/pkg/git");
        let locator = parse_package_locator("org/core").expect("store locator should parse");

        let error = session
            .plan_materialization(&locator, "abc123")
            .expect_err("non-git locators should be rejected");

        assert!(error
            .message()
            .contains("requires a git locator"));
    }

    #[test]
    fn wrapped_git_failures_keep_locator_context() {
        let locator = parse_package_locator("git@github.com:bresilla/logtiny.git")
            .expect("git locator should parse");

        let error = wrap_git_failure("fetch", &locator, "network timeout");

        assert!(error
            .message()
            .contains("git dependency 'git@github.com:bresilla/logtiny.git' failed while trying to fetch"));
        assert!(error.message().contains("network timeout"));
    }
}
