use crate::{
    build_dependency::DependencyBuildSurfaceSet, build_native::NativeArtifactSet,
    PackageBuildDefinition, PackageIdentity, PackageMetadata, PackageSourceKind,
};
use fol_parser::ast::{ParsedPackage, ParsedSourceUnit, ParsedSourceUnitKind};

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
    pub dependency_surfaces: Option<DependencyBuildSurfaceSet>,
    pub native_surfaces: Option<NativeArtifactSet>,
    pub syntax: ParsedPackage,
}

impl PreparedPackage {
    pub fn new(identity: PackageIdentity, syntax: ParsedPackage) -> Self {
        Self {
            identity,
            metadata: None,
            build: None,
            exports: Vec::new(),
            dependency_surfaces: None,
            native_surfaces: None,
            syntax,
        }
    }

    pub fn with_controls(
        identity: PackageIdentity,
        metadata: PackageMetadata,
        build: PackageBuildDefinition,
        exports: Vec<PreparedExportMount>,
        dependency_surfaces: Option<DependencyBuildSurfaceSet>,
        native_surfaces: Option<NativeArtifactSet>,
        syntax: ParsedPackage,
    ) -> Self {
        Self {
            identity,
            metadata: Some(metadata),
            build: Some(build),
            exports,
            dependency_surfaces,
            native_surfaces,
            syntax,
        }
    }

    pub fn package_name(&self) -> &str {
        &self.syntax.package
    }

    pub fn source_kind(&self) -> PackageSourceKind {
        self.identity.source_kind
    }

    pub fn build_entry_point(&self) -> Option<&crate::build::PackageBuildEntryPoint> {
        self.build.as_ref().and_then(|build| build.entry_point())
    }

    pub fn has_build_entry_point(&self) -> bool {
        self.build_entry_point().is_some()
    }

    pub fn source_units(&self) -> &[ParsedSourceUnit] {
        &self.syntax.source_units
    }

    pub fn build_source_units(&self) -> impl Iterator<Item = &ParsedSourceUnit> {
        self.syntax
            .source_units
            .iter()
            .filter(|unit| unit.kind == ParsedSourceUnitKind::Build)
    }

    pub fn ordinary_source_units(&self) -> impl Iterator<Item = &ParsedSourceUnit> {
        self.syntax
            .source_units
            .iter()
            .filter(|unit| unit.kind == ParsedSourceUnitKind::Ordinary)
    }
}

#[cfg(test)]
mod tests {
    use super::PreparedPackage;
    use crate::{
        build_dependency::DependencyBuildSurfaceSet,
        build_native::{NativeArtifactDefinition, NativeArtifactKind, NativeArtifactSet},
        build::{PackageBuildCompatibility, PackageBuildEntryPoint, PackageBuildEntryPointKind},
        BuildDependency, BuildExport, PackageBuildDefinition, PackageConfig, PackageDependencyDecl,
        PackageDependencySourceKind, PackageIdentity, PackageLocator, PackageMetadata,
        PackageNativeArtifact, PackageNativeArtifactKind, PackageSourceKind, PreparedExportMount,
    };
    use fol_parser::ast::{AstParser, ParsedPackage, ParsedSourceUnitKind};
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
        assert!(prepared.dependency_surfaces.is_none());
        assert!(prepared.native_surfaces.is_none());
        assert_eq!(prepared.syntax.source_units.len(), 1);
        assert_eq!(
            prepared.source_units()[0].kind,
            ParsedSourceUnitKind::Ordinary
        );
    }

    #[test]
    fn prepared_package_can_carry_metadata_and_build_controls() {
        let syntax = parse_fixture_package();
        let mut native_surfaces = NativeArtifactSet::new();
        native_surfaces.add(NativeArtifactDefinition {
            name: "api".to_string(),
            kind: NativeArtifactKind::Header,
            relative_path: "include/api.h".to_string(),
        });
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
                entry_point: Some(PackageBuildEntryPoint {
                    kind: PackageBuildEntryPointKind::BuildFunction,
                    name: "build".to_string(),
                }),
            },
            vec![PreparedExportMount {
                source_namespace: "json::src".to_string(),
                mounted_namespace_suffix: None,
            }],
            Some(DependencyBuildSurfaceSet::new()),
            Some(native_surfaces),
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
        assert!(prepared.dependency_surfaces.is_some());
        assert_eq!(
            prepared.native_surfaces.as_ref().map(|set| set.definitions().len()),
            Some(1)
        );
        assert!(prepared.has_build_entry_point());
        assert_eq!(
            prepared.build_entry_point(),
            Some(&PackageBuildEntryPoint {
                kind: PackageBuildEntryPointKind::BuildFunction,
                name: "build".to_string(),
            })
        );
    }

    #[test]
    fn prepared_package_helpers_split_build_and_ordinary_units() {
        let mut syntax = parse_fixture_package();
        syntax.source_units.push(fol_parser::ast::ParsedSourceUnit {
            path: "build.fol".to_string(),
            package: syntax.package.clone(),
            namespace: syntax.package.clone(),
            kind: ParsedSourceUnitKind::Build,
            items: Vec::new(),
        });
        let prepared = PreparedPackage::new(
            PackageIdentity {
                source_kind: PackageSourceKind::Entry,
                canonical_root: "/tmp/fixture".to_string(),
                display_name: "fixture".to_string(),
            },
            syntax,
        );

        assert_eq!(prepared.build_source_units().count(), 1);
        assert_eq!(prepared.ordinary_source_units().count(), 1);
        assert_eq!(
            prepared
                .build_source_units()
                .next()
                .map(|unit| unit.path.as_str()),
            Some("build.fol")
        );
    }
}
