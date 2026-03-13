use crate::{
    build_definition::{parse_package_build, PackageBuildDefinition},
    collect, imports,
    manifest::{parse_package_metadata, PackageMetadata},
    model::ResolvedProgram,
    traverse, ResolverError, ResolverErrorKind, ResolverResult,
};
use fol_parser::ast::{ParsedPackage, UsePathSegment};
use fol_stream::{FileStream, Source, SourceType};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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
    pub metadata: Option<PackageMetadata>,
    pub build: Option<PackageBuildDefinition>,
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

    #[cfg(test)]
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
            if self.loading_stack.contains(&identity) {
                return Err(vec![self.import_cycle_error(&identity)]);
            }
            self.loading_stack.push(identity);
        }

        let mut program = ResolvedProgram::new(syntax);
        let collected = collect::collect_top_level_symbols(&mut program)
            .and_then(|_| imports::resolve_import_targets(self, &mut program))
            .and_then(|_| traverse::collect_routine_scopes(self, &mut program));

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
        let canonical_root = canonical_directory_root(directory, source_kind)?;
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
        self.load_package_from_root(canonical_root, source_kind, display_name, None, None)
    }

    pub(crate) fn load_package_from_store(
        &mut self,
        store_root: &Path,
        package_path: &[UsePathSegment],
    ) -> Result<LoadedPackage, ResolverError> {
        let target_root = resolve_directory_path(store_root, package_path);
        let canonical_root = canonical_directory_root(target_root.as_path(), PackageSourceKind::Package)?;
        let metadata_path = canonical_root.join("package.yaml");
        let build_path = canonical_root.join("build.fol");
        if !metadata_path.is_file() {
            return Err(ResolverError::new(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver pkg import target '{}' is missing required package metadata '{}'",
                    canonical_root.display(),
                    metadata_path.display()
                ),
            ));
        }
        if !build_path.is_file() {
            return Err(ResolverError::new(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver pkg import target '{}' is missing required package build file '{}'",
                    canonical_root.display(),
                    build_path.display()
                ),
            ));
        }
        let metadata = parse_package_metadata(metadata_path.as_path())?;
        let build = parse_package_build(build_path.as_path())?;
        self.load_package_from_root(
            canonical_root,
            PackageSourceKind::Package,
            metadata.name.clone(),
            Some(metadata),
            Some(build),
        )
    }

    fn load_package_from_root(
        &mut self,
        canonical_root: PathBuf,
        source_kind: PackageSourceKind,
        display_name: String,
        metadata: Option<PackageMetadata>,
        build: Option<PackageBuildDefinition>,
    ) -> Result<LoadedPackage, ResolverError> {
        let identity = PackageIdentity {
            source_kind,
            canonical_root: canonical_root.to_string_lossy().to_string(),
            display_name: display_name.clone(),
        };

        if self.loading_stack.contains(&identity) {
            return Err(self.import_cycle_error(&identity));
        }

        if let Some(cached) = self.cached_package(&identity) {
            return Ok(cached.clone());
        }

        self.loading_stack.push(identity.clone());

        let loaded_result = (|| {
            if source_kind == PackageSourceKind::Package {
                if let Some(build) = build.as_ref() {
                    self.preload_build_dependencies(build)?;
                }
            }

            let syntax =
                parse_package_from_directory(canonical_root.as_path(), &display_name, source_kind)?;
            let program = self.resolve_parsed_package(syntax, None).map_err(|errors| {
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
            Ok(LoadedPackage {
                identity,
                metadata,
                build,
                program,
            })
        })();

        self.loading_stack.pop();

        let loaded = loaded_result?;
        self.cache_package(loaded.clone());
        Ok(loaded)
    }

    fn preload_build_dependencies(
        &mut self,
        build: &PackageBuildDefinition,
    ) -> Result<(), ResolverError> {
        let store_root = self
            .config()
            .package_store_root
            .clone()
            .ok_or_else(|| {
                ResolverError::new(
                    ResolverErrorKind::InvalidInput,
                    "resolver package loading requires an explicit package store root",
                )
            })?;

        for dependency in &build.dependencies {
            let path_segments = build_dependency_path_segments(&dependency.package_path)?;
            self.load_package_from_store(Path::new(&store_root), &path_segments)?;
        }

        Ok(())
    }

    fn import_cycle_error(&self, next: &PackageIdentity) -> ResolverError {
        let cycle = self
            .loading_stack
            .iter()
            .map(|identity| identity.canonical_root.as_str())
            .chain(std::iter::once(next.canonical_root.as_str()))
            .collect::<Vec<_>>()
            .join(" -> ");
        ResolverError::new(
            ResolverErrorKind::ImportCycle,
            format!("import cycle detected while loading package roots: {cycle}"),
        )
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

fn canonical_directory_root(
    directory: &Path,
    source_kind: PackageSourceKind,
) -> Result<PathBuf, ResolverError> {
    let metadata = std::fs::metadata(directory).map_err(|error| {
        let label = import_source_label(source_kind);
        if error.kind() == std::io::ErrorKind::NotFound {
            ResolverError::new(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver {label} import target '{}' does not exist",
                    directory.display(),
                ),
            )
        } else {
            ResolverError::new(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver could not inspect {label} import target '{}': {}",
                    directory.display(),
                    error
                ),
            )
        }
    })?;

    if metadata.is_file() {
        let label = import_source_label(source_kind);
        return Err(ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "resolver {label} import target '{}' must point to a directory, not a file",
                directory.display(),
            ),
        ));
    }

    if !metadata.is_dir() {
        let label = import_source_label(source_kind);
        return Err(ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "resolver {label} import target '{}' must point to a directory",
                directory.display(),
            ),
        ));
    }

    std::fs::canonicalize(directory).map_err(|error| {
        ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "resolver could not canonicalize {} import target '{}': {}",
                import_source_label(source_kind),
                directory.display(),
                error
            ),
        )
    })
}

