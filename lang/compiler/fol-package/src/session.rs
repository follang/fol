use crate::{
    PackageBuildDefinition, PackageConfig, PackageError, PackageErrorKind, PackageIdentity,
    PackageLocator, PackageSourceKind, PreparedExportMount, PreparedPackage,
};
use fol_lexer::lexer::stage3::Elements;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use fol_parser::ast::{AstParser, ParsedPackage, UsePathSegment};
use fol_stream::{FileStream, Source, SourceType};

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

    pub fn prepare_entry_package(
        &self,
        syntax: ParsedPackage,
    ) -> Result<PreparedPackage, PackageError> {
        let inferred_root = infer_package_root(&syntax)?;
        let display_name = inferred_root
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("root")
            .to_string();
        Ok(PreparedPackage::new(
            PackageIdentity {
                source_kind: PackageSourceKind::Entry,
                canonical_root: inferred_root.to_string_lossy().to_string(),
                display_name,
            },
            syntax,
        ))
    }

    pub fn load_directory_package(
        &mut self,
        directory: &Path,
        source_kind: PackageSourceKind,
    ) -> Result<PreparedPackage, PackageError> {
        let canonical_root = canonical_directory_root(directory, source_kind)?;
        reject_formal_package_roots_for_directory_imports(canonical_root.as_path(), source_kind)?;
        let display_name = canonical_root
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .ok_or_else(|| {
                PackageError::new(
                    PackageErrorKind::InvalidInput,
                    format!(
                        "package loader could not derive a package name from '{}'",
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

        self.begin_loading(&identity)?;

        if let Some(cached) = self.cached_package(&identity).cloned() {
            self.finish_loading();
            return Ok(cached);
        }

        let prepared_result = parse_directory_package_syntax(
            canonical_root.as_path(),
            &display_name,
            source_kind,
        )
        .map(|syntax| PreparedPackage::new(identity.clone(), syntax));

        self.finish_loading();

        let prepared = prepared_result?;
        self.cache_package(prepared.clone());
        Ok(prepared)
    }

    pub fn load_package_from_store(
        &mut self,
        store_root: &Path,
        path_segments: &[UsePathSegment],
    ) -> Result<PreparedPackage, PackageError> {
        let target_root = resolve_directory_path(store_root, path_segments);
        let canonical_root =
            canonical_directory_root(target_root.as_path(), PackageSourceKind::Package)?;
        let canonical_root_str = canonical_root.to_string_lossy().to_string();
        if let Some(cached) =
            self.cached_package_by_root(PackageSourceKind::Package, &canonical_root_str)
        {
            return Ok(cached);
        }
        let metadata_path = canonical_root.join("package.yaml");
        let build_path = canonical_root.join("build.fol");
        if !metadata_path.is_file() {
            return Err(PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package pkg import target '{}' is missing required package metadata '{}'",
                    canonical_root.display(),
                    metadata_path.display()
                ),
            ));
        }
        if !build_path.is_file() {
            return Err(PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package pkg import target '{}' is missing required package build file '{}'",
                    canonical_root.display(),
                    build_path.display()
                ),
            ));
        }

        self.load_formal_package_root(
            canonical_root.as_path(),
            canonical_root_str,
            PackageSourceKind::Package,
            Some(store_root),
        )
    }

    pub fn load_materialized_package(
        &mut self,
        package_root: &Path,
    ) -> Result<PreparedPackage, PackageError> {
        let canonical_root =
            canonical_directory_root(package_root, PackageSourceKind::Package)?;
        let canonical_root_str = canonical_root.to_string_lossy().to_string();
        self.load_formal_package_root(
            canonical_root.as_path(),
            canonical_root_str,
            PackageSourceKind::Package,
            None,
        )
    }

    #[cfg(test)]
    pub(crate) fn loading_depth(&self) -> usize {
        self.loading_stack.len()
    }

    pub(crate) fn cached_package(&self, identity: &PackageIdentity) -> Option<&PreparedPackage> {
        self.prepared_packages.get(identity)
    }

    fn cached_package_by_root(
        &self,
        source_kind: PackageSourceKind,
        canonical_root: &str,
    ) -> Option<PreparedPackage> {
        self.prepared_packages
            .iter()
            .find_map(|(identity, package)| {
                (identity.source_kind == source_kind && identity.canonical_root == canonical_root)
                    .then(|| package.clone())
            })
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

    fn load_formal_package_root(
        &mut self,
        canonical_root: &Path,
        canonical_root_str: String,
        source_kind: PackageSourceKind,
        store_root: Option<&Path>,
    ) -> Result<PreparedPackage, PackageError> {
        if let Some(cached) = self.cached_package_by_root(source_kind, &canonical_root_str) {
            return Ok(cached);
        }
        let metadata_path = canonical_root.join("package.yaml");
        let build_path = canonical_root.join("build.fol");
        if !metadata_path.is_file() {
            return Err(PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package pkg import target '{}' is missing required package metadata '{}'",
                    canonical_root.display(),
                    metadata_path.display()
                ),
            ));
        }
        if !build_path.is_file() {
            return Err(PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package pkg import target '{}' is missing required package build file '{}'",
                    canonical_root.display(),
                    build_path.display()
                ),
            ));
        }

        let metadata = crate::parse_package_metadata(metadata_path.as_path())?;
        let build = crate::parse_package_build(build_path.as_path())?;
        let identity = PackageIdentity {
            source_kind,
            canonical_root: canonical_root_str,
            display_name: metadata.name.clone(),
        };

        self.begin_loading(&identity)?;

        if let Some(cached) = self.cached_package(&identity).cloned() {
            self.finish_loading();
            return Ok(cached);
        }

        if let Some(store_root) = store_root {
            for dependency in &build.dependencies {
                let path_segments = locator_use_path_segments(&dependency.locator);
                self.load_package_from_store(store_root, &path_segments)?;
            }
        }

        let prepared_result = parse_directory_package_syntax(
            canonical_root,
            &metadata.name,
            source_kind,
        )
        .and_then(|syntax| {
            let exports = compute_prepared_exports(&build, &syntax)?;
            Ok(PreparedPackage::with_controls(
                identity.clone(),
                metadata,
                build,
                exports,
                syntax,
            ))
        });

        self.finish_loading();

        let prepared = prepared_result?;
        self.cache_package(prepared.clone());
        Ok(prepared)
    }
}

