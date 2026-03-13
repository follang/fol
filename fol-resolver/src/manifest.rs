use crate::{ResolverError, ResolverErrorKind};
use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::{AstNode, AstParser, FolType, Literal, SyntaxIndex, SyntaxNodeId, SyntaxOrigin, UsePathSegment};
use fol_stream::FileStream;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManifestDependency {
    pub alias: String,
    pub package_path: Vec<UsePathSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackageManifest {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<ManifestDependency>,
}

pub(crate) fn parse_package_manifest(path: &Path) -> Result<PackageManifest, ResolverError> {
    let path_str = path.to_str().ok_or_else(|| {
        ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!("package manifest '{}' is not valid UTF-8", path.display()),
        )
    })?;
    let mut stream = FileStream::from_file(path_str).map_err(|error| {
        ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "resolver could not read package manifest '{}': {}",
                path.display(),
                error
            ),
        )
    })?;
    let mut lexer = Elements::init(&mut stream);
    let mut parser = AstParser::new();
    let parsed = parser.parse_package(&mut lexer).map_err(|errors| {
        let first = errors
            .into_iter()
            .next()
            .expect("manifest parse should produce at least one error");
        if let Some(parse_error) = first
            .as_ref()
            .as_any()
            .downcast_ref::<fol_parser::ast::ParseError>()
        {
            ResolverError::with_origin(
                ResolverErrorKind::InvalidInput,
                format!(
                    "resolver could not parse package manifest '{}': {}",
                    path.display(),
                    parse_error
                ),
                SyntaxOrigin {
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
                    "resolver could not parse package manifest '{}': {}",
                    path.display(),
                    first
                ),
            )
        }
    })?;
    let source_unit = parsed.source_units.first().ok_or_else(|| {
        ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "resolver package manifest '{}' did not produce any source units",
                path.display()
            ),
        )
    })?;

    let mut name = None;
    let mut version = None;
    let mut dependencies = Vec::new();
    let mut dependency_aliases = BTreeSet::new();

    for item in &source_unit.items {
        match unwrap_comment_wrappers(&item.node) {
            AstNode::Comment { .. } => {}
            AstNode::VarDecl {
                name: field_name,
                value,
                ..
            } => match field_name.as_str() {
                "name" => {
                    let field_value = manifest_string_field(
                        "name",
                        value.as_deref(),
                        &parsed.syntax_index,
                        item.node_id,
                    )?;
                    if name.replace(field_value).is_some() {
                        return Err(manifest_item_error(
                            &parsed.syntax_index,
                            item.node_id,
                            "package manifest field 'name' may only be declared once",
                        ));
                    }
                }
                "version" => {
                    let field_value = manifest_string_field(
                        "version",
                        value.as_deref(),
                        &parsed.syntax_index,
                        item.node_id,
                    )?;
                    if version.replace(field_value).is_some() {
                        return Err(manifest_item_error(
                            &parsed.syntax_index,
                            item.node_id,
                            "package manifest field 'version' may only be declared once",
                        ));
                    }
                }
                other => {
                    return Err(manifest_item_error(
                        &parsed.syntax_index,
                        item.node_id,
                        format!(
                            "unsupported package manifest field '{other}'; expected only 'name', 'version', and pkg dependencies"
                        ),
                    ));
                }
            },
            AstNode::UseDecl {
                name: alias,
                path_type,
                path_segments,
                ..
            } => {
                if !matches!(path_type, FolType::Package { .. }) {
                    return Err(manifest_item_error(
                        &parsed.syntax_index,
                        item.node_id,
                        "package manifest dependencies must use the 'pkg' source kind",
                    ));
                }
                let canonical_alias = fol_types::canonical_identifier_key(alias);
                if !dependency_aliases.insert(canonical_alias) {
                    return Err(manifest_item_error(
                        &parsed.syntax_index,
                        item.node_id,
                        format!(
                            "package manifest dependency alias '{}' is declared more than once",
                            alias
                        ),
                    ));
                }
                dependencies.push(ManifestDependency {
                    alias: alias.clone(),
                    package_path: path_segments.clone(),
                });
            }
            _ => {
                return Err(manifest_item_error(
                    &parsed.syntax_index,
                    item.node_id,
                    "package manifests may only contain comments, string fields, and pkg dependencies",
                ));
            }
        }
    }

    let name = name.ok_or_else(|| {
        ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "package manifest '{}' is missing required field 'name'",
                path.display()
            ),
        )
    })?;
    if !is_valid_package_name(&name) {
        return Err(ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "package manifest '{}' has invalid package name '{}'; package names must follow namespace identifier rules",
                path.display(),
                name
            ),
        ));
    }
    let version = version.ok_or_else(|| {
        ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "package manifest '{}' is missing required field 'version'",
                path.display()
            ),
        )
    })?;
    if version.trim().is_empty() {
        return Err(ResolverError::new(
            ResolverErrorKind::InvalidInput,
            format!(
                "package manifest '{}' has an empty version string",
                path.display()
            ),
        ));
    }

    Ok(PackageManifest {
        name,
        version,
        dependencies,
    })
}

