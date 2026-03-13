use crate::{PackageError, PackageErrorKind};
use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::{AstNode, AstParser, FolType, Literal, SyntaxIndex, SyntaxNodeId, SyntaxOrigin};
use fol_stream::FileStream;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildDependency {
    pub alias: String,
    pub package_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildExport {
    pub alias: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageBuildDefinition {
    pub dependencies: Vec<BuildDependency>,
    pub exports: Vec<BuildExport>,
}

pub fn parse_package_build(path: &Path) -> Result<PackageBuildDefinition, PackageError> {
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
    let parsed = parser.parse_package(&mut lexer).map_err(|errors| {
        let first = errors
            .into_iter()
            .next()
            .expect("build parser should produce at least one error");
        if let Some(parse_error) = first
            .as_ref()
            .as_any()
            .downcast_ref::<fol_parser::ast::ParseError>()
        {
            PackageError::with_origin(
                PackageErrorKind::InvalidInput,
                format!(
                    "package loader could not parse package build file '{}': {}",
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
            PackageError::new(
                PackageErrorKind::InvalidInput,
                format!(
                    "package loader could not parse package build file '{}': {}",
                    path.display(),
                    first
                ),
            )
        }
    })?;
    let source_unit = parsed.source_units.first().ok_or_else(|| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package build file '{}' did not produce any source units",
                path.display()
            ),
        )
    })?;

    let mut build = PackageBuildDefinition::default();
    for item in &source_unit.items {
        match unwrap_comment_wrappers(&item.node) {
            AstNode::Comment { .. } => {}
            AstNode::DefDecl {
                name,
                params,
                def_type,
                body,
                options,
            } => match def_type {
                FolType::Package { .. } => {
                    if !options.is_empty() {
                        return Err(build_item_error(
                            &parsed.syntax_index,
                            item.node_id,
                            "package build dependency definitions do not accept declaration options",
                        ));
                    }
                    if !params.is_empty() {
                        return Err(build_item_error(
                            &parsed.syntax_index,
                            item.node_id,
                            "package build dependency definitions do not accept parameters",
                        ));
                    }
                    build.dependencies.push(BuildDependency {
                        alias: name.clone(),
                        package_path: build_string_body(
                            "package dependency",
                            body,
                            &parsed.syntax_index,
                            item.node_id,
                        )?,
                    });
                }
                FolType::Location { .. } => {
                    if !options.is_empty() {
                        return Err(build_item_error(
                            &parsed.syntax_index,
                            item.node_id,
                            "package build export definitions do not accept declaration options",
                        ));
                    }
                    if !params.is_empty() {
                        return Err(build_item_error(
                            &parsed.syntax_index,
                            item.node_id,
                            "package build export definitions do not accept parameters",
                        ));
                    }
                    build.exports.push(BuildExport {
                        alias: name.clone(),
                        relative_path: build_string_body(
                            "export",
                            body,
                            &parsed.syntax_index,
                            item.node_id,
                        )?,
                    });
                }
                _ => {
                    return Err(build_item_error(
                        &parsed.syntax_index,
                        item.node_id,
                        "package build files currently accept only pkg dependency and loc export definitions",
                    ));
                }
            },
            AstNode::UseDecl { .. } => {
                return Err(build_item_error(
                    &parsed.syntax_index,
                    item.node_id,
                    "package build files must use 'def' to define dependencies; 'use' is not allowed here",
                ));
            }
            _ => {
                return Err(build_item_error(
                    &parsed.syntax_index,
                    item.node_id,
                    "package build files currently accept only comments, pkg dependency definitions, and loc export definitions",
                ));
            }
        }
    }

    Ok(build)
}

fn build_string_body(
    label: &str,
    body: &[AstNode],
    syntax_index: &SyntaxIndex,
    node_id: SyntaxNodeId,
) -> Result<String, PackageError> {
    match body {
        [AstNode::Literal(Literal::String(value))] => Ok(value.clone()),
        [AstNode::Commented { node, .. }] => match node.as_ref() {
            AstNode::Literal(Literal::String(value)) => Ok(value.clone()),
            _ => Err(build_item_error(
                syntax_index,
                node_id,
                format!("package build {label} targets must be string literals"),
            )),
        },
        _ => Err(build_item_error(
            syntax_index,
            node_id,
            format!("package build {label} targets must be string literals"),
        )),
    }
}

fn unwrap_comment_wrappers(node: &AstNode) -> &AstNode {
    match node {
        AstNode::Commented { node, .. } => unwrap_comment_wrappers(node),
        other => other,
    }
}

fn build_item_error(
    syntax_index: &SyntaxIndex,
    node_id: SyntaxNodeId,
    message: impl Into<String>,
) -> PackageError {
    match syntax_index.origin(node_id).cloned() {
        Some(origin) => PackageError::with_origin(PackageErrorKind::InvalidInput, message, origin),
        None => PackageError::new(PackageErrorKind::InvalidInput, message),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_package_build, BuildDependency, BuildExport, PackageBuildDefinition};
    use crate::PackageErrorKind;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_root(label: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "fol_package_build_definition_{}_{}_{}",
            label,
            std::process::id(),
            stamp
        ))
    }

    #[test]
    fn package_build_parser_extracts_pkg_dependency_definitions() {
        let temp_root = unique_temp_root("pkg_defs");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(
            &build_path,
            "def core: pkg = \"core\";\ndef tools: pkg = \"org/tools\";\n",
        )
        .expect("Should write the build dependency fixture");

        let build =
            parse_package_build(&build_path).expect("Build dependency fixture should parse");

        assert_eq!(
            build,
            PackageBuildDefinition {
                dependencies: vec![
                    BuildDependency {
                        alias: "core".to_string(),
                        package_path: "core".to_string(),
                    },
                    BuildDependency {
                        alias: "tools".to_string(),
                        package_path: "org/tools".to_string(),
                    },
                ],
                exports: Vec::new(),
            }
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_rejects_non_string_dependency_targets() {
        let temp_root = unique_temp_root("pkg_non_string_target");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, "def core: pkg = core;\n")
            .expect("Should write the invalid build dependency fixture");

        let error = parse_package_build(&build_path)
            .expect_err("Non-string build dependency targets should be rejected");

        assert_eq!(error.kind(), PackageErrorKind::InvalidInput);
        assert!(error
            .to_string()
            .contains("package build package dependency targets must be string literals"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_extracts_loc_export_definitions() {
        let temp_root = unique_temp_root("loc_defs");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(
            &build_path,
            "def root: loc = \"src\";\ndef fmt: loc = \"src/fmt\";\n",
        )
        .expect("Should write the build export fixture");

        let build = parse_package_build(&build_path).expect("Build export fixture should parse");

        assert_eq!(
            build,
            PackageBuildDefinition {
                dependencies: Vec::new(),
                exports: vec![
                    BuildExport {
                        alias: "root".to_string(),
                        relative_path: "src".to_string(),
                    },
                    BuildExport {
                        alias: "fmt".to_string(),
                        relative_path: "src/fmt".to_string(),
                    },
                ],
            }
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_rejects_non_string_export_targets() {
        let temp_root = unique_temp_root("loc_non_string_target");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, "def root: loc = src;\n")
            .expect("Should write the invalid build export fixture");

        let error = parse_package_build(&build_path)
            .expect_err("Non-string build export targets should be rejected");

        assert_eq!(error.kind(), PackageErrorKind::InvalidInput);
        assert!(error
            .to_string()
            .contains("package build export targets must be string literals"));

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_rejects_use_declarations_with_exact_locations() {
        let temp_root = unique_temp_root("build_use_decl");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, "use core: pkg = {core};\n")
            .expect("Should write the invalid build use fixture");

        let error = parse_package_build(&build_path)
            .expect_err("Build files should reject use declarations");

        assert_eq!(error.kind(), PackageErrorKind::InvalidInput);
        assert!(error
            .to_string()
            .contains("must use 'def' to define dependencies"));
        let origin = error
            .origin()
            .expect("Build-file validation errors should retain syntax origins");
        assert_eq!(origin.file.as_deref(), build_path.to_str());
        assert_eq!(origin.line, 1);
        assert_eq!(origin.column, 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_rejects_unsupported_top_level_nodes_with_exact_locations() {
        let temp_root = unique_temp_root("build_var_decl");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, "var name: str = \"json\";\n")
            .expect("Should write the invalid build declaration fixture");

        let error = parse_package_build(&build_path)
            .expect_err("Build files should reject unsupported top-level nodes");

        assert_eq!(error.kind(), PackageErrorKind::InvalidInput);
        assert!(error.to_string().contains(
            "package build files currently accept only comments, pkg dependency definitions, and loc export definitions"
        ));
        let origin = error
            .origin()
            .expect("Unsupported build nodes should keep syntax origins");
        assert_eq!(origin.file.as_deref(), build_path.to_str());
        assert_eq!(origin.line, 1);
        assert_eq!(origin.column, 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }
}
