use crate::{PackageIdentity, PackageSourceKind};
use fol_parser::ast::ParsedPackage;

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedPackage {
    pub identity: PackageIdentity,
    pub syntax: ParsedPackage,
}

impl PreparedPackage {
    pub fn new(identity: PackageIdentity, syntax: ParsedPackage) -> Self {
        Self { identity, syntax }
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
    use crate::{PackageConfig, PackageIdentity, PackageSourceKind};
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
        assert_eq!(prepared.syntax.source_units.len(), 1);
    }
}
