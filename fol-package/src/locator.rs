use crate::{PackageError, PackageErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageLocatorKind {
    InstalledStore,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageLocator {
    pub kind: PackageLocatorKind,
    pub raw: String,
    pub path_segments: Vec<String>,
}

pub fn parse_package_locator(raw: &str) -> Result<PackageLocator, PackageError> {
    let trimmed = raw.trim();
    if looks_like_future_git_locator(trimmed) {
        return Err(PackageError::new(
            PackageErrorKind::Unsupported,
            format!(
                "package dependency locator '{}' looks like a future git or remote locator; only installed-store slash paths are supported today",
                raw
            ),
        ));
    }
    let parts = trimmed.split('/').map(str::trim).collect::<Vec<_>>();
    if trimmed.is_empty() || parts.iter().any(|part| part.is_empty()) {
        return Err(PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package dependency locator '{}' must contain non-empty slash-separated segments",
                raw
            ),
        ));
    }

    Ok(PackageLocator {
        kind: PackageLocatorKind::InstalledStore,
        raw: trimmed.to_string(),
        path_segments: parts.into_iter().map(str::to_string).collect(),
    })
}

fn looks_like_future_git_locator(raw: &str) -> bool {
    raw.contains("://")
        || raw.starts_with("git+")
        || raw.starts_with("git@")
        || raw.ends_with(".git")
}

#[cfg(test)]
mod tests {
    use super::{parse_package_locator, PackageLocator, PackageLocatorKind};

    #[test]
    fn package_locator_parses_installed_store_paths() {
        let locator = parse_package_locator("org/tools")
            .expect("Slash-separated package dependency paths should parse as installed-store locators");

        assert_eq!(
            locator,
            PackageLocator {
                kind: PackageLocatorKind::InstalledStore,
                raw: "org/tools".to_string(),
                path_segments: vec!["org".to_string(), "tools".to_string()],
            }
        );
    }

    #[test]
    fn package_locator_rejects_empty_store_segments() {
        let error = parse_package_locator("org//tools")
            .expect_err("Package locators should reject empty slash-separated segments");

        assert_eq!(error.kind(), crate::PackageErrorKind::InvalidInput);
        assert!(
            error
                .to_string()
                .contains("must contain non-empty slash-separated segments"),
            "Invalid locators should explain the accepted slash-separated path form",
        );
    }

    #[test]
    fn package_locator_reports_remote_git_forms_as_explicit_placeholders() {
        let error = parse_package_locator("https://github.com/follang/json.git")
            .expect_err("Remote git-like locators should fail with an explicit placeholder diagnostic");

        assert_eq!(error.kind(), crate::PackageErrorKind::Unsupported);
        assert!(
            error
                .to_string()
                .contains("future git or remote locator"),
            "Remote locators should fail with an explicit future-support diagnostic",
        );
    }
}
