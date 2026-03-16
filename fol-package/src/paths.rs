use crate::PackageLocator;
use std::path::{Path, PathBuf};

pub fn git_cache_path(root: &Path, locator: &PackageLocator) -> PathBuf {
    let identity = locator
        .normalized_git_identity()
        .expect("git cache paths require git locators");
    let mut path = root.to_path_buf();
    let mut segments = identity.split('/').collect::<Vec<_>>();
    let repo = segments
        .pop()
        .expect("normalized git identities should have a repository segment");
    for segment in segments {
        path.push(segment);
    }
    path.push(format!("{repo}.git"));
    path
}

#[cfg(test)]
mod tests {
    use super::git_cache_path;
    use crate::{parse_package_locator, PackageLocatorKind};
    use std::path::PathBuf;

    #[test]
    fn git_cache_paths_are_derived_from_normalized_locator_identity() {
        let root = PathBuf::from("/tmp/demo/.fol/cache/git");
        let locator = parse_package_locator("git@GitHub.com:bresilla/logtiny.git")
            .expect("git locator should parse");

        assert_eq!(locator.kind, PackageLocatorKind::Git);
        assert_eq!(
            git_cache_path(&root, &locator),
            PathBuf::from("/tmp/demo/.fol/cache/git/github.com/bresilla/logtiny.git")
        );
    }
}
