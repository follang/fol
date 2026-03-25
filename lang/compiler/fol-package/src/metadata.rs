use crate::{PackageError, PackageErrorKind};
use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::{AstNode, AstParser, CallSurface, ParsedPackage, SyntaxOrigin};
use fol_stream::FileStream;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedPackageMetadataField {
    pub name: String,
    pub value: String,
    pub origin: Option<SyntaxOrigin>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedPackageDependencyDecl {
    pub alias: String,
    pub source: String,
    pub target: String,
    pub origin: Option<SyntaxOrigin>,
}

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

pub fn parse_package_metadata_from_build(path: &Path) -> Result<PackageMetadata, PackageError> {
    let fields = extract_package_metadata_fields_from_build(path)?;
    let dependencies = extract_package_dependencies_from_build(path)?;
    materialize_package_metadata_from_build(path, fields, dependencies)
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

pub fn extract_package_metadata_fields_from_build(
    path: &Path,
) -> Result<Vec<ExtractedPackageMetadataField>, PackageError> {
    let parsed = parse_build_package_for_metadata(path)?;
    let build_body = canonical_build_body(&parsed).ok_or_else(|| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "build.fol '{}' must declare exactly one canonical `pro[] build(): non` entry",
                path.display()
            ),
        )
    })?;
    let mut fields = Vec::new();
    let mut build_aliases = BTreeSet::new();
    let mut string_bindings = BTreeMap::new();
    collect_metadata_fields(
        build_body,
        &parsed,
        &mut build_aliases,
        &mut string_bindings,
        &mut fields,
    );
    Ok(fields)
}

pub fn extract_package_dependencies_from_build(
    path: &Path,
) -> Result<Vec<ExtractedPackageDependencyDecl>, PackageError> {
    let parsed = parse_build_package_for_metadata(path)?;
    let build_body = canonical_build_body(&parsed).ok_or_else(|| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "build.fol '{}' must declare exactly one canonical `pro[] build(): non` entry",
                path.display()
            ),
        )
    })?;
    let mut dependencies = Vec::new();
    let mut build_aliases = BTreeSet::new();
    let mut string_bindings = BTreeMap::new();
    collect_dependency_decls(
        build_body,
        &parsed,
        &mut build_aliases,
        &mut string_bindings,
        &mut dependencies,
    );
    Ok(dependencies)
}

fn parse_build_package_for_metadata(path: &Path) -> Result<ParsedPackage, PackageError> {
    let path_str = path.to_str().ok_or_else(|| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!("package build file '{}' is not valid UTF-8", path.display()),
        )
    })?;
    let mut stream = FileStream::from_file(path_str).map_err(|error| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package loader could not read package build file '{}': {}",
                path.display(),
                error
            ),
        )
    })?;
    let mut lexer = Elements::init(&mut stream);
    let mut parser = AstParser::new();
    parser.parse_package(&mut lexer).map_err(|diagnostics| {
        let message = diagnostics
            .into_iter()
            .next()
            .map(|d| d.message)
            .unwrap_or_else(|| "unknown parse error".to_string());
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package loader could not parse package build file '{}': {}",
                path.display(),
                message
            ),
        )
    })
}

fn canonical_build_body<'a>(parsed: &'a ParsedPackage) -> Option<&'a [AstNode]> {
    let mut found = None;
    for unit in &parsed.source_units {
        for item in &unit.items {
            let AstNode::ProDecl {
                name,
                params,
                return_type,
                body,
                ..
            } = &item.node
            else {
                continue;
            };
            if name != "build" || !params.is_empty() {
                continue;
            }
            let Some(return_type_name) = (match return_type.as_ref() {
                Some(fol_parser::ast::FolType::None) => Some("non".to_string()),
                Some(ty) => ty.named_text(),
                None => None,
            }) else {
                continue;
            };
            if return_type_name != "non" && return_type_name != "none" {
                continue;
            }
            if found.is_some() {
                return None;
            }
            found = Some(body.as_slice());
        }
    }
    found
}

