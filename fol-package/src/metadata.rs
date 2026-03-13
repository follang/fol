use crate::{PackageError, PackageErrorKind};
use fol_parser::ast::SyntaxOrigin;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
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

    let mut fields = BTreeMap::new();
    let supported_fields = BTreeSet::from(["name", "version", "kind", "description", "license"]);
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

        if fields.insert(key.to_string(), value).is_some() {
            return Err(metadata_line_error(
                path,
                line_no,
                1,
                key.len(),
                format!("package metadata field '{key}' may only be declared once"),
            ));
        }
    }

    let name = fields.remove("name").ok_or_else(|| {
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
    let version = fields.remove("version").ok_or_else(|| {
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
        kind: non_empty_optional_field(path, "kind", fields.remove("kind"))?,
        description: non_empty_optional_field(path, "description", fields.remove("description"))?,
        license: non_empty_optional_field(path, "license", fields.remove("license"))?,
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

    for (index, ch) in raw.char_indices() {
        match ch {
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
    use super::{parse_package_metadata, PackageMetadata};
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
        fs::write(
            &metadata_path,
            "name: json\nversion: 1.0.0\nname: other\n",
        )
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
    fn yaml_metadata_parser_rejects_unsupported_fields() {
        let temp_root = unique_temp_root("unsupported_field");
        fs::create_dir_all(&temp_root).expect("Should create temporary metadata fixture root");
        let metadata_path = temp_root.join("package.yaml");
        fs::write(
            &metadata_path,
            "name: json\nversion: 1.0.0\ndeps: none\n",
        )
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
}