fn import_source_label(source_kind: PackageSourceKind) -> &'static str {
    match source_kind {
        PackageSourceKind::Local => "loc",
        PackageSourceKind::Standard => "std",
        PackageSourceKind::Package => "pkg",
        PackageSourceKind::Entry => "entry",
    }
}

fn resolve_directory_path(source_dir: &Path, path_segments: &[UsePathSegment]) -> PathBuf {
    let mut relative = PathBuf::new();
    for segment in path_segments {
        relative.push(&segment.spelling);
    }

    if relative.is_absolute() {
        relative
    } else {
        source_dir.join(relative)
    }
}

fn build_dependency_path_segments(
    package_path: &str,
) -> Result<Vec<UsePathSegment>, ResolverError> {
    let parts = package_path
        .split('/')
        .map(str::trim)
        .collect::<Vec<_>>();
    if parts.is_empty() || parts.iter().any(|part| part.is_empty()) {
        return Err(ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "resolver package build dependency path '{}' must contain non-empty slash-separated segments",
                package_path
            ),
        ));
    }

    Ok(parts
        .into_iter()
        .enumerate()
        .map(|(index, part)| UsePathSegment {
            separator: (index > 0).then_some(fol_parser::ast::UsePathSeparator::Slash),
            spelling: part.to_string(),
        })
        .collect())
}

fn parse_package_from_directory(
    root: &Path,
    display_name: &str,
    source_kind: PackageSourceKind,
) -> Result<ParsedPackage, ResolverError> {
    let root_str = root.to_str().ok_or_else(|| {
        ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!("package root '{}' is not valid UTF-8", root.display()),
        )
    })?;
    let mut sources = Source::init_with_package(root_str, SourceType::Folder, display_name).map_err(
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
    if source_kind == PackageSourceKind::Package {
        sources.retain(|source| !is_package_control_file(root, source));
        if sources.is_empty() {
            return Err(ResolverError::new(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver pkg import target '{}' has no loadable source files after excluding package control files",
                    root.display()
                ),
            ));
        }
    }
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

fn is_package_control_file(root: &Path, source: &Source) -> bool {
    let source_path = Path::new(&source.path);
    let Some(parent) = source_path.parent() else {
        return false;
    };
    if parent != root {
        return false;
    }
    matches!(
        source_path.file_name().and_then(|name| name.to_str()),
        Some("package.yaml") | Some("package.fol") | Some("build.fol")
    )
}