pub fn infer_package_root(syntax: &ParsedPackage) -> Result<PathBuf, PackageError> {
    let mut parents = syntax
        .source_units
        .iter()
        .filter_map(|unit| Path::new(&unit.path).parent().map(Path::to_path_buf));

    let Some(mut common_root) = parents.next() else {
        return Err(PackageError::new(
            PackageErrorKind::InvalidInput,
            "package loading requires at least one parsed source unit",
        ));
    };

    for parent in parents {
        while !parent.starts_with(&common_root) {
            if !common_root.pop() {
                return Err(PackageError::new(
                    PackageErrorKind::Internal,
                    "package loading could not infer a common package root from parsed source paths",
                ));
            }
        }
    }

    Ok(common_root)
}

pub fn parse_directory_package_syntax(
    root: &Path,
    display_name: &str,
    source_kind: PackageSourceKind,
) -> Result<ParsedPackage, PackageError> {
    let root_str = root.to_str().ok_or_else(|| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!("package root '{}' is not valid UTF-8", root.display()),
        )
    })?;
    let mut sources = Source::init_with_package(root_str, SourceType::Folder, display_name)
        .map_err(|error| {
            PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package loader could not initialize package sources from '{}': {}",
                    root.display(),
                    error
                ),
            )
        })?;
    if source_kind == PackageSourceKind::Package {
        sources.retain(|source| !is_package_control_file(root, source));
        if sources.is_empty() {
            return Err(PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package import target '{}' has no loadable source files after excluding package control files",
                    root.display()
                ),
            ));
        }
    }
    let mut stream = FileStream::from_sources(sources).map_err(|error| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package loader could not read package sources from '{}': {}",
                root.display(),
                error
            ),
        )
    })?;
    let mut lexer = Elements::init(&mut stream);
    let mut parser = AstParser::new();

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
            PackageError::with_origin(
                PackageErrorKind::InvalidInput,
                format!(
                    "package loader could not parse imported package '{}': {}",
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
            PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package loader could not parse imported package '{}': {}",
                    root.display(),
                    first
                ),
            )
        }
    })
}

