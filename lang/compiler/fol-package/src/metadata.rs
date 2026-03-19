use crate::{PackageError, PackageErrorKind};
use fol_parser::ast::SyntaxOrigin;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageDependencySourceKind {
    Local,
    PackageStore,
    Git,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDependencyDecl {
    pub alias: String,
    pub source_kind: PackageDependencySourceKind,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub dependencies: Vec<PackageDependencyDecl>,
}

pub fn parse_package_metadata(path: &Path) -> Result<PackageMetadata, PackageError> {
    let raw = std::fs::read_to_string(path).map_err(|error| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package loader could not read package metadata '{}': {}",
                path.display(),
                error
            ),
        )
    })?;

    let mut fields: BTreeMap<String, (String, SyntaxOrigin)> = BTreeMap::new();
    let supported_fields = BTreeSet::from(["name", "version", "kind", "description", "license"]);
    let mut dependencies = Vec::new();
    let mut dependency_aliases = BTreeMap::<String, SyntaxOrigin>::new();
    for (index, line) in raw.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            return Err(metadata_line_error(
                path,
                line_no,
                1,
                line.len().max(1),
                "package metadata lines must follow 'key: value' form",
            ));
        };
        let key = raw_key.trim();
        if key.is_empty() {
            return Err(metadata_line_error(
                path,
                line_no,
                1,
                line.len().max(1),
                "package metadata keys may not be empty",
            ));
        }
        if let Some(alias) = key.strip_prefix("dep.") {
            let value_offset = line.find(':').unwrap_or(0) + 2;
            let value = parse_yaml_scalar(raw_value.trim(), path, line_no, value_offset)?;
            let origin = SyntaxOrigin {
                file: Some(path.to_string_lossy().to_string()),
                line: line_no,
                column: 1,
                length: key.len(),
            };
            if let Some(first_origin) = dependency_aliases.insert(alias.to_string(), origin.clone())
            {
                return Err(PackageError::with_origin(
                    PackageErrorKind::InvalidInput,
                    format!(
                        "package dependency alias '{}' in '{}' may only be declared once",
                        alias,
                        path.display()
                    ),
                    origin,
                )
                .with_related_origin(first_origin, "first package dependency alias declaration"));
            }
            dependencies.push(parse_dependency_decl(path, alias, &value, origin)?);
            continue;
        }
        if !supported_fields.contains(key) {
            return Err(metadata_line_error(
                path,
                line_no,
                1,
                key.len(),
                format!(
                    "unsupported package metadata field '{key}'; expected only name, version, kind, description, or license"
                ),
            ));
        }
        let value_offset = line.find(':').unwrap_or(0) + 2;
        let value = parse_yaml_scalar(raw_value.trim(), path, line_no, value_offset)?;
        let origin = SyntaxOrigin {
            file: Some(path.to_string_lossy().to_string()),
            line: line_no,
            column: 1,
            length: key.len(),
        };

        if let Some((_, first_origin)) = fields.insert(key.to_string(), (value, origin.clone())) {
            return Err(metadata_line_error(
                path,
                line_no,
                1,
                key.len(),
                format!("package metadata field '{key}' may only be declared once"),
            )
            .with_related_origin(first_origin, "first package metadata field declaration"));
        }
    }

    let name = fields
        .remove("name")
        .map(|(value, _)| value)
        .ok_or_else(|| {
            PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package metadata '{}' is missing required field 'name'",
                    path.display()
                ),
            )
        })?;
    if !is_valid_package_name(&name) {
        return Err(PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package metadata '{}' has invalid package name '{}'; package names must follow namespace identifier rules",
                path.display(),
                name
            ),
        ));
    }
    let version = fields
        .remove("version")
        .map(|(value, _)| value)
        .ok_or_else(|| {
            PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package metadata '{}' is missing required field 'version'",
                    path.display()
                ),
            )
        })?;
    if version.trim().is_empty() {
        return Err(PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package metadata '{}' has an empty version string",
                path.display()
            ),
        ));
    }

    Ok(PackageMetadata {
        name,
        version,
        kind: non_empty_optional_field(
            path,
            "kind",
            fields.remove("kind").map(|(value, _)| value),
        )?,
        description: non_empty_optional_field(
            path,
            "description",
            fields.remove("description").map(|(value, _)| value),
        )?,
        license: non_empty_optional_field(
            path,
            "license",
            fields.remove("license").map(|(value, _)| value),
        )?,
        dependencies,
    })
}

