use crate::{PackageConfig, PackageError, PackageErrorKind, PackageIdentity, PreparedPackage};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct PackageSession {
    config: PackageConfig,
    prepared_packages: BTreeMap<PackageIdentity, PreparedPackage>,
    loading_stack: Vec<PackageIdentity>,
}

impl PackageSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: PackageConfig) -> Self {
        Self {
            config,
            prepared_packages: BTreeMap::new(),
            loading_stack: Vec::new(),
        }
    }

    pub fn config(&self) -> &PackageConfig {
        &self.config
    }

    pub fn cached_package_count(&self) -> usize {
        self.prepared_packages.len()
    }

    #[cfg(test)]
    pub(crate) fn loading_depth(&self) -> usize {
        self.loading_stack.len()
    }

    pub(crate) fn cached_package(&self, identity: &PackageIdentity) -> Option<&PreparedPackage> {
        self.prepared_packages.get(identity)
    }

    pub(crate) fn cache_package(&mut self, package: PreparedPackage) {
        self.prepared_packages
            .insert(package.identity.clone(), package);
    }

    pub(crate) fn begin_loading(&mut self, identity: &PackageIdentity) -> Result<(), PackageError> {
        if self.loading_stack.contains(identity) {
            return Err(self.import_cycle_error(identity));
        }
        self.loading_stack.push(identity.clone());
        Ok(())
    }

    pub(crate) fn finish_loading(&mut self) {
        self.loading_stack.pop();
    }

    fn import_cycle_error(&self, next: &PackageIdentity) -> PackageError {
        let cycle = self
            .loading_stack
            .iter()
            .map(|identity| identity.canonical_root.as_str())
            .chain(std::iter::once(next.canonical_root.as_str()))
            .collect::<Vec<_>>()
            .join(" -> ");
        PackageError::new(
            PackageErrorKind::ImportCycle,
            format!("package import cycle detected while loading package roots: {cycle}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PackageSession;
    use crate::{PackageConfig, PackageIdentity, PackageSourceKind, PreparedPackage};
    use fol_parser::ast::{AstParser, ParsedPackage};
    use fol_stream::FileStream;

    fn parse_fixture_package() -> ParsedPackage {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open package fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        parser
            .parse_package(&mut lexer)
            .expect("Package fixture should parse as a package")
    }

    #[test]
    fn package_session_config_can_be_provided_explicitly() {
        let session = PackageSession::with_config(PackageConfig {
            std_root: Some("/tmp/fol_std".to_string()),
            package_store_root: Some("/tmp/fol_pkg".to_string()),
            package_cache_root: Some("/tmp/fol_cache".to_string()),
        });

        assert_eq!(session.config().std_root.as_deref(), Some("/tmp/fol_std"));
        assert_eq!(
            session.config().package_store_root.as_deref(),
            Some("/tmp/fol_pkg")
        );
        assert_eq!(
            session.config().package_cache_root.as_deref(),
            Some("/tmp/fol_cache")
        );
        assert_eq!(session.cached_package_count(), 0);
        assert_eq!(session.loading_depth(), 0);
    }

    #[test]
    fn package_session_caches_prepared_packages_by_identity() {
        let mut session = PackageSession::new();
        let identity = PackageIdentity {
            source_kind: PackageSourceKind::Local,
            canonical_root: "/tmp/example".to_string(),
            display_name: "example".to_string(),
        };
        session.cache_package(PreparedPackage::new(identity.clone(), parse_fixture_package()));

        assert!(session.cached_package(&identity).is_some());
        assert_eq!(session.cached_package_count(), 1);
    }

    #[test]
    fn package_session_tracks_loading_stack_for_cycle_detection() {
        let identity = PackageIdentity {
            source_kind: PackageSourceKind::Package,
            canonical_root: "/tmp/example".to_string(),
            display_name: "example".to_string(),
        };
        let mut session = PackageSession::new();

        session
            .begin_loading(&identity)
            .expect("First load of a package root should succeed");
        let error = session
            .begin_loading(&identity)
            .expect_err("Repeated active loads of the same package root should report a cycle");

        assert_eq!(error.kind(), crate::PackageErrorKind::ImportCycle);
        assert!(error
            .to_string()
            .contains("package import cycle detected while loading package roots"));
        assert_eq!(session.loading_depth(), 1);

        session.finish_loading();
        assert_eq!(session.loading_depth(), 0);
    }
}
