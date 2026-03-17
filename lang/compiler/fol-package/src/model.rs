use crate::{PackageBuildDefinition, PackageIdentity, PackageMetadata, PackageSourceKind};
use fol_parser::ast::ParsedPackage;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PreparedExportMount {
    pub source_namespace: String,
    pub mounted_namespace_suffix: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedPackage {
    pub identity: PackageIdentity,
    pub metadata: Option<PackageMetadata>,
    pub build: Option<PackageBuildDefinition>,
    pub exports: Vec<PreparedExportMount>,
    pub syntax: ParsedPackage,
}

impl PreparedPackage {
    pub fn new(identity: PackageIdentity, syntax: ParsedPackage) -> Self {
        Self {
            identity,
            metadata: None,
            build: None,
            exports: Vec::new(),
            syntax,
        }
    }

    pub fn with_controls(
        identity: PackageIdentity,
        metadata: PackageMetadata,
        build: PackageBuildDefinition,
        exports: Vec<PreparedExportMount>,
        syntax: ParsedPackage,
    ) -> Self {
        Self {
            identity,
            metadata: Some(metadata),
            build: Some(build),
            exports,
            syntax,
        }
    }

    pub fn package_name(&self) -> &str {
        &self.syntax.package
    }

    pub fn source_kind(&self) -> PackageSourceKind {
        self.identity.source_kind
    }
}

#[cfg(test)]
mod tests {
    use super::PreparedPackage;
    use crate::{
        build::PackageBuildCompatibility, BuildDependency, BuildExport, PackageBuildDefinition,
        PackageConfig, PackageDependencyDecl, PackageDependencySourceKind, PackageIdentity,
        PackageLocator, PackageMetadata, PackageNativeArtifact, PackageNativeArtifactKind,
        PackageSourceKind, PreparedExportMount,
    };
    use fol_parser::ast::{AstParser, ParsedPackage};
    use fol_stream::FileStream;

    fn parse_fixture_package() -> ParsedPackage {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open package fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        parser
            .parse_package(&mut lexer)
            .expect("Package fixture should parse as a package")
    }

    #[test]
    fn package_config_defaults_to_no_external_roots() {
        let config = PackageConfig::default();

        assert_eq!(config.std_root, None);
        assert_eq!(config.package_store_root, None);
        assert_eq!(config.package_cache_root, None);
    }

    #[test]
    fn prepared_package_retains_identity_and_parsed_package_name() {
        let syntax = parse_fixture_package();
        let prepared = PreparedPackage::new(
            PackageIdentity {
                source_kind: PackageSourceKind::Entry,
                canonical_root: "/tmp/fixture".to_string(),
                display_name: "fixture".to_string(),
            },
            syntax,
        );

        assert_eq!(prepared.package_name(), "parser");
        assert_eq!(prepared.source_kind(), PackageSourceKind::Entry);
        assert_eq!(prepared.identity.display_name, "fixture");
        assert!(prepared.metadata.is_none());
        assert!(prepared.build.is_none());
        assert!(prepared.exports.is_empty());
        assert_eq!(prepared.syntax.source_units.len(), 1);
    }

    #[test]
    fn prepared_package_can_carry_metadata_and_build_controls() {
        let syntax = parse_fixture_package();
        let prepared = PreparedPackage::with_controls(
            PackageIdentity {
                source_kind: PackageSourceKind::Package,
                canonical_root: "/tmp/pkg/json".to_string(),
                display_name: "json".to_string(),
            },
            PackageMetadata {
                name: "json".to_string(),
                version: "1.0.0".to_string(),
                kind: Some("lib".to_string()),
                description: None,
                license: None,
                dependencies: vec![PackageDependencyDecl {
                    alias: "core".to_string(),
                    source_kind: PackageDependencySourceKind::PackageStore,
                    target: "core".to_string(),
                }],
            },
            PackageBuildDefinition {
                compatibility: PackageBuildCompatibility {
                    dependencies: vec![BuildDependency {
                        alias: "core".to_string(),
                        locator: PackageLocator::installed_store("core", vec!["core".to_string()]),
                    }],
                    exports: vec![BuildExport {
                        alias: "root".to_string(),
                        relative_path: "src".to_string(),
                    }],
                    native_artifacts: vec![PackageNativeArtifact {
                        alias: "api".to_string(),
                        kind: PackageNativeArtifactKind::Header,
                        relative_path: "include/api.h".to_string(),
                    }],
                },
                entry_point: None,
            },
            vec![PreparedExportMount {
                source_namespace: "json::src".to_string(),
                mounted_namespace_suffix: None,
            }],
            syntax,
        );

        assert_eq!(prepared.metadata.as_ref().map(|meta| meta.name.as_str()), Some("json"));
        assert_eq!(
            prepared
                .build
                .as_ref()
                .map(|build| build.exports().len()),
            Some(1)
        );
        assert_eq!(
            prepared
                .build
                .as_ref()
                .map(|build| build.native_artifacts().len()),
            Some(1)
        );
        assert_eq!(prepared.exports.len(), 1);
    }
}
