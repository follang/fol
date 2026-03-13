use crate::{
    collect, imports,
    model::ResolvedProgram,
    traverse, ResolverError, ResolverErrorKind, ResolverResult,
};
use fol_parser::ast::ParsedPackage;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolverConfig {
    pub std_root: Option<String>,
    pub package_store_root: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageSourceKind {
    Entry,
    Local,
    Standard,
    Package,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageIdentity {
    pub source_kind: PackageSourceKind,
    pub canonical_root: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackageSurface {
    pub identity: PackageIdentity,
}

#[derive(Debug, Default)]
pub struct ResolverSession {
    config: ResolverConfig,
    loaded_packages: BTreeMap<PackageIdentity, PackageSurface>,
    loading_stack: Vec<PackageIdentity>,
}

impl ResolverSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: ResolverConfig) -> Self {
        Self {
            config,
            loaded_packages: BTreeMap::new(),
            loading_stack: Vec::new(),
        }
    }

    pub fn config(&self) -> &ResolverConfig {
        &self.config
    }

    pub fn cached_package_count(&self) -> usize {
        self.loaded_packages.len()
    }

    pub(crate) fn loading_depth(&self) -> usize {
        self.loading_stack.len()
    }

    pub fn resolve_package(&mut self, syntax: ParsedPackage) -> ResolverResult<ResolvedProgram> {
        self.resolve_entry_package(syntax)
    }

    pub(crate) fn resolve_entry_package(
        &mut self,
        syntax: ParsedPackage,
    ) -> ResolverResult<ResolvedProgram> {
        let inferred_root = infer_package_root(&syntax).map_err(|error| vec![error])?;
        self.resolve_parsed_package(
            syntax,
            Some(PackageIdentity {
                source_kind: PackageSourceKind::Entry,
                display_name: inferred_root
                    .file_name()
                    .and_then(|name| name.to_str())
                    .filter(|name| !name.is_empty())
                    .unwrap_or("root")
                    .to_string(),
                canonical_root: inferred_root.to_string_lossy().to_string(),
            }),
        )
    }

    pub(crate) fn resolve_parsed_package(
        &mut self,
        syntax: ParsedPackage,
        current_identity: Option<PackageIdentity>,
    ) -> ResolverResult<ResolvedProgram> {
        if let Some(identity) = current_identity {
            self.loading_stack.push(identity);
        }

        let mut program = ResolvedProgram::new(syntax);
        let collected = collect::collect_top_level_symbols(&mut program)
            .and_then(|_| imports::resolve_import_targets(&mut program))
            .and_then(|_| traverse::collect_routine_scopes(&mut program));

        if !self.loading_stack.is_empty() {
            self.loading_stack.pop();
        }

        collected.map(|_| program)
    }

    pub(crate) fn cached_surface(&self, identity: &PackageIdentity) -> Option<&PackageSurface> {
        self.loaded_packages.get(identity)
    }

    pub(crate) fn cache_surface(&mut self, surface: PackageSurface) {
        self.loaded_packages
            .insert(surface.identity.clone(), surface);
    }
}

pub(crate) fn infer_package_root(syntax: &ParsedPackage) -> Result<std::path::PathBuf, ResolverError> {
    let mut parents = syntax
        .source_units
        .iter()
        .filter_map(|unit| Path::new(&unit.path).parent().map(Path::to_path_buf));

    let Some(mut common_root) = parents.next() else {
        return Err(ResolverError::new(
            ResolverErrorKind::InvalidInput,
            "resolver requires at least one parsed source unit",
        ));
    };

    for parent in parents {
        while !parent.starts_with(&common_root) {
            if !common_root.pop() {
                return Err(ResolverError::new(
                    ResolverErrorKind::Internal,
                    "resolver could not infer a common package root from parsed source paths",
                ));
            }
        }
    }

    Ok(common_root)
}

#[cfg(test)]
mod tests {
    use super::{infer_package_root, PackageIdentity, PackageSourceKind, ResolverConfig, ResolverSession};
    use fol_lexer::lexer::stage3::Elements;
    use fol_parser::ast::AstParser;
    use fol_stream::FileStream;

    fn parse_package(path: &str) -> fol_parser::ast::ParsedPackage {
        let mut stream = FileStream::from_folder(path).expect("Should open parser fixture folder");
        let mut lexer = Elements::init(&mut stream);
        let mut parser = AstParser::new();
        parser
            .parse_package(&mut lexer)
            .expect("Fixture folder should parse as a package")
    }

    #[test]
    fn session_config_can_be_provided_explicitly() {
        let session = ResolverSession::with_config(ResolverConfig {
            std_root: Some("/tmp/fol_std".to_string()),
            package_store_root: Some("/tmp/fol_pkg".to_string()),
        });

        assert_eq!(session.config().std_root.as_deref(), Some("/tmp/fol_std"));
        assert_eq!(
            session.config().package_store_root.as_deref(),
            Some("/tmp/fol_pkg")
        );
        assert_eq!(session.cached_package_count(), 0);
        assert_eq!(session.loading_depth(), 0);
    }

    #[test]
    fn inferred_package_root_uses_common_parent_of_parsed_source_units() {
        let parsed = parse_package("../test/parser/source_units");
        let inferred = infer_package_root(&parsed).expect("Should infer a common package root");

        assert!(
            inferred.ends_with("source_units"),
            "Expected inferred package root to end with the parsed folder name, got {:?}",
            inferred
        );
    }

    #[test]
    fn session_cache_keys_track_source_kind_and_canonical_root() {
        let mut session = ResolverSession::new();
        let identity = PackageIdentity {
            source_kind: PackageSourceKind::Local,
            canonical_root: "/tmp/example".to_string(),
            display_name: "example".to_string(),
        };
        session.cache_surface(super::PackageSurface {
            identity: identity.clone(),
        });

        assert!(session.cached_surface(&identity).is_some());
        assert_eq!(session.cached_package_count(), 1);
    }
}