pub fn canonical_directory_root(
    directory: &Path,
    source_kind: PackageSourceKind,
) -> Result<PathBuf, PackageError> {
    let metadata = std::fs::metadata(directory).map_err(|error| {
        let label = import_source_label(source_kind);
        if error.kind() == std::io::ErrorKind::NotFound {
            PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package {label} import target '{}' does not exist",
                    directory.display(),
                ),
            )
        } else {
            PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package loader could not inspect {label} import target '{}': {}",
                    directory.display(),
                    error
                ),
            )
        }
    })?;

    if metadata.is_file() {
        let label = import_source_label(source_kind);
        return Err(PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package {label} import target '{}' must point to a directory, not a file",
                directory.display(),
            ),
        ));
    }

    if !metadata.is_dir() {
        let label = import_source_label(source_kind);
        return Err(PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package {label} import target '{}' must point to a directory",
                directory.display(),
            ),
        ));
    }

    std::fs::canonicalize(directory).map_err(|error| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package loader could not canonicalize {} import target '{}': {}",
                import_source_label(source_kind),
                directory.display(),
                error
            ),
        )
    })
}

pub fn resolve_directory_path(source_dir: &Path, path_segments: &[UsePathSegment]) -> PathBuf {
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

fn import_source_label(source_kind: PackageSourceKind) -> &'static str {
    match source_kind {
        PackageSourceKind::Local => "loc",
        PackageSourceKind::Standard => "std",
        PackageSourceKind::Package => "pkg",
        PackageSourceKind::Entry => "entry",
    }
}

fn reject_formal_package_roots_for_directory_imports(
    root: &Path,
    source_kind: PackageSourceKind,
) -> Result<(), PackageError> {
    if source_kind != PackageSourceKind::Local {
        return Ok(());
    }

    let build_path = root.join("build.fol");
    if build_path.is_file() {
        return Err(PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package loc import target '{}' contains '{}'; formal package roots must be imported with pkg instead of loc",
                root.display(),
                build_path.display()
            ),
        ));
    }

    Ok(())
}

fn locator_use_path_segments(locator: &PackageLocator) -> Vec<UsePathSegment> {
    locator
        .path_segments
        .iter()
        .enumerate()
        .map(|(index, part)| UsePathSegment {
            separator: (index > 0).then_some(fol_parser::ast::UsePathSeparator::Slash),
            spelling: part.clone(),
        })
        .collect()
}

fn compute_prepared_exports(
    build: &PackageBuildDefinition,
    syntax: &ParsedPackage,
) -> Result<Vec<PreparedExportMount>, PackageError> {
    let mut exports = BTreeSet::new();
    for export in &build.exports {
        let source_prefix =
            build_export_namespace_prefix(syntax.package.as_str(), export.relative_path.as_str())?;
        let matching_namespaces = syntax
            .source_units
            .iter()
            .map(|unit| unit.namespace.as_str())
            .filter(|namespace| {
                *namespace == source_prefix
                    || namespace.starts_with(&format!("{source_prefix}::"))
            })
            .collect::<BTreeSet<_>>();

        for namespace in matching_namespaces {
            let mounted_namespace_suffix = namespace
                .strip_prefix(source_prefix.as_str())
                .and_then(|suffix| suffix.strip_prefix("::"))
                .map(|suffix| suffix.to_string());
            let mounted_namespace_suffix = if export.alias == "root" {
                mounted_namespace_suffix
            } else {
                Some(match mounted_namespace_suffix {
                    Some(suffix) => format!("{}::{suffix}", export.alias),
                    None => export.alias.clone(),
                })
            };
            exports.insert(PreparedExportMount {
                source_namespace: namespace.to_string(),
                mounted_namespace_suffix,
            });
        }
    }

    Ok(exports.into_iter().collect())
}