fn parse_dependency_decl(
    path: &Path,
    alias: &str,
    value: &str,
    origin: SyntaxOrigin,
) -> Result<PackageDependencyDecl, PackageError> {
    if !is_valid_package_name(alias) {
        return Err(PackageError::with_origin(
            PackageErrorKind::InvalidInput,
            format!(
                "package metadata '{}' has invalid dependency alias '{}'; dependency aliases must follow namespace identifier rules",
                path.display(),
                alias
            ),
            origin,
        ));
    }
    let Some((source_raw, target_raw)) = value.split_once(':') else {
        return Err(PackageError::with_origin(
            PackageErrorKind::InvalidInput,
            format!(
                "package dependency '{}' in '{}' must use 'source:target' form",
                alias,
                path.display()
            ),
            origin,
        ));
    };
    let source_kind = match source_raw.trim() {
        "loc" => PackageDependencySourceKind::Local,
        "pkg" => PackageDependencySourceKind::PackageStore,
        "git" => PackageDependencySourceKind::Git,
        other => {
            return Err(PackageError::with_origin(
                PackageErrorKind::InvalidInput,
                format!(
                    "package dependency '{}' in '{}' uses unsupported source '{}'; expected loc, pkg, or git",
                    alias,
                    path.display(),
                    other
                ),
                origin,
            ))
        }
    };
    let target = target_raw.trim();
    if target.is_empty() {
        return Err(PackageError::with_origin(
            PackageErrorKind::InvalidInput,
            format!(
                "package dependency '{}' in '{}' has an empty target",
                alias,
                path.display()
            ),
            origin,
        ));
    }
    Ok(PackageDependencyDecl {
        alias: alias.to_string(),
        source_kind,
        target: target.to_string(),
    })
}

fn parse_yaml_scalar(
    raw: &str,
    path: &Path,
    line: usize,
    column: usize,
) -> Result<String, PackageError> {
    if raw.is_empty() {
        return Err(metadata_line_error(
            path,
            line,
            column,
            1,
            "package metadata values may not be empty",
        ));
    }

    let raw = strip_inline_comment(raw).trim();
    if raw.is_empty() {
        return Err(metadata_line_error(
            path,
            line,
            column,
            1,
            "package metadata values may not be empty",
        ));
    }

    if let Some(unquoted) = strip_matching_quotes(raw) {
        return Ok(unquoted.to_string());
    }

    Ok(raw.to_string())
}

fn strip_inline_comment(raw: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = raw.char_indices().peekable();

    while let Some((index, ch)) = chars.next() {
        match ch {
            '\\' if in_double || in_single => {
                // skip the next character (escaped quote or backslash)
                chars.next();
            }
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' if !in_single && !in_double => return &raw[..index],
            _ => {}
        }
    }

    raw
}

fn strip_matching_quotes(raw: &str) -> Option<&str> {
    let bytes = raw.as_bytes();
    if bytes.len() >= 2 && bytes.first() == bytes.last() {
        match bytes[0] {
            b'"' | b'\'' => Some(&raw[1..raw.len() - 1]),
            _ => None,
        }
    } else {
        None
    }
}

