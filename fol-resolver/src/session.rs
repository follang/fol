use crate::{
    collect, imports,
    model::ResolvedProgram,
    traverse, ResolverError, ResolverErrorKind, ResolverResult,
};
use fol_parser::ast::ParsedPackage;
use fol_stream::{FileStream, Source, SourceType};
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LoadedPackage {
    pub identity: PackageIdentity,
    pub program: ResolvedProgram,
}

#[derive(Debug, Default)]
pub struct ResolverSession {
    config: ResolverConfig,
    loaded_packages: BTreeMap<PackageIdentity, LoadedPackage>,
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

    pub(crate) fn cached_package(&self, identity: &PackageIdentity) -> Option<&LoadedPackage> {
        self.loaded_packages.get(identity)
    }

    pub(crate) fn cache_package(&mut self, package: LoadedPackage) {
        self.loaded_packages
            .insert(package.identity.clone(), package);
    }

    pub(crate) fn load_package_from_directory(
        &mut self,
        directory: &Path,
        source_kind: PackageSourceKind,
    ) -> Result<LoadedPackage, ResolverError> {
        let canonical_root = std::fs::canonicalize(directory).map_err(|error| {
            ResolverError::new(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver could not canonicalize package root '{}': {}",
                    directory.display(),
                    error
                ),
            )
        })?;
        let display_name = canonical_root
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                ResolverError::new(
                    ResolverErrorKind::InvalidInput,
                    format!(
                        "resolver could not derive a package name from '{}'",
                        canonical_root.display()
                    ),
                )
            })?
            .to_string();
        let identity = PackageIdentity {
            source_kind,
            canonical_root: canonical_root.to_string_lossy().to_string(),
            display_name: display_name.clone(),
        };

        if let Some(cached) = self.cached_package(&identity) {
            return Ok(cached.clone());
        }

        let syntax = parse_package_from_directory(canonical_root.as_path(), &display_name)?;
        let program = self
            .resolve_parsed_package(syntax, Some(identity.clone()))
            .map_err(|errors| {
                errors.into_iter().next().unwrap_or_else(|| {
                    ResolverError::new(
                        ResolverErrorKind::Internal,
                        format!(
                            "resolver failed to load package root '{}'",
                            canonical_root.display()
                        ),
                    )
                })
            })?;
        let loaded = LoadedPackage { identity, program };
        self.cache_package(loaded.clone());
        Ok(loaded)
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

fn parse_package_from_directory(
    root: &Path,
    display_name: &str,
) -> Result<ParsedPackage, ResolverError> {
    let root_str = root.to_str().ok_or_else(|| {
        ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!("package root '{}' is not valid UTF-8", root.display()),
        )
    })?;
    let sources = Source::init_with_package(root_str, SourceType::Folder, display_name).map_err(
        |error| {
            ResolverError::new(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver could not initialize package sources from '{}': {}",
                    root.display(),
                    error
                ),
            )
        },
    )?;
    let mut stream = FileStream::from_sources(sources).map_err(|error| {
        ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "resolver could not read package sources from '{}': {}",
                root.display(),
                error
            ),
        )
    })?;
    let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
    let mut parser = fol_parser::ast::AstParser::new();

    parser.parse_package(&mut lexer).map_err(|errors| {
        let first = errors
            .into_iter()
            .next()
            .expect("parse_package should produce at least one error");
        if let Some(parse_error) = first
            .as_ref()
            .as_any()
            .downcast_ref::<fol_parser::ast::ParseError>()
        {
            ResolverError::with_origin(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver could not parse imported package '{}': {}",
                    root.display(),
                    parse_error
                ),
                fol_parser::ast::SyntaxOrigin {
                    file: parse_error.file(),
                    line: parse_error.line(),
                    column: parse_error.column(),
                    length: parse_error.length(),
                },
            )
        } else {
            ResolverError::new(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver could not parse imported package '{}': {}",
                    root.display(),
                    first
                ),
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{infer_package_root, PackageIdentity, PackageSourceKind, ResolverConfig, ResolverSession};
    use fol_lexer::lexer::stage3::Elements;
    use fol_parser::ast::AstParser;
    use fol_stream::FileStream;
    use std::{fs, path::Path};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn parse_package(path: &str) -> fol_parser::ast::ParsedPackage {
        let mut stream = FileStream::from_folder(path).expect("Should open parser fixture folder");
        let mut lexer = Elements::init(&mut stream);
        let mut parser = AstParser::new();
        parser
            .parse_package(&mut lexer)
            .expect("Fixture folder should parse as a package")
    }

    fn unique_temp_root(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fol_resolver_session_{}_{}_{}",
            label,
            std::process::id(),
            stamp
        ))
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
        session.cache_package(super::LoadedPackage {
            identity: identity.clone(),
            program: super::parse_package_from_directory(
                Path::new("../test/parser/source_units"),
                "source_units",
            )
            .map(|syntax| {
                let mut nested = ResolverSession::new();
                nested
                    .resolve_parsed_package(syntax, None)
                    .expect("Fixture package should resolve")
            })
            .expect("Fixture package should parse"),
        });

        assert!(session.cached_package(&identity).is_some());
        assert_eq!(session.cached_package_count(), 1);
    }

    #[test]
    fn session_can_load_additional_package_roots_from_directories() {
        let temp_root = unique_temp_root("load_package_root");
        fs::create_dir_all(temp_root.join("dep"))
            .expect("Should create a temporary package root fixture");
        fs::write(temp_root.join("dep/main.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the dependency package fixture");
        let mut session = ResolverSession::new();

        let loaded = session
            .load_package_from_directory(&temp_root.join("dep"), PackageSourceKind::Local)
            .expect("Session should load additional package roots from disk");

        assert_eq!(loaded.program.package_name(), "dep");
        assert_eq!(loaded.program.source_units.len(), 1);
        assert_eq!(session.cached_package_count(), 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary session fixture directory should be removable after the test");
    }

    #[test]
    fn session_reuses_cached_packages_for_repeated_canonical_roots() {
        let temp_root = unique_temp_root("load_package_cache");
        fs::create_dir_all(temp_root.join("dep"))
            .expect("Should create a temporary package root fixture");
        fs::write(temp_root.join("dep/main.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the dependency package fixture");
        let mut session = ResolverSession::new();

        let first = session
            .load_package_from_directory(&temp_root.join("dep"), PackageSourceKind::Local)
            .expect("Session should load the package root the first time");
        let second = session
            .load_package_from_directory(&temp_root.join("dep"), PackageSourceKind::Local)
            .expect("Session should reuse the cached package root");

        assert_eq!(first.identity, second.identity);
        assert_eq!(session.cached_package_count(), 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary session fixture directory should be removable after the test");
    }
}