fn build_export_namespace_prefix(
    package_name: &str,
    relative_path: &str,
) -> Result<String, PackageError> {
    let trimmed = relative_path.trim();
    if trimmed.is_empty() || trimmed == "." {
        return Ok(package_name.to_string());
    }

    let segments = trimmed.split('/').map(str::trim).collect::<Vec<_>>();
    if segments.iter().any(|segment| segment.is_empty()) {
        return Err(PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package build export path '{}' must contain non-empty slash-separated segments",
                relative_path
            ),
        ));
    }

    Ok(format!("{package_name}::{}", segments.join("::")))
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
        canonical_directory_root, infer_package_root, parse_directory_package_syntax,
        resolve_directory_path, PackageSession,
    };
    use crate::{
        PackageConfig, PackageIdentity, PackageSourceKind, PreparedExportMount, PreparedPackage,
    };
    use fol_parser::ast::{AstParser, ParsedPackage, UsePathSegment};
    use fol_stream::FileStream;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn parse_fixture_package() -> ParsedPackage {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open package fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        parser
            .parse_package(&mut lexer)
            .expect("Package fixture should parse as a package")
    }

    fn unique_temp_root(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fol_package_session_{}_{}_{}",
            label,
            std::process::id(),
            stamp
        ))
    }

    #[test]
    fn package_session_config_can_be_provided_explicitly() {
        let session = PackageSession::with_config(PackageConfig {
            std_root: Some("/tmp/fol_std".to_string()),
            package_store_root: Some("/tmp/fol_pkg".to_string()),
            package_cache_root: Some("/tmp/fol_cache".to_string()),
            package_git_cache_root: Some("/tmp/fol_git_cache".to_string()),
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
        assert_eq!(
            session.config().package_git_cache_root.as_deref(),
            Some("/tmp/fol_git_cache")
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
    fn package_session_can_prepare_entry_packages_from_parsed_syntax() {
        let session = PackageSession::new();
        let prepared = session
            .prepare_entry_package(parse_fixture_package())
            .expect("Package session should prepare parsed entry packages");

        assert_eq!(prepared.identity.source_kind, PackageSourceKind::Entry);
        assert!(
            prepared.identity.canonical_root.ends_with("parser"),
            "Prepared entry packages should infer a canonical package root from parsed source units",
        );
        assert_eq!(prepared.package_name(), "parser");
        assert!(prepared.metadata.is_none());
        assert!(prepared.build.is_none());
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

    #[test]
    fn inferred_package_root_uses_common_parent_of_parsed_source_units() {
        let parsed = {
            let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../test/parser/source_units");
            let mut stream =
                FileStream::from_folder(fixture_path).expect("Should open folder package fixture");
            let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
            let mut parser = AstParser::new();
            parser
                .parse_package(&mut lexer)
                .expect("Folder fixture should parse as a package")
        };

        let inferred = infer_package_root(&parsed).expect("Should infer a common package root");

        assert!(
            inferred.ends_with("source_units"),
            "Expected inferred package root to end with the parsed folder name, got {:?}",
            inferred
        );
    }

    #[test]
    fn canonical_directory_root_rejects_file_targets() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../test/parser/simple_var.fol");

        let error = canonical_directory_root(Path::new(fixture_path), PackageSourceKind::Local)
            .expect_err("Package directory loading should reject direct file targets");

        assert_eq!(error.kind(), crate::PackageErrorKind::InvalidInput);
        assert!(error
            .to_string()
            .contains("must point to a directory, not a file"));
    }

    #[test]
    fn resolve_directory_path_joins_relative_segments() {
        let resolved = resolve_directory_path(
            Path::new("/tmp/root"),
            &[
                UsePathSegment {
                    separator: None,
                    spelling: "deps".to_string(),
                },
                UsePathSegment {
                    separator: Some(fol_parser::ast::UsePathSeparator::Slash),
                    spelling: "json".to_string(),
                },
            ],
        );

        assert_eq!(resolved, Path::new("/tmp/root/deps/json"));
    }

    #[test]
    fn parse_directory_package_syntax_loads_folder_packages() {
        let temp_root = unique_temp_root("parse_directory_syntax");
        fs::create_dir_all(temp_root.join("dep"))
            .expect("Should create a temporary package root fixture");
        fs::write(temp_root.join("dep/main.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the dependency package fixture");

        let parsed = parse_directory_package_syntax(
            temp_root.join("dep").as_path(),
            "dep",
            PackageSourceKind::Local,
        )
        .expect("Package session helpers should parse directory packages");

        assert_eq!(parsed.package, "dep");
        assert_eq!(parsed.source_units.len(), 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-session fixture directory should be removable after the test");
    }

    #[test]
    fn package_session_can_load_local_directory_packages() {
        let temp_root = unique_temp_root("load_local_directory");
        fs::create_dir_all(temp_root.join("dep"))
            .expect("Should create a temporary package root fixture");
        fs::write(temp_root.join("dep/main.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the dependency package fixture");
        let mut session = PackageSession::new();

        let loaded = session
            .load_directory_package(temp_root.join("dep").as_path(), PackageSourceKind::Local)
            .expect("Package session should load local directory packages");

        assert_eq!(loaded.package_name(), "dep");
        assert_eq!(loaded.source_kind(), PackageSourceKind::Local);
        assert_eq!(session.cached_package_count(), 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-session fixture directory should be removable after the test");
    }

    #[test]
    fn package_session_rejects_local_directory_targets_that_define_build_fol() {
        let temp_root = unique_temp_root("load_local_directory_with_build");
        fs::create_dir_all(temp_root.join("dep"))
            .expect("Should create a temporary package root fixture");
        fs::write(temp_root.join("dep/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the dependency package fixture");
        fs::write(temp_root.join("dep/build.fol"), "def root: loc = \"src\";\n")
            .expect("Should write the formal package build marker");
        let mut session = PackageSession::new();

        let error = session
            .load_directory_package(temp_root.join("dep").as_path(), PackageSourceKind::Local)
            .expect_err("Local directory imports should reject formal package roots");

        assert_eq!(error.kind(), crate::PackageErrorKind::InvalidInput);
        assert!(
            error
                .to_string()
                .contains("must be imported with pkg instead of loc"),
            "Local directory import errors should explain that formal package roots belong to pkg",
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-session fixture directory should be removable after the test");
    }

    #[test]
    fn package_session_reuses_cached_local_directory_packages() {
        let temp_root = unique_temp_root("load_local_directory_cache");
        fs::create_dir_all(temp_root.join("dep"))
            .expect("Should create a temporary package root fixture");
        fs::write(temp_root.join("dep/main.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the dependency package fixture");
        let mut session = PackageSession::new();

        let first = session
            .load_directory_package(temp_root.join("dep").as_path(), PackageSourceKind::Local)
            .expect("Package session should load the local directory the first time");
        let second = session
            .load_directory_package(temp_root.join("dep").as_path(), PackageSourceKind::Local)
            .expect("Package session should reuse cached local directories");

        assert_eq!(first.identity, second.identity);
        assert_eq!(session.cached_package_count(), 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-session fixture directory should be removable after the test");
    }

    #[test]
    fn package_session_can_load_standard_directory_packages() {
        let temp_root = unique_temp_root("load_standard_directory");
        fs::create_dir_all(temp_root.join("fmt"))
            .expect("Should create a temporary std package root fixture");
        fs::write(temp_root.join("fmt/main.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the standard package fixture");
        let mut session = PackageSession::with_config(PackageConfig {
            std_root: Some(
                temp_root
                    .to_str()
                    .expect("Temporary std fixture root should be valid UTF-8")
                    .to_string(),
            ),
            package_store_root: None,
            package_cache_root: None,
            package_git_cache_root: None,
        });

        let loaded = session
            .load_directory_package(temp_root.join("fmt").as_path(), PackageSourceKind::Standard)
            .expect("Package session should load standard directory packages");

        assert_eq!(loaded.package_name(), "fmt");
        assert_eq!(loaded.source_kind(), PackageSourceKind::Standard);
        assert_eq!(session.cached_package_count(), 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-session fixture directory should be removable after the test");
    }

    #[test]
    fn package_session_can_load_installed_pkg_roots_with_required_controls() {
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
        let mut session = PackageSession::new();

        let loaded = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect("Package session should load installed package roots from the package store");

        assert_eq!(loaded.identity.source_kind, PackageSourceKind::Package);
        assert_eq!(loaded.identity.display_name, "json");
        assert_eq!(loaded.package_name(), "json");
        assert_eq!(loaded.syntax.source_units.len(), 1);
        assert!(
            loaded
                .syntax
                .source_units
                .iter()
                .all(|unit| !unit.path.ends_with("package.yaml") && !unit.path.ends_with("build.fol")),
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
        assert_eq!(
            loaded.exports,
            vec![PreparedExportMount {
                source_namespace: "json::lib".to_string(),
                mounted_namespace_suffix: None,
            }]
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn parse_directory_package_syntax_excludes_control_files_for_pkg_roots() {
        let temp_root = unique_temp_root("pkg_control_file_exclusion");
        fs::create_dir_all(temp_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(temp_root.join("json/package.yaml"), "name: json\nversion: 1.0.0\n")
            .expect("Should write the package metadata fixture");
        fs::write(temp_root.join("json/build.fol"), "def root: loc = \"src\";\n")
            .expect("Should write the package build fixture");
        fs::write(temp_root.join("json/package.fol"), "var ignored: int = 1;\n")
            .expect("Should write the ignored legacy control fixture");
        fs::create_dir_all(temp_root.join("json/src"))
            .expect("Should create the exported source fixture");
        fs::write(temp_root.join("json/src/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the package source fixture");

        let parsed = parse_directory_package_syntax(
            temp_root.join("json").as_path(),
            "json",
            PackageSourceKind::Package,
        )
        .expect("Pkg source parsing should exclude control files and keep ordinary source files");

        assert_eq!(parsed.source_units.len(), 1);
        assert!(
            parsed
                .source_units
                .iter()
                .all(|unit| {
                    !unit.path.ends_with("package.yaml")
                        && !unit.path.ends_with("package.fol")
                        && !unit.path.ends_with("build.fol")
                }),
            "Pkg source parsing should keep package control files out of the parsed source set",
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn parse_directory_package_syntax_rejects_pkg_roots_with_only_control_files() {
        let temp_root = unique_temp_root("pkg_controls_only");
        fs::create_dir_all(temp_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(temp_root.join("json/package.yaml"), "name: json\nversion: 1.0.0\n")
            .expect("Should write the package metadata fixture");
        fs::write(temp_root.join("json/build.fol"), "def root: loc = \"src\";\n")
            .expect("Should write the package build fixture");
        fs::write(temp_root.join("json/package.fol"), "var ignored: int = 1;\n")
            .expect("Should write the ignored legacy control fixture");

        let error = parse_directory_package_syntax(
            temp_root.join("json").as_path(),
            "json",
            PackageSourceKind::Package,
        )
        .expect_err("Pkg roots with only control files should fail after control-file exclusion");

        assert_eq!(error.kind(), crate::PackageErrorKind::InvalidInput);
        assert!(
            error
                .to_string()
                .contains("no loadable source files after excluding package control files"),
            "Pkg control-only roots should fail with an explicit exclusion diagnostic",
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn package_session_computes_nested_export_namespace_mounts() {
        let temp_root = unique_temp_root("pkg_nested_exports");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json/src/root"))
            .expect("Should create the exported root fixture");
        fs::create_dir_all(store_root.join("json/src/fmt/nested"))
            .expect("Should create the nested export fixture");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the package metadata fixture");
        fs::write(
            store_root.join("json/build.fol"),
            concat!(
                "def root: loc = \"src/root\";\n",
                "def fmt: loc = \"src/fmt\";\n",
            ),
        )
        .expect("Should write the package build fixture");
        fs::write(
            store_root.join("json/src/root/value.fol"),
            "var[exp] answer: int = 42;\n",
        )
        .expect("Should write the exported root source fixture");
        fs::write(
            store_root.join("json/src/fmt/value.fol"),
            "var[exp] formatted: int = 7;\n",
        )
        .expect("Should write the exported namespace source fixture");
        fs::write(
            store_root.join("json/src/fmt/nested/value.fol"),
            "var[exp] nested_value: int = 9;\n",
        )
        .expect("Should write the nested exported namespace source fixture");
        let mut session = PackageSession::new();

        let loaded = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect("Package session should compute concrete export namespace mounts");

        assert_eq!(
            loaded.exports,
            vec![
                PreparedExportMount {
                    source_namespace: "json::src::fmt".to_string(),
                    mounted_namespace_suffix: Some("fmt".to_string()),
                },
                PreparedExportMount {
                    source_namespace: "json::src::fmt::nested".to_string(),
                    mounted_namespace_suffix: Some("fmt::nested".to_string()),
                },
                PreparedExportMount {
                    source_namespace: "json::src::root".to_string(),
                    mounted_namespace_suffix: None,
                },
            ]
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn package_session_rejects_pkg_roots_without_required_metadata() {
        let temp_root = unique_temp_root("missing_pkg_metadata");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json"))
            .expect("Should create a temporary package-store fixture");
        fs::write(store_root.join("json/lib.fol"), "var[exp] answer: int = 42;\n")
            .expect("Should write the package source fixture");
        let mut session = PackageSession::new();

        let error = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect_err("Package session should reject installed package roots without metadata");

        assert_eq!(error.kind(), crate::PackageErrorKind::InvalidInput);
        assert!(error.to_string().contains("missing required package metadata"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn package_session_ignores_package_fol_when_package_yaml_is_present() {
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
        let mut session = PackageSession::new();

        let loaded = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect("Package session should ignore package.fol when package.yaml is present");

        assert_eq!(loaded.identity.display_name, "json");
        assert_eq!(loaded.package_name(), "json");

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn package_session_preloads_transitive_pkg_dependencies() {
        let temp_root = unique_temp_root("transitive_pkg_graph");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("core/src/root"))
            .expect("Should create the transitive dependency export root fixture");
        fs::create_dir_all(store_root.join("json/src/root"))
            .expect("Should create the direct dependency export root fixture");
        fs::write(
            store_root.join("core/package.yaml"),
            "name: core\nversion: 1.0.0\n",
        )
        .expect("Should write the transitive dependency metadata fixture");
        fs::write(store_root.join("core/build.fol"), "def root: loc = \"src/root\";\n")
            .expect("Should write the transitive dependency build fixture");
        fs::write(
            store_root.join("core/src/root/value.fol"),
            "var[exp] shared: int = 7;\n",
        )
        .expect("Should write the transitive dependency source fixture");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the direct dependency metadata fixture");
        fs::write(
            store_root.join("json/build.fol"),
            "def core: pkg = \"core\";\ndef root: loc = \"src/root\";\n",
        )
        .expect("Should write the direct dependency build fixture");
        fs::write(
            store_root.join("json/src/root/value.fol"),
            "use core: pkg = {core};\nvar[exp] answer: int = shared;\n",
        )
        .expect("Should write the direct dependency source fixture");
        let mut session = PackageSession::new();

        let loaded = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect("Package session should load direct package roots");

        assert_eq!(loaded.identity.display_name, "json");
        assert_eq!(session.cached_package_count(), 2);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn package_session_reports_explicit_pkg_dependency_cycles() {
        let temp_root = unique_temp_root("cyclic_pkg_graph");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("json/src/root"))
            .expect("Should create the first cyclic package fixture");
        fs::create_dir_all(store_root.join("core/src/root"))
            .expect("Should create the second cyclic package fixture");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the first package metadata fixture");
        fs::write(
            store_root.join("json/build.fol"),
            "def core: pkg = \"core\";\ndef root: loc = \"src/root\";\n",
        )
        .expect("Should write the first package build fixture");
        fs::write(
            store_root.join("json/src/root/value.fol"),
            "var[exp] answer: int = 1;\n",
        )
        .expect("Should write the first package source fixture");
        fs::write(
            store_root.join("core/package.yaml"),
            "name: core\nversion: 1.0.0\n",
        )
        .expect("Should write the second package metadata fixture");
        fs::write(
            store_root.join("core/build.fol"),
            "def json: pkg = \"json\";\ndef root: loc = \"src/root\";\n",
        )
        .expect("Should write the second package build fixture");
        fs::write(
            store_root.join("core/src/root/value.fol"),
            "var[exp] shared: int = 2;\n",
        )
        .expect("Should write the second package source fixture");
        let mut session = PackageSession::new();

        let error = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "json".to_string(),
                }],
            )
            .expect_err("Package session should reject cyclic package dependency graphs");

        assert_eq!(error.kind(), crate::PackageErrorKind::ImportCycle);
        assert!(error
            .to_string()
            .contains("package import cycle detected while loading package roots"));
        assert!(
            error
                .to_string()
                .contains("json")
                && error.to_string().contains("core"),
            "Cycle diagnostics should list the participating package roots",
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }

    #[test]
    fn package_session_dedupes_shared_transitive_pkg_dependencies() {
        let temp_root = unique_temp_root("shared_pkg_graph");
        let store_root = temp_root.join("store");
        fs::create_dir_all(store_root.join("core/src/root"))
            .expect("Should create the shared dependency export root fixture");
        fs::create_dir_all(store_root.join("json/src/root"))
            .expect("Should create the first direct dependency export root fixture");
        fs::create_dir_all(store_root.join("xml/src/root"))
            .expect("Should create the second direct dependency export root fixture");
        fs::create_dir_all(store_root.join("combo/src/root"))
            .expect("Should create the top-level package export root fixture");
        fs::write(
            store_root.join("core/package.yaml"),
            "name: core\nversion: 1.0.0\n",
        )
        .expect("Should write the shared dependency metadata fixture");
        fs::write(store_root.join("core/build.fol"), "def root: loc = \"src/root\";\n")
            .expect("Should write the shared dependency build fixture");
        fs::write(
            store_root.join("core/src/root/value.fol"),
            "var[exp] shared: int = 7;\n",
        )
        .expect("Should write the shared dependency source fixture");
        fs::write(
            store_root.join("json/package.yaml"),
            "name: json\nversion: 1.0.0\n",
        )
        .expect("Should write the first direct dependency metadata fixture");
        fs::write(
            store_root.join("json/build.fol"),
            "def core: pkg = \"core\";\ndef root: loc = \"src/root\";\n",
        )
        .expect("Should write the first direct dependency build fixture");
        fs::write(
            store_root.join("json/src/root/value.fol"),
            "var[exp] answer: int = 1;\n",
        )
        .expect("Should write the first direct dependency source fixture");
        fs::write(
            store_root.join("xml/package.yaml"),
            "name: xml\nversion: 1.0.0\n",
        )
        .expect("Should write the second direct dependency metadata fixture");
        fs::write(
            store_root.join("xml/build.fol"),
            "def core: pkg = \"core\";\ndef root: loc = \"src/root\";\n",
        )
        .expect("Should write the second direct dependency build fixture");
        fs::write(
            store_root.join("xml/src/root/value.fol"),
            "var[exp] answer: int = 2;\n",
        )
        .expect("Should write the second direct dependency source fixture");
        fs::write(
            store_root.join("combo/package.yaml"),
            "name: combo\nversion: 1.0.0\n",
        )
        .expect("Should write the top-level package metadata fixture");
        fs::write(
            store_root.join("combo/build.fol"),
            concat!(
                "def json: pkg = \"json\";\n",
                "def xml: pkg = \"xml\";\n",
                "def root: loc = \"src/root\";\n",
            ),
        )
        .expect("Should write the top-level package build fixture");
        fs::write(
            store_root.join("combo/src/root/value.fol"),
            "var[exp] answer: int = 3;\n",
        )
        .expect("Should write the top-level package source fixture");
        let mut session = PackageSession::new();

        let loaded = session
            .load_package_from_store(
                &store_root,
                &[UsePathSegment {
                    separator: None,
                    spelling: "combo".to_string(),
                }],
            )
            .expect("Package session should load shared dependency graphs");

        assert_eq!(loaded.identity.display_name, "combo");
        assert_eq!(session.cached_package_count(), 4);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary package-store fixture should be removable after the test");
    }
}