fn collect_metadata_fields(
    nodes: &[AstNode],
    parsed: &ParsedPackage,
    build_aliases: &mut BTreeSet<String>,
    string_bindings: &mut BTreeMap<String, String>,
    fields: &mut Vec<ExtractedPackageMetadataField>,
) {
    for node in nodes {
        match node {
            AstNode::VarDecl {
                name,
                value: Some(value),
                ..
            } => {
                if is_ambient_build(value) {
                    build_aliases.insert(name.clone());
                } else if let Some(literal) = resolve_string_value(value, string_bindings) {
                    string_bindings.insert(name.clone(), literal);
                }
            }
            AstNode::MethodCall { object, method, args } if method == "meta" => {
                if !is_build_receiver(object, build_aliases) {
                    continue;
                }
                let [AstNode::RecordInit { syntax_id, fields: record_fields, .. }] = &args[..] else {
                    continue;
                };
                let origin = syntax_id.and_then(|id| parsed.syntax_index.origin(id).cloned());
                for field in record_fields {
                    if let Some(value) = resolve_string_value(&field.value, string_bindings) {
                        fields.push(ExtractedPackageMetadataField {
                            name: field.name.clone(),
                            value,
                            origin: origin.clone(),
                        });
                    }
                }
            }
            AstNode::When { cases, default, .. } => {
                for case in cases {
                    if let fol_parser::ast::WhenCase::Case { body, .. } = case {
                        let mut aliases = build_aliases.clone();
                        let mut bindings = string_bindings.clone();
                        collect_metadata_fields(body, parsed, &mut aliases, &mut bindings, fields);
                    }
                }
                if let Some(default_body) = default {
                    let mut aliases = build_aliases.clone();
                    let mut bindings = string_bindings.clone();
                    collect_metadata_fields(default_body, parsed, &mut aliases, &mut bindings, fields);
                }
            }
            AstNode::Loop { body, .. } | AstNode::Defer { body, .. } => {
                let mut aliases = build_aliases.clone();
                let mut bindings = string_bindings.clone();
                collect_metadata_fields(body, parsed, &mut aliases, &mut bindings, fields);
            }
            AstNode::Block { statements, .. } => {
                let mut aliases = build_aliases.clone();
                let mut bindings = string_bindings.clone();
                collect_metadata_fields(statements, parsed, &mut aliases, &mut bindings, fields);
            }
            _ => {}
        }
    }
}

fn collect_dependency_decls(
    nodes: &[AstNode],
    parsed: &ParsedPackage,
    build_aliases: &mut BTreeSet<String>,
    string_bindings: &mut BTreeMap<String, String>,
    dependencies: &mut Vec<ExtractedPackageDependencyDecl>,
) {
    for node in nodes {
        match node {
            AstNode::VarDecl {
                name,
                value: Some(value),
                ..
            } => {
                if is_ambient_build(value) {
                    build_aliases.insert(name.clone());
                } else if let Some(literal) = resolve_string_value(value, string_bindings) {
                    string_bindings.insert(name.clone(), literal);
                }
            }
            AstNode::MethodCall { object, method, args } if method == "add_dep" => {
                if !is_build_receiver(object, build_aliases) {
                    continue;
                }
                let [AstNode::RecordInit { syntax_id, fields: record_fields, .. }] = &args[..] else {
                    continue;
                };
                let origin = syntax_id.and_then(|id| parsed.syntax_index.origin(id).cloned());
                let mut alias = None;
                let mut source = None;
                let mut target = None;
                for field in record_fields {
                    let value = resolve_string_value(&field.value, string_bindings);
                    match field.name.as_str() {
                        "alias" => alias = value,
                        "source" => source = value,
                        "target" => target = value,
                        _ => {}
                    }
                }
                if let (Some(alias), Some(source), Some(target)) = (alias, source, target) {
                    dependencies.push(ExtractedPackageDependencyDecl {
                        alias,
                        source,
                        target,
                        origin,
                    });
                }
            }
            AstNode::When { cases, default, .. } => {
                for case in cases {
                    if let fol_parser::ast::WhenCase::Case { body, .. } = case {
                        let mut aliases = build_aliases.clone();
                        let mut bindings = string_bindings.clone();
                        collect_dependency_decls(
                            body,
                            parsed,
                            &mut aliases,
                            &mut bindings,
                            dependencies,
                        );
                    }
                }
                if let Some(default_body) = default {
                    let mut aliases = build_aliases.clone();
                    let mut bindings = string_bindings.clone();
                    collect_dependency_decls(
                        default_body,
                        parsed,
                        &mut aliases,
                        &mut bindings,
                        dependencies,
                    );
                }
            }
            AstNode::Loop { body, .. } | AstNode::Defer { body, .. } => {
                let mut aliases = build_aliases.clone();
                let mut bindings = string_bindings.clone();
                collect_dependency_decls(body, parsed, &mut aliases, &mut bindings, dependencies);
            }
            AstNode::Block { statements, .. } => {
                let mut aliases = build_aliases.clone();
                let mut bindings = string_bindings.clone();
                collect_dependency_decls(
                    statements,
                    parsed,
                    &mut aliases,
                    &mut bindings,
                    dependencies,
                );
            }
            _ => {}
        }
    }
}