fn manifest_string_field(
    field_name: &str,
    value: Option<&AstNode>,
    syntax_index: &SyntaxIndex,
    node_id: SyntaxNodeId,
) -> Result<String, ResolverError> {
    match value.map(unwrap_comment_wrappers) {
        Some(AstNode::Literal(Literal::String(value))) => Ok(value.clone()),
        Some(_) => Err(manifest_item_error(
            syntax_index,
            node_id,
            format!("package manifest field '{field_name}' must be a string literal"),
        )),
        None => Err(manifest_item_error(
            syntax_index,
            node_id,
            format!("package manifest field '{field_name}' must have a value"),
        )),
    }
}

fn unwrap_comment_wrappers(node: &AstNode) -> &AstNode {
    match node {
        AstNode::Commented { node, .. } => unwrap_comment_wrappers(node),
        other => other,
    }
}

fn manifest_item_error(
    syntax_index: &SyntaxIndex,
    node_id: SyntaxNodeId,
    message: impl Into<String>,
) -> ResolverError {
    match syntax_index.origin(node_id).cloned() {
        Some(origin) => ResolverError::with_origin(ResolverErrorKind::InvalidInput, message, origin),
        None => ResolverError::new(ResolverErrorKind::InvalidInput, message),
    }
}

fn is_valid_package_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii() || first.is_ascii_digit() || !(first.is_ascii_alphabetic() || first == '_')
    {
        return false;
    }

    if name.contains("__") {
        return false;
    }

    chars.all(|ch| ch.is_ascii() && (ch.is_ascii_alphanumeric() || ch == '_'))
}

#[cfg(test)]
mod tests {
    use super::{parse_package_manifest, ManifestDependency, PackageManifest};
    use crate::ResolverErrorKind;
    use fol_parser::ast::{UsePathSegment, UsePathSeparator};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fol_resolver_manifest_{}_{}_{}",
            label,
            std::process::id(),
            stamp
        ))
    }

    #[test]
    fn package_manifest_parser_extracts_identity_and_pkg_dependencies() {
        let temp_root = unique_temp_root("parse_manifest");
        fs::create_dir_all(&temp_root).expect("Should create temporary manifest fixture root");
        let manifest_path = temp_root.join("package.fol");
        fs::write(
            &manifest_path,
            concat!(
                "` package metadata `\n",
                "var name: str = \"json\";\n",
                "var version: str = \"1.0.0\";\n",
                "use serde: pkg = {serde};\n",
                "use fmt: pkg = {core/fmt};\n",
            ),
        )
        .expect("Should write the manifest fixture");

        let manifest =
            parse_package_manifest(&manifest_path).expect("Manifest fixture should parse");

        assert_eq!(
            manifest,
            PackageManifest {
                name: "json".to_string(),
                version: "1.0.0".to_string(),
                dependencies: vec![
                    ManifestDependency {
                        alias: "serde".to_string(),
                        package_path: vec![UsePathSegment {
                            separator: None,
                            spelling: "serde".to_string(),
                        }],
                    },
                    ManifestDependency {
                        alias: "fmt".to_string(),
                        package_path: vec![
                            UsePathSegment {
                                separator: None,
                                spelling: "core".to_string(),
                            },
                            UsePathSegment {
                                separator: Some(UsePathSeparator::Slash),
                                spelling: "fmt".to_string(),
                            },
                        ],
                    },
                ],
            }
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary manifest fixture root should be removable after the test");
    }

    #[test]
    fn package_manifest_parser_rejects_missing_required_fields() {
        let temp_root = unique_temp_root("missing_manifest_fields");
        fs::create_dir_all(&temp_root).expect("Should create temporary manifest fixture root");
        let manifest_path = temp_root.join("package.fol");
        fs::write(&manifest_path, "var name: str = \"json\";\n")
            .expect("Should write the incomplete manifest fixture");

        let error = parse_package_manifest(&manifest_path)
            .expect_err("Manifest parser should reject missing version fields");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error.to_string().contains("missing required field 'version'"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary manifest fixture root should be removable after the test");
    }

    #[test]
    fn package_manifest_parser_rejects_non_string_identity_fields() {
        let temp_root = unique_temp_root("manifest_non_string_field");
        fs::create_dir_all(&temp_root).expect("Should create temporary manifest fixture root");
        let manifest_path = temp_root.join("package.fol");
        fs::write(
            &manifest_path,
            "var name: str = 1;\nvar version: str = \"1.0.0\";\n",
        )
        .expect("Should write the malformed manifest fixture");

        let error = parse_package_manifest(&manifest_path)
            .expect_err("Manifest parser should reject non-string identity fields");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error.to_string().contains("field 'name' must be a string literal"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary manifest fixture root should be removable after the test");
    }

    #[test]
    fn package_manifest_parser_rejects_non_pkg_dependencies() {
        let temp_root = unique_temp_root("manifest_non_pkg_dep");
        fs::create_dir_all(&temp_root).expect("Should create temporary manifest fixture root");
        let manifest_path = temp_root.join("package.fol");
        fs::write(
            &manifest_path,
            concat!(
                "var name: str = \"json\";\n",
                "var version: str = \"1.0.0\";\n",
                "use fmt: loc = {\"../fmt\"};\n",
            ),
        )
        .expect("Should write the malformed manifest fixture");

        let error = parse_package_manifest(&manifest_path)
            .expect_err("Manifest parser should reject non-pkg dependencies");

        assert_eq!(error.kind(), ResolverErrorKind::InvalidInput);
        assert!(error.to_string().contains("must use the 'pkg' source kind"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary manifest fixture root should be removable after the test");
    }
}
