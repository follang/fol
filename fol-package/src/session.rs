use crate::{PackageConfig, PackageError, PackageErrorKind, PackageIdentity, PackageSourceKind, PreparedPackage};
use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::{AstParser, ParsedPackage, UsePathSegment};
use fol_stream::{FileStream, Source, SourceType};
use std::path::{Path, PathBuf};
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
    use crate::{PackageConfig, PackageIdentity, PackageSourceKind, PreparedPackage};
    use fol_parser::ast::{AstParser, ParsedPackage, UsePathSegment};
    use fol_stream::FileStream;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn parse_fixture_package() -> ParsedPackage {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
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

    #[test]
    fn inferred_package_root_uses_common_parent_of_parsed_source_units() {
        let parsed = {
            let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/source_units");
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
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");

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
}