fn materialize_package_metadata_from_build(
    path: &Path,
    fields: Vec<ExtractedPackageMetadataField>,
    dependencies: Vec<ExtractedPackageDependencyDecl>,
) -> Result<PackageMetadata, PackageError> {
    let supported_fields = BTreeSet::from(["name", "version", "kind", "description", "license"]);
    let mut values = BTreeMap::<String, (String, Option<SyntaxOrigin>)>::new();

    for field in fields {
        if !supported_fields.contains(field.name.as_str()) {
            let error = match field.origin {
                Some(origin) => PackageError::with_origin(
                    PackageErrorKind::InvalidInput,
                    format!(
                        "unsupported package metadata field '{}'; expected only name, version, kind, description, or license",
                        field.name
                    ),
                    origin,
                ),
                None => PackageError::new(
                    PackageErrorKind::InvalidInput,
                    format!(
                        "unsupported package metadata field '{}'; expected only name, version, kind, description, or license",
                        field.name
                    ),
                ),
            };
            return Err(error);
        }
        if let Some((_, first_origin)) = values.insert(
            field.name.clone(),
            (field.value.clone(), field.origin.clone()),
        ) {
            let error = match field.origin {
                Some(origin) => PackageError::with_origin(
                    PackageErrorKind::InvalidInput,
                    format!(
                        "package metadata field '{}' may only be declared once",
                        field.name
                    ),
                    origin,
                ),
                None => PackageError::new(
                    PackageErrorKind::InvalidInput,
                    format!(
                        "package metadata field '{}' may only be declared once",
                        field.name
                    ),
                ),
            };
            return Err(match first_origin {
                Some(origin) => error
                    .with_related_origin(origin, "first package metadata field declaration"),
                None => error,
            });
        }
    }

    let name = values
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

    let version = values
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

    let mut materialized_deps = Vec::new();
    let mut dep_aliases = BTreeMap::<String, Option<SyntaxOrigin>>::new();
    for dependency in dependencies {
        if let Some(first_origin) =
            dep_aliases.insert(dependency.alias.clone(), dependency.origin.clone())
        {
            let error = match dependency.origin {
                Some(origin) => PackageError::with_origin(
                    PackageErrorKind::InvalidInput,
                    format!(
                        "package dependency alias '{}' in '{}' may only be declared once",
                        dependency.alias,
                        path.display()
                    ),
                    origin,
                ),
                None => PackageError::new(
                    PackageErrorKind::InvalidInput,
                    format!(
                        "package dependency alias '{}' in '{}' may only be declared once",
                        dependency.alias,
                        path.display()
                    ),
                ),
            };
            return Err(match first_origin {
                Some(origin) => error
                    .with_related_origin(origin, "first package dependency alias declaration"),
                None => error,
            });
        }
        let source_target = format!("{}:{}", dependency.source, dependency.target);
        materialized_deps.push(parse_dependency_decl(
            path,
            &dependency.alias,
            &source_target,
            dependency.origin.unwrap_or_else(|| SyntaxOrigin {
                file: Some(path.to_string_lossy().to_string()),
                line: 1,
                column: 1,
                length: dependency.alias.len().max(1),
            }),
        )?);
    }

    Ok(PackageMetadata {
        name,
        version,
        kind: non_empty_optional_field(
            path,
            "kind",
            values.remove("kind").map(|(value, _)| value),
        )?,
        description: non_empty_optional_field(
            path,
            "description",
            values.remove("description").map(|(value, _)| value),
        )?,
        license: non_empty_optional_field(
            path,
            "license",
            values.remove("license").map(|(value, _)| value),
        )?,
        dependencies: materialized_deps,
    })
}

