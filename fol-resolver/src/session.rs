use crate::{
    collect, imports,
    manifest::{parse_package_manifest, PackageManifest},
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
    pub manifest: Option<PackageManifest>,
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
        self.load_package_from_root(canonical_root, source_kind, display_name, None)
    }

    pub(crate) fn load_package_from_store(
        &mut self,
        store_root: &Path,
        package_path: &[UsePathSegment],
    ) -> Result<LoadedPackage, ResolverError> {
        let target_root = resolve_directory_path(store_root, package_path);
        let canonical_root = canonical_directory_root(target_root.as_path(), PackageSourceKind::Package)?;
        let manifest_path = canonical_root.join("package.fol");
        if !manifest_path.is_file() {
            return Err(ResolverError::new(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver pkg import target '{}' is missing required package manifest '{}'",
                    canonical_root.display(),
                    manifest_path.display()
                ),
            ));
        }
        let manifest = parse_package_manifest(manifest_path.as_path())?;
        self.load_package_from_root(
            canonical_root,
            PackageSourceKind::Package,
            manifest.name.clone(),
            Some(manifest),
        )
    }

    fn load_package_from_root(
        &mut self,
        canonical_root: PathBuf,
        source_kind: PackageSourceKind,
        display_name: String,
        manifest: Option<PackageManifest>,
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

        let syntax = parse_package_from_directory(canonical_root.as_path(), &display_name, source_kind)?;
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
        let loaded = LoadedPackage {
            identity,
            manifest,
            program,
        };
        self.cache_package(loaded.clone());
        Ok(loaded)
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
                    "resolver pkg import target '{}' has no loadable source files after excluding package.fol/build.fol",
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
        Some("package.fol") | Some("build.fol")
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
            manifest: None,
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
        assert!(loaded.manifest.is_none());
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
    fn session_can_load_installed_pkg_roots_with_required_manifests() {
        let temp_root = unique_temp_root("load_pkg_root");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(
            store_root.join("json/package.fol"),
            concat!(
                "var name: str = \"json\";\n",
                "var version: str = \"1.0.0\";\n",
                "use serde: pkg = {serde};\n",
            ),
        )
        .expect("Should write the package manifest fixture");
        fs::write(store_root.join("json/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the package source fixture");
        fs::write(store_root.join("json/build.fol"), "fun[] build(): int = { return 0; }\n")
            .expect("Should write the package build script fixture");
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
                .all(|unit| !unit.path.ends_with("package.fol") && !unit.path.ends_with("build.fol")),
            "Installed package source loading should exclude package.fol/build.fol from the parsed source set",
        );
        assert_eq!(
            loaded
                .manifest
                .as_ref()
                .expect("Installed package roots should retain parsed manifest metadata")
                .version,
            "1.0.0"
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn session_rejects_pkg_roots_without_required_manifests() {
        let temp_root = unique_temp_root("missing_pkg_manifest");
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
            .expect_err("Session should reject installed package roots without package manifests");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error.to_string().contains("missing required package manifest"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn session_rejects_malformed_pkg_manifests_explicitly() {
        let temp_root = unique_temp_root("malformed_pkg_manifest");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(
            store_root.join("json/package.fol"),
            "var name: str = \"json\";\n",
        )
        .expect("Should write the malformed package manifest fixture");
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
            .expect_err("Session should reject malformed package manifests");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error.to_string().contains("missing required field 'version'"));

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
            store_root.join("core/package.fol"),
            "var name: str = \"core\";\nvar version: str = \"1.0.0\";\n",
        )
        .expect("Should write the transitive dependency manifest");
        fs::write(store_root.join("core/lib.fol"), "var[exp] shared: int = 7;\n")
            .expect("Should write the transitive dependency export");
        fs::write(
            store_root.join("json/package.fol"),
            concat!(
                "var name: str = \"json\";\n",
                "var version: str = \"1.0.0\";\n",
                "use core: pkg = {core};\n",
            ),
        )
        .expect("Should write the direct dependency manifest");
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
}