#[cfg(test)]
mod tests {
    use super::{
        infer_package_root, PackageIdentity, PackageSourceKind, ResolverConfig, ResolverSession,
    };
    use crate::ResolverErrorKind;
    use fol_parser::ast::UsePathSegment;
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
            metadata: None,
            build: None,
            program: super::parse_package_from_directory(
                Path::new("../test/parser/source_units"),
                "source_units",
                PackageSourceKind::Local,
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
        assert!(loaded.metadata.is_none());
        assert!(loaded.build.is_none());
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

    #[test]
    fn session_reports_explicit_import_cycles_with_participating_roots() {
        let temp_root = unique_temp_root("import_cycle");
        fs::create_dir_all(temp_root.join("dep"))
            .expect("Should create a temporary package root fixture");
        fs::write(temp_root.join("dep/main.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the dependency package fixture");
        let canonical_root = std::fs::canonicalize(temp_root.join("dep"))
            .expect("Temporary dependency root should canonicalize");
        let identity = PackageIdentity {
            source_kind: PackageSourceKind::Local,
            canonical_root: canonical_root.to_string_lossy().to_string(),
            display_name: "dep".to_string(),
        };
        let mut session = ResolverSession::new();
        session.loading_stack.push(identity.clone());

        let error = session
            .load_package_from_directory(canonical_root.as_path(), PackageSourceKind::Local)
            .expect_err("Session should reject canonical package roots already in the load stack");

        assert_eq!(error.kind(), ResolverErrorKind::ImportCycle);
        assert!(
            error
                .to_string()
                .contains("import cycle detected while loading package roots"),
            "Import cycle diagnostics should explain the active loading cycle",
        );
        assert!(
            error.to_string().contains(&identity.canonical_root),
            "Import cycle diagnostics should list the participating canonical roots",
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary session fixture directory should be removable after the test");
    }

    #[test]
    fn session_can_load_installed_pkg_roots_with_required_metadata_and_build_files() {
        let temp_root = unique_temp_root("load_pkg_root");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(
            store_root.join("json/package.yaml"),
            concat!("name: json\n", "version: 1.0.0\n", "kind: lib\n"),
        )
        .expect("Should write the package metadata fixture");
        fs::write(store_root.join("json/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the package source fixture");
        fs::write(store_root.join("json/build.fol"), "def root: loc = \"lib\";\n")
            .expect("Should write the package build fixture");
        let mut session = ResolverSession::new();

        let loaded = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect("Session should load installed package roots from the package store");

        assert_eq!(loaded.identity.source_kind, PackageSourceKind::Package);
        assert_eq!(loaded.identity.display_name, "json");
        assert_eq!(loaded.program.package_name(), "json");
        assert_eq!(loaded.program.source_units.len(), 1);
        assert!(
            loaded
                .program
                .source_units
                .iter()
                .all(|unit| {
                    !unit.path.ends_with("package.yaml")
                        && !unit.path.ends_with("build.fol")
                }),
            "Installed package source loading should exclude package control files from the parsed source set",
        );
        assert_eq!(
            loaded
                .metadata
                .as_ref()
                .expect("Installed package roots should retain parsed package metadata")
                .version,
            "1.0.0"
        );
        assert_eq!(
            loaded
                .build
                .as_ref()
                .expect("Installed package roots should retain parsed build definitions")
                .exports
                .len(),
            1
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn session_rejects_pkg_roots_without_required_metadata() {
        let temp_root = unique_temp_root("missing_pkg_metadata");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(store_root.join("json/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the package source fixture");
        let mut session = ResolverSession::new();

        let error = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect_err("Session should reject installed package roots without package metadata");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error.to_string().contains("missing required package metadata"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn session_rejects_pkg_roots_without_required_build_files() {
        let temp_root = unique_temp_root("missing_pkg_build");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(store_root.join("json/package.yaml"), "name: json\nversion: 1.0.0\n")
            .expect("Should write the package metadata fixture");
        fs::write(store_root.join("json/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the package source fixture");
        let mut session = ResolverSession::new();

        let error = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect_err("Session should reject installed package roots without build files");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error.to_string().contains("missing required package build file"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn session_ignores_package_fol_when_package_yaml_is_present() {
        let temp_root = unique_temp_root("ignored_package_fol");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(store_root.join("json/package.yaml"), "name: json\nversion: 1.0.0\n")
            .expect("Should write the package metadata fixture");
        fs::write(
            store_root.join("json/package.fol"),
            "var name: str = \"json\";\nvar version: str = \"1.0.0\";\n",
        )
        .expect("Should write the ignored package.fol fixture");
        fs::write(store_root.join("json/build.fol"), "def root: loc = \"lib\";\n")
            .expect("Should write the package build fixture");
        fs::write(store_root.join("json/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the package source fixture");
        let mut session = ResolverSession::new();

        let loaded = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect("Session should ignore package.fol when package.yaml is present");

        assert_eq!(loaded.identity.display_name, "json");
        assert_eq!(loaded.program.package_name(), "json");

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn session_package_fol_only_roots_still_fail_missing_metadata() {
        let temp_root = unique_temp_root("package_fol_only");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(
            store_root.join("json/package.fol"),
            "var name: str = \"json\";\nvar version: str = \"1.0.0\";\n",
        )
        .expect("Should write the ignored package.fol fixture");
        fs::write(store_root.join("json/build.fol"), "def root: loc = \"lib\";\n")
            .expect("Should write the package build fixture");
        let mut session = ResolverSession::new();

        let error = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect_err("Session should still require package.yaml even if package.fol exists");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error.to_string().contains("missing required package metadata"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn session_rejects_malformed_pkg_metadata_explicitly() {
        let temp_root = unique_temp_root("malformed_pkg_metadata");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(
            store_root.join("json/package.yaml"),
            "name json\n",
        )
        .expect("Should write the malformed package metadata fixture");
        fs::write(store_root.join("json/build.fol"), "def root: loc = \"lib\";\n")
            .expect("Should write the package build fixture");
        fs::write(store_root.join("json/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the package source fixture");
        let mut session = ResolverSession::new();

        let error = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect_err("Session should reject malformed package metadata");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error.to_string().contains("must follow 'key: value' form"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn session_rejects_pkg_roots_with_only_control_files_after_exclusion() {
        let temp_root = unique_temp_root("pkg_control_only");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(store_root.join("json/package.yaml"), "name: json\nversion: 1.0.0\n")
            .expect("Should write the package metadata fixture");
        fs::write(store_root.join("json/build.fol"), "def root: loc = \"src\";\n")
            .expect("Should write the package build fixture");
        let mut session = ResolverSession::new();

        let error = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect_err("Session should reject pkg roots whose control files are the only files present");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error
            .to_string()
            .contains("has no loadable source files after excluding package control files"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn session_recursively_loads_transitive_pkg_dependencies_from_store() {
        let temp_root = unique_temp_root("transitive_pkg_graph");
        let store_root = temp_root.join("store");
        let app_root = temp_root.join("app");
        fs::create_dir_all(store_root.join("core"))
            .expect("Should create the transitive dependency root fixture");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create the direct dependency root fixture");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing app fixture directory");
        fs::write(
            store_root.join("core/package.yaml"),
            "name: core\nversion: 1.0.0\n",
        )
        .expect("Should write the transitive dependency metadata");
        fs::write(store_root.join("core/build.fol"), "def root: loc = \"lib\";\n")
            .expect("Should write the transitive dependency build fixture");
        fs::write(store_root.join("core/lib.fol"), "var[exp] shared: int = 7;\n")
            .expect("Should write the transitive dependency export");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the direct dependency metadata");
        fs::write(
            store_root.join("json/build.fol"),
            "def core: pkg = \"core\";\ndef root: loc = \"lib\";\n",
        )
        .expect("Should write the direct dependency build fixture");
        fs::write(
            store_root.join("json/lib.fol"),
            "use core: pkg = {core};\nvar[exp] answer: int = shared;\n",
        )
        .expect("Should write the direct dependency source");
        fs::write(
            app_root.join("main.fol"),
            "use json: pkg = {json};\nfun[] main(): int = {\n    return answer;\n}\n",
        )
        .expect("Should write the importing app source");
        let parsed = parse_package(
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        );
        let mut session = ResolverSession::with_config(ResolverConfig {
            std_root: None,
            package_store_root: Some(
                store_root
                    .to_str()
                    .expect("Temporary package-store fixture path should be valid UTF-8")
                    .to_string(),
            ),
        });

        session
            .resolve_package(parsed)
            .expect("Transitive pkg dependencies should resolve through the shared session");

        assert_eq!(
            session.cached_package_count(),
            2,
            "Resolving one direct pkg import with one transitive pkg dependency should cache both package roots",
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary transitive package graph fixture should be removable after the test");
    }

    #[test]
    fn session_preloads_pkg_dependencies_from_build_definitions() {
        let temp_root = unique_temp_root("build_pkg_preload");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("core"))
            .expect("Should create the dependency root fixture");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create the dependent package root fixture");
        fs::write(
            store_root.join("core/package.yaml"),
            "name: core\nversion: 1.0.0\n",
        )
        .expect("Should write the dependency metadata");
        fs::write(store_root.join("core/build.fol"), "def root: loc = \"lib\";\n")
            .expect("Should write the dependency build fixture");
        fs::write(store_root.join("core/lib.fol"), "var[exp] shared: int = 7;\n")
            .expect("Should write the dependency source fixture");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the dependent package metadata");
        fs::write(
            store_root.join("json/build.fol"),
            "def core: pkg = \"core\";\ndef root: loc = \"lib\";\n",
        )
        .expect("Should write the dependent package build fixture");
        fs::write(store_root.join("json/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the dependent package source fixture");
        let mut session = ResolverSession::with_config(ResolverConfig {
            std_root: None,
            package_store_root: Some(
                store_root
                    .to_str()
                    .expect("Temporary package-store fixture path should be valid UTF-8")
                    .to_string(),
            ),
        });

        let loaded = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect("Session should load build-declared pkg dependencies eagerly");

        assert_eq!(loaded.identity.display_name, "json");
        assert_eq!(loaded.build.as_ref().map(|build| build.dependencies.len()), Some(1));
        assert_eq!(
            session.cached_package_count(),
            2,
            "Loading a package root should also cache any build-declared pkg dependencies",
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build-preload fixture directory should be removable after the test");
    }

    #[test]
    fn session_reuses_cached_shared_pkg_dependencies_across_multiple_dependents() {
        let temp_root = unique_temp_root("shared_pkg_graph");
        let store_root = temp_root.join("store");
        let app_root = temp_root.join("app");
        fs::create_dir_all(store_root.join("core"))
            .expect("Should create the shared dependency root fixture");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create the first direct dependency root fixture");
        fs::create_dir_all(store_root.join("xml"))
            .expect("Should create the second direct dependency root fixture");
        fs::create_dir_all(&app_root)
            .expect("Should create the importing app fixture directory");
        fs::write(
            store_root.join("core/package.yaml"),
            "name: core\nversion: 1.0.0\n",
        )
        .expect("Should write the shared dependency metadata");
        fs::write(store_root.join("core/build.fol"), "def root: loc = \"lib\";\n")
            .expect("Should write the shared dependency build fixture");
        fs::write(store_root.join("core/lib.fol"), "var[exp] shared: int = 7;\n")
            .expect("Should write the shared dependency export");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the first direct dependency metadata");
        fs::write(
            store_root.join("json/build.fol"),
            "def core: pkg = \"core\";\ndef root: loc = \"lib\";\n",
        )
        .expect("Should write the first direct dependency build fixture");
        fs::write(
            store_root.join("json/lib.fol"),
            "use core: pkg = {core};\nvar[exp] left: int = shared;\n",
        )
        .expect("Should write the first direct dependency source");
        fs::write(
            store_root.join("xml/package.yaml"),
            "name: xml\nversion: 1.0.0\n",
        )
        .expect("Should write the second direct dependency metadata");
        fs::write(
            store_root.join("xml/build.fol"),
            "def core: pkg = \"core\";\ndef root: loc = \"lib\";\n",
        )
        .expect("Should write the second direct dependency build fixture");
        fs::write(
            store_root.join("xml/lib.fol"),
            "use core: pkg = {core};\nvar[exp] right: int = shared;\n",
        )
        .expect("Should write the second direct dependency source");
        fs::write(
            app_root.join("main.fol"),
            concat!(
                "use json: pkg = {json};\n",
                "use xml: pkg = {xml};\n",
                "fun[] main(): int = {\n",
                "    return left + right;\n",
                "}\n",
            ),
        )
        .expect("Should write the importing app source");
        let parsed = parse_package(
            app_root
                .to_str()
                .expect("Temporary app fixture path should be valid UTF-8"),
        );
        let mut session = ResolverSession::with_config(ResolverConfig {
            std_root: None,
            package_store_root: Some(
                store_root
                    .to_str()
                    .expect("Temporary package-store fixture path should be valid UTF-8")
                    .to_string(),
            ),
        });

        session
            .resolve_package(parsed)
            .expect("Shared pkg dependencies should resolve through one cached session");

        assert_eq!(
            session.cached_package_count(),
            3,
            "Two direct pkg imports sharing one transitive dependency should cache json, xml, and core once each",
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary shared package graph fixture should be removable after the test");
    }
}