fn is_ambient_build(node: &AstNode) -> bool {
    matches!(
        node,
        AstNode::FunctionCall {
            surface: CallSurface::DotIntrinsic,
            name,
            args,
            ..
        } if name == "build" && args.is_empty()
    )
}

fn is_build_receiver(node: &AstNode, build_aliases: &BTreeSet<String>) -> bool {
    is_ambient_build(node)
        || matches!(
            node,
            AstNode::Identifier { name, .. } if build_aliases.contains(name)
        )
}

fn resolve_string_value(node: &AstNode, string_bindings: &BTreeMap<String, String>) -> Option<String> {
    match node {
        AstNode::Literal(fol_parser::ast::Literal::String(value)) => Some(value.clone()),
        AstNode::Identifier { name, .. } => string_bindings.get(name).cloned(),
        _ => None,
    }
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
        extract_package_dependencies_from_build, extract_package_metadata_fields_from_build,
        parse_package_metadata, parse_package_metadata_from_build, PackageDependencyDecl,
        PackageDependencySourceKind, PackageMetadata,
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

    fn write_build_fixture(label: &str, source: &str) -> std::path::PathBuf {
        let temp_root = unique_temp_root(label);
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, source).expect("Should write the build fixture");
        build_path
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
    fn build_metadata_extractor_reads_direct_meta_fields() {
        let build_path = write_build_fixture(
            "build_meta_direct",
            concat!(
                "pro[] build(): non = {\n",
                "    .build().meta({\n",
                "        name = \"demo\",\n",
                "        version = \"1.0.0\",\n",
                "        kind = \"exe\",\n",
                "    });\n",
                "}\n",
            ),
        );

        let fields = extract_package_metadata_fields_from_build(&build_path)
            .expect("build metadata should extract from direct meta call");

        assert!(fields.iter().any(|field| field.name == "name" && field.value == "demo"));
        assert!(fields
            .iter()
            .any(|field| field.name == "version" && field.value == "1.0.0"));
        assert!(fields.iter().any(|field| field.name == "kind" && field.value == "exe"));

        fs::remove_dir_all(build_path.parent().unwrap()).ok();
    }

    #[test]
    fn build_metadata_extractor_reads_string_bindings_through_build_local() {
        let build_path = write_build_fixture(
            "build_meta_local_bindings",
            concat!(
                "pro[] build(): non = {\n",
                "    var build = .build();\n",
                "    var name = \"demo\";\n",
                "    var version = \"1.0.0\";\n",
                "    build.meta({\n",
                "        name = name,\n",
                "        version = version,\n",
                "    });\n",
                "}\n",
            ),
        );

        let fields = extract_package_metadata_fields_from_build(&build_path)
            .expect("build metadata should extract through inferred build locals");

        assert_eq!(fields.len(), 2);
        assert!(fields.iter().any(|field| field.name == "name" && field.value == "demo"));
        assert!(fields
            .iter()
            .any(|field| field.name == "version" && field.value == "1.0.0"));

        fs::remove_dir_all(build_path.parent().unwrap()).ok();
    }

    #[test]
    fn build_metadata_materializer_requires_name_and_version() {
        let build_path = write_build_fixture(
            "build_meta_missing_required",
            concat!(
                "pro[] build(): non = {\n",
                "    .build().meta({\n",
                "        description = \"demo\",\n",
                "    });\n",
                "}\n",
            ),
        );

        let error = parse_package_metadata_from_build(&build_path)
            .expect_err("missing required build metadata should be rejected");

        assert!(error
            .to_string()
            .contains("missing required field 'name'"));

        fs::remove_dir_all(build_path.parent().unwrap()).ok();
    }

    #[test]
    fn build_metadata_materializer_rejects_duplicate_fields() {
        let build_path = write_build_fixture(
            "build_meta_duplicate_field",
            concat!(
                "pro[] build(): non = {\n",
                "    .build().meta({ name = \"demo\", version = \"1.0.0\" });\n",
                "    .build().meta({ name = \"other\" });\n",
                "}\n",
            ),
        );

        let diagnostic = parse_package_metadata_from_build(&build_path)
            .expect_err("duplicate build metadata fields should be rejected")
            .to_diagnostic();

        assert_eq!(diagnostic.labels.len(), 2);
        assert_eq!(
            diagnostic.labels[1].message.as_deref(),
            Some("first package metadata field declaration")
        );

        fs::remove_dir_all(build_path.parent().unwrap()).ok();
    }

    #[test]
    fn build_metadata_materializer_rejects_invalid_package_names() {
        let build_path = write_build_fixture(
            "build_meta_invalid_name",
            concat!(
                "pro[] build(): non = {\n",
                "    .build().meta({ name = \"9demo\", version = \"1.0.0\" });\n",
                "}\n",
            ),
        );

        let error = parse_package_metadata_from_build(&build_path)
            .expect_err("invalid package names should be rejected");

        assert!(error.to_string().contains("invalid package name '9demo'"));

        fs::remove_dir_all(build_path.parent().unwrap()).ok();
    }

    #[test]
    fn build_dependency_extractor_reads_direct_dependency_configs() {
        let build_path = write_build_fixture(
            "build_dep_extract_direct",
            concat!(
                "pro[] build(): non = {\n",
                "    .build().add_dep({ alias = \"core\", source = \"pkg\", target = \"core/tools\" });\n",
                "    .build().add_dep({ alias = \"shared\", source = \"loc\", target = \"../shared\" });\n",
                "}\n",
            ),
        );

        let deps = extract_package_dependencies_from_build(&build_path)
            .expect("build dependencies should extract from direct calls");

        assert_eq!(deps.len(), 2);
        assert!(deps
            .iter()
            .any(|dep| dep.alias == "core" && dep.source == "pkg" && dep.target == "core/tools"));
        assert!(deps
            .iter()
            .any(|dep| dep.alias == "shared" && dep.source == "loc" && dep.target == "../shared"));

        fs::remove_dir_all(build_path.parent().unwrap()).ok();
    }

    #[test]
    fn build_metadata_materializer_materializes_direct_dependencies() {
        let build_path = write_build_fixture(
            "build_dep_materialize",
            concat!(
                "pro[] build(): non = {\n",
                "    var build = .build();\n",
                "    build.meta({ name = \"demo\", version = \"1.0.0\" });\n",
                "    build.add_dep({ alias = \"core\", source = \"pkg\", target = \"core\" });\n",
                "    build.add_dep({ alias = \"logtiny\", source = \"git\", target = \"https://github.com/bresilla/logtiny.git\" });\n",
                "}\n",
            ),
        );

        let metadata = parse_package_metadata_from_build(&build_path)
            .expect("build metadata should materialize direct dependencies");

        assert_eq!(metadata.name, "demo");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.dependencies.len(), 2);
        assert_eq!(metadata.dependencies[0].alias, "core");
        assert_eq!(
            metadata.dependencies[0].source_kind,
            PackageDependencySourceKind::PackageStore
        );
        assert_eq!(metadata.dependencies[1].alias, "logtiny");
        assert_eq!(
            metadata.dependencies[1].source_kind,
            PackageDependencySourceKind::Git
        );

        fs::remove_dir_all(build_path.parent().unwrap()).ok();
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