fn metadata_line_error(
    path: &Path,
    line: usize,
    column: usize,
    length: usize,
    message: impl Into<String>,
) -> PackageError {
    PackageError::with_origin(
        PackageErrorKind::InvalidInput,
        message,
        SyntaxOrigin {
            file: Some(path.to_string_lossy().to_string()),
            line,
            column,
            length,
        },
    )
}

fn non_empty_optional_field(
    path: &Path,
    field_name: &str,
    value: Option<String>,
) -> Result<Option<String>, PackageError> {
    match value {
        Some(value) if value.trim().is_empty() => Err(PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package metadata '{}' has an empty '{}' field",
                path.display(),
                field_name
            ),
        )),
        other => Ok(other),
    }
}

fn is_valid_package_name(package_name: &str) -> bool {
    if package_name.len() > 256 {
        return false;
    }
    let mut chars = package_name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii() || first.is_ascii_digit() || !(first.is_ascii_alphabetic() || first == '_')
    {
        return false;
    }

    if package_name.contains("__") {
        return false;
    }

    chars.all(|ch| ch.is_ascii() && (ch.is_ascii_alphanumeric() || ch == '_'))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_package_metadata, PackageDependencyDecl, PackageDependencySourceKind, PackageMetadata,
    };
    use fol_diagnostics::ToDiagnostic;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fol_package_metadata_{}_{}_{}",
            label,
            std::process::id(),
            stamp
        ))
    }

    #[test]
    fn yaml_metadata_parser_extracts_required_and_optional_fields() {
        let temp_root = unique_temp_root("parse_metadata");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(
            &metadata_path,
            concat!(
                "name: json\n",
                "version: '1.0.0'\n",
                "kind: lib\n",
                "description: \"JSON tooling\"\n",
                "license: MIT\n"
            ),
        )
        .expect("Should write the metadata fixture");

        let metadata =
            parse_package_metadata(&metadata_path).expect("YAML metadata fixture should parse");

        assert_eq!(
            metadata,
            PackageMetadata {
                name: "json".to_string(),
                version: "1.0.0".to_string(),
                kind: Some("lib".to_string()),
                description: Some("JSON tooling".to_string()),
                license: Some("MIT".to_string()),
                dependencies: Vec::new(),
            }
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary metadata fixture root should be removable after the test");
    }

    #[test]
    fn yaml_metadata_parser_rejects_duplicate_fields_with_exact_origin() {
        let temp_root = unique_temp_root("duplicate_field");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(&metadata_path, "name: json\nversion: 1.0.0\nname: other\n")
            .expect("Should write the duplicate metadata fixture");

        let error = parse_package_metadata(&metadata_path)
            .expect_err("Duplicate metadata fields should be rejected");

        assert!(error
            .to_string()
            .contains("package metadata field 'name' may only be declared once"));
        let origin = error
            .origin()
            .expect("Duplicate metadata errors should keep exact line origins");
        assert_eq!(origin.line, 3);
        assert_eq!(origin.column, 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary metadata fixture root should be removable after the test");
    }

    #[test]
    fn yaml_metadata_parser_lowers_duplicate_fields_with_first_site_secondary_label() {
        let temp_root = unique_temp_root("duplicate_field_secondary_label");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(&metadata_path, "name: json\nversion: 1.0.0\nname: other\n")
            .expect("Should write the duplicate metadata fixture");

        let diagnostic = parse_package_metadata(&metadata_path)
            .expect_err("Duplicate metadata fields should be rejected")
            .to_diagnostic();

        assert_eq!(diagnostic.labels.len(), 2);
        assert_eq!(diagnostic.labels[0].location.line, 3);
        assert_eq!(diagnostic.labels[0].message, None);
        assert_eq!(
            diagnostic.labels[1].message.as_deref(),
            Some("first package metadata field declaration")
        );
        assert_eq!(diagnostic.labels[1].location.line, 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary metadata fixture root should be removable after the test");
    }

    #[test]
    fn yaml_metadata_parser_rejects_unsupported_fields() {
        let temp_root = unique_temp_root("unsupported_field");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(&metadata_path, "name: json\nversion: 1.0.0\ndeps: none\n")
            .expect("Should write the unsupported-field metadata fixture");

        let error = parse_package_metadata(&metadata_path)
            .expect_err("Unsupported metadata fields should be rejected");

        assert!(error
            .to_string()
            .contains("unsupported package metadata field 'deps'"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary metadata fixture root should be removable after the test");
    }

    #[test]
    fn yaml_metadata_parser_rejects_invalid_line_shape() {
        let temp_root = unique_temp_root("invalid_shape");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(&metadata_path, "name json\n")
            .expect("Should write the invalid-shape metadata fixture");

        let error = parse_package_metadata(&metadata_path)
            .expect_err("Malformed metadata lines should be rejected");

        assert!(error
            .to_string()
            .contains("package metadata lines must follow 'key: value' form"));
        let origin = error
            .origin()
            .expect("Malformed metadata errors should keep exact line origins");
        assert_eq!(origin.line, 1);
        assert_eq!(origin.column, 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary metadata fixture root should be removable after the test");
    }

    #[test]
    fn yaml_metadata_parser_extracts_dependency_entries() {
        let temp_root = unique_temp_root("deps");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(
            &metadata_path,
            concat!(
                "name: app\n",
                "version: 0.1.0\n",
                "dep.core: pkg:core/tools\n",
                "dep.logtiny: git:https://github.com/bresilla/logtiny.git\n",
            ),
        )
        .expect("Should write the dependency metadata fixture");

        let metadata =
            parse_package_metadata(&metadata_path).expect("dependency metadata should parse");

        assert_eq!(
            metadata.dependencies,
            vec![
                PackageDependencyDecl {
                    alias: "core".to_string(),
                    source_kind: PackageDependencySourceKind::PackageStore,
                    target: "core/tools".to_string(),
                },
                PackageDependencyDecl {
                    alias: "logtiny".to_string(),
                    source_kind: PackageDependencySourceKind::Git,
                    target: "https://github.com/bresilla/logtiny.git".to_string(),
                },
            ]
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary metadata fixture root should be removable after the test");
    }

    #[test]
    fn yaml_metadata_parser_supports_source_qualified_local_pkg_and_git_dependencies() {
        let temp_root = unique_temp_root("deps_sources");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(
            &metadata_path,
            concat!(
                "name: app\n",
                "version: 0.1.0\n",
                "dep.shared: loc:../shared\n",
                "dep.core: pkg:core/tools\n",
                "dep.logtiny: git:https://github.com/bresilla/logtiny.git\n",
            ),
        )
        .expect("Should write the source-qualified dependency metadata fixture");

        let metadata = parse_package_metadata(&metadata_path)
            .expect("source-qualified dependency metadata should parse");

        assert_eq!(metadata.dependencies.len(), 3);
        assert_eq!(
            metadata.dependencies[0].source_kind,
            PackageDependencySourceKind::Local
        );
        assert_eq!(metadata.dependencies[0].target, "../shared");
        assert_eq!(
            metadata.dependencies[1].source_kind,
            PackageDependencySourceKind::PackageStore
        );
        assert_eq!(metadata.dependencies[1].target, "core/tools");
        assert_eq!(
            metadata.dependencies[2].source_kind,
            PackageDependencySourceKind::Git
        );
        assert_eq!(
            metadata.dependencies[2].target,
            "https://github.com/bresilla/logtiny.git"
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary metadata fixture root should be removable after the test");
    }

    #[test]
    fn yaml_metadata_parser_rejects_duplicate_dependency_aliases() {
        let temp_root = unique_temp_root("duplicate_dep_alias");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(
            &metadata_path,
            concat!(
                "name: app\n",
                "version: 0.1.0\n",
                "dep.core: pkg:core\n",
                "dep.core: git:https://github.com/bresilla/logtiny.git\n",
            ),
        )
        .expect("Should write the duplicate dependency alias fixture");

        let error = parse_package_metadata(&metadata_path)
            .expect_err("duplicate dependency aliases should be rejected");

        assert!(
            error
                .to_string()
                .contains("package dependency alias 'core'"),
            "duplicate dependency aliases should mention the duplicated alias",
        );
        let diagnostic = error.to_diagnostic();
        assert_eq!(diagnostic.labels.len(), 2);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary metadata fixture root should be removable after the test");
    }

    #[test]
    fn yaml_metadata_parser_reports_invalid_dependency_forms_clearly() {
        let cases = [
            ("dep.core: core/tools\n", "must use 'source:target' form"),
            (
                "dep.core: svn:core/tools\n",
                "uses unsupported source 'svn'",
            ),
            ("dep.core: pkg:\n", "has an empty target"),
            (
                "dep.9core: pkg:core/tools\n",
                "has invalid dependency alias '9core'",
            ),
        ];

        for (index, (dependency_line, message)) in cases.into_iter().enumerate() {
            let temp_root = unique_temp_root(&format!("invalid_dep_{index}"));
            fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
            let metadata_path = temp_root.join("package.yaml");
            fs::write(
                &metadata_path,
                format!("name: app\nversion: 0.1.0\n{dependency_line}"),
            )
            .expect("Should write the invalid dependency metadata fixture");

            let error = parse_package_metadata(&metadata_path)
                .expect_err("invalid dependency forms should be rejected");

            assert!(
                error.to_string().contains(message),
                "invalid dependency form should explain '{message}', got: {error}",
            );

            fs::remove_dir_all(&temp_root)
                .expect("Temporary metadata fixture root should be removable after the test");
        }
    }

    #[test]
    fn inline_comment_stripping_handles_escaped_quotes_in_double_quoted_strings() {
        use super::strip_inline_comment;

        // backslash-escaped quote inside double-quoted string must not end the string
        assert_eq!(strip_inline_comment(r#""foo\"bar" # comment"#), r#""foo\"bar" "#);
        // backslash-escaped backslash followed by a closing quote
        assert_eq!(strip_inline_comment(r#""foo\\" # comment"#), r#""foo\\" "#);
        // unquoted value with a comment
        assert_eq!(strip_inline_comment("hello # comment"), "hello ");
    }

    #[test]
    fn package_name_validation_rejects_names_longer_than_256_characters() {
        use super::is_valid_package_name;

        let long_name = "a".repeat(257);
        assert!(!is_valid_package_name(&long_name));
        let max_name = "a".repeat(256);
        assert!(is_valid_package_name(&max_name));
    }

    #[test]
    fn yaml_metadata_dependency_matrix_stays_stable_for_local_pkg_and_git_forms() {
        let temp_root = unique_temp_root("dep_matrix");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(
            &metadata_path,
            concat!(
                "name: demo\n",
                "version: 0.1.0\n",
                "dep.shared: loc:../shared\n",
                "dep.core: pkg:org/core\n",
                "dep.logtiny: git:git@github.com:bresilla/logtiny.git\n",
            ),
        )
        .expect("Should write the dependency matrix fixture");

        let metadata =
            parse_package_metadata(&metadata_path).expect("dependency matrix fixture should parse");

        let matrix = metadata
            .dependencies
            .iter()
            .map(|dep| (dep.alias.as_str(), dep.source_kind, dep.target.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(
            matrix,
            vec![
                ("shared", PackageDependencySourceKind::Local, "../shared"),
                (
                    "core",
                    PackageDependencySourceKind::PackageStore,
                    "org/core"
                ),
                (
                    "logtiny",
                    PackageDependencySourceKind::Git,
                    "git@github.com:bresilla/logtiny.git",
                ),
            ]
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary metadata fixture root should be removable after the test");
    }
}
