use crate::{parse_package_locator, PackageError, PackageErrorKind, PackageLocator};
use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::{
    AstNode, AstParser, FolType, Literal, ParsedPackage, SyntaxIndex, SyntaxNodeId, SyntaxOrigin,
};
use fol_stream::FileStream;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildDependency {
    pub alias: String,
    pub locator: PackageLocator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildExport {
    pub alias: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageNativeArtifactKind {
    Header,
    Object,
    StaticLibrary,
    SharedLibrary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageNativeArtifact {
    pub alias: String,
    pub kind: PackageNativeArtifactKind,
    pub relative_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageBuildCompatibility {
    pub dependencies: Vec<BuildDependency>,
    pub exports: Vec<BuildExport>,
    pub native_artifacts: Vec<PackageNativeArtifact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageBuildEntryPointKind {
    BuildFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageBuildMode {
    Empty,
    CompatibilityOnly,
    ModernOnly,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageBuildEntryPoint {
    pub kind: PackageBuildEntryPointKind,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackageBuildDefinition {
    pub compatibility: PackageBuildCompatibility,
    pub entry_point: Option<PackageBuildEntryPoint>,
}

impl PackageBuildDefinition {
    pub fn compatibility(&self) -> &PackageBuildCompatibility {
        &self.compatibility
    }

    pub fn dependencies(&self) -> &[BuildDependency] {
        &self.compatibility.dependencies
    }

    pub fn exports(&self) -> &[BuildExport] {
        &self.compatibility.exports
    }

    pub fn native_artifacts(&self) -> &[PackageNativeArtifact] {
        &self.compatibility.native_artifacts
    }

    pub fn has_compatibility_controls(&self) -> bool {
        !self.dependencies().is_empty()
            || !self.exports().is_empty()
            || !self.native_artifacts().is_empty()
    }

    pub fn entry_point(&self) -> Option<&PackageBuildEntryPoint> {
        self.entry_point.as_ref()
    }

    pub fn has_entry_point(&self) -> bool {
        self.entry_point().is_some()
    }

    pub fn mode(&self) -> PackageBuildMode {
        match (self.has_compatibility_controls(), self.has_entry_point()) {
            (false, false) => PackageBuildMode::Empty,
            (true, false) => PackageBuildMode::CompatibilityOnly,
            (false, true) => PackageBuildMode::ModernOnly,
            (true, true) => PackageBuildMode::Hybrid,
        }
    }
}

pub fn parse_package_build(path: &Path) -> Result<PackageBuildDefinition, PackageError> {
    let path_str = path.to_str().ok_or_else(|| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!("package build file '{}' is not valid UTF-8", path.display()),
        )
    })?;
    let source = std::fs::read_to_string(path).map_err(|error| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package loader could not read package build file '{}': {}",
                path.display(),
                error
            ),
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
    let parsed = match parser.parse_package(&mut lexer) {
        Ok(parsed) => parsed,
        Err(errors) => {
            let parse_error = build_parse_error(path, errors);
            if let Some(build) = extract_package_build_definition_from_source_fallback(&source)? {
                return Ok(build);
            }
            return Err(parse_error);
        }
    };
    extract_package_build_definition(&parsed)
}

fn build_parse_error(
    path: &Path,
    errors: Vec<Box<dyn fol_types::Glitch>>,
) -> PackageError {
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
}

fn extract_package_build_definition_from_source_fallback(
    source: &str,
) -> Result<Option<PackageBuildDefinition>, PackageError> {
    let mut build = PackageBuildDefinition::default();

    for raw_line in source.lines() {
        let line = strip_build_line_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if try_record_fallback_build_entry(&mut build, line) {
            continue;
        }
        if try_record_fallback_compatibility_def(&mut build, line)? {
            continue;
        }
    }

    if build.has_compatibility_controls() || build.has_entry_point() {
        Ok(Some(build))
    } else {
        Ok(None)
    }
}

fn strip_build_line_comment(line: &str) -> &str {
    line.split_once("//").map_or(line, |(prefix, _)| prefix)
}

fn try_record_fallback_build_entry(build: &mut PackageBuildDefinition, line: &str) -> bool {
    if !line.starts_with("def build(") || build.entry_point.is_some() {
        return false;
    }
    build.entry_point = Some(PackageBuildEntryPoint {
        kind: PackageBuildEntryPointKind::BuildFunction,
        name: "build".to_string(),
    });
    true
}

fn try_record_fallback_compatibility_def(
    build: &mut PackageBuildDefinition,
    line: &str,
) -> Result<bool, PackageError> {
    if !line.starts_with("def ") {
        return Ok(false);
    }
    let Some(rest) = line.strip_prefix("def ") else {
        return Ok(false);
    };
    let Some((head, body)) = rest.split_once('=') else {
        return Ok(false);
    };
    let Some((alias, type_name)) = head.split_once(':') else {
        return Ok(false);
    };
    if alias.contains('(') {
        return Ok(false);
    }

    let alias = alias.trim();
    let type_name = type_name.trim();
    let body = body.trim().trim_end_matches(';').trim();
    let Some(string_body) = unquote_build_string(body) else {
        return Ok(false);
    };

    match type_name {
        "pkg" => {
            build.compatibility.dependencies.push(BuildDependency {
                alias: alias.to_string(),
                locator: parse_package_locator(string_body)?,
            });
            Ok(true)
        }
        "loc" => {
            build.compatibility.exports.push(BuildExport {
                alias: alias.to_string(),
                relative_path: string_body.to_string(),
            });
            Ok(true)
        }
        other => {
            if let Some(kind) = native_artifact_kind(other) {
                build.compatibility.native_artifacts.push(PackageNativeArtifact {
                    alias: alias.to_string(),
                    kind,
                    relative_path: string_body.to_string(),
                });
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}

fn unquote_build_string(body: &str) -> Option<&str> {
    body.strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
}

pub fn extract_package_build_definition(
    parsed: &ParsedPackage,
) -> Result<PackageBuildDefinition, PackageError> {
    let source_unit = parsed.source_units.first().ok_or_else(|| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            "package build parsing did not produce any source units",
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
                    let dependency_target = build_string_body(
                        "package dependency",
                        body,
                        &parsed.syntax_index,
                        item.node_id,
                    )?;
                    build.compatibility.dependencies.push(BuildDependency {
                        alias: name.clone(),
                        locator: parse_package_locator(&dependency_target).map_err(|error| {
                            build_item_error(
                                &parsed.syntax_index,
                                item.node_id,
                                error.message().to_string(),
                            )
                        })?,
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
                    build.compatibility.exports.push(BuildExport {
                        alias: name.clone(),
                        relative_path: build_string_body(
                            "export",
                            body,
                            &parsed.syntax_index,
                            item.node_id,
                        )?,
                    });
                }
                FolType::Named { name: type_name, .. } => {
                    if let Some(kind) = native_artifact_kind(type_name.as_str()) {
                        if !options.is_empty() {
                            return Err(build_item_error(
                                &parsed.syntax_index,
                                item.node_id,
                                "package native artifact definitions do not accept declaration options",
                            ));
                        }
                        if !params.is_empty() {
                            return Err(build_item_error(
                                &parsed.syntax_index,
                                item.node_id,
                                "package native artifact definitions do not accept parameters",
                            ));
                        }
                        build.compatibility.native_artifacts.push(PackageNativeArtifact {
                            alias: name.clone(),
                            kind,
                            relative_path: build_string_body(
                                "native artifact",
                                body,
                                &parsed.syntax_index,
                                item.node_id,
                            )?,
                        });
                    } else {
                        maybe_record_build_entry_point(&mut build, name, params);
                    }
                }
                _ => {
                    maybe_record_build_entry_point(&mut build, name, params);
                }
            },
            _ => {}
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

fn native_artifact_kind(name: &str) -> Option<PackageNativeArtifactKind> {
    match name {
        "header" => Some(PackageNativeArtifactKind::Header),
        "object" => Some(PackageNativeArtifactKind::Object),
        "static_lib" => Some(PackageNativeArtifactKind::StaticLibrary),
        "shared_lib" => Some(PackageNativeArtifactKind::SharedLibrary),
        _ => None,
    }
}

fn maybe_record_build_entry_point(
    build: &mut PackageBuildDefinition,
    name: &str,
    params: &[fol_parser::ast::Parameter],
) {
    if name == "build" && !params.is_empty() && build.entry_point.is_none() {
        build.entry_point = Some(PackageBuildEntryPoint {
            kind: PackageBuildEntryPointKind::BuildFunction,
            name: name.to_string(),
        });
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
    use super::{
        extract_package_build_definition, extract_package_build_definition_from_source_fallback,
        parse_package_build, BuildDependency, BuildExport, PackageBuildCompatibility,
        PackageBuildDefinition, PackageBuildEntryPoint, PackageBuildEntryPointKind,
        PackageBuildMode, PackageNativeArtifact, PackageNativeArtifactKind,
    };
    use crate::{PackageErrorKind, PackageLocator};
    use fol_parser::ast::AstParser;
    use fol_stream::FileStream;
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
                compatibility: PackageBuildCompatibility {
                    dependencies: vec![
                        BuildDependency {
                            alias: "core".to_string(),
                            locator: PackageLocator::installed_store(
                                "core",
                                vec!["core".to_string()],
                            ),
                        },
                        BuildDependency {
                            alias: "tools".to_string(),
                            locator: PackageLocator::installed_store(
                                "org/tools",
                                vec!["org".to_string(), "tools".to_string()],
                            ),
                        },
                    ],
                    exports: Vec::new(),
                    native_artifacts: Vec::new(),
                },
                entry_point: None,
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
        let origin = error
            .origin()
            .expect("Invalid package dependency extraction should keep syntax origins");
        assert_eq!(origin.file.as_deref(), build_path.to_str());
        assert_eq!(origin.line, 1);
        assert_eq!(origin.column, 1);

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
                compatibility: PackageBuildCompatibility {
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
                    native_artifacts: Vec::new(),
                },
                entry_point: None,
            }
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_extracts_reserved_native_artifact_placeholders() {
        let temp_root = unique_temp_root("native_artifact_defs");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(
            &build_path,
            concat!(
                "def api: header = \"include/api.h\";\n",
                "def math_obj: object = \"build/math.o\";\n",
                "def c_runtime: static_lib = \"build/libc_runtime.a\";\n",
                "def plugin: shared_lib = \"build/libplugin.so\";\n",
            ),
        )
        .expect("Should write the build native artifact fixture");

        let build =
            parse_package_build(&build_path).expect("Build native artifact fixture should parse");

        assert_eq!(
            build.native_artifacts(),
            vec![
                PackageNativeArtifact {
                    alias: "api".to_string(),
                    kind: PackageNativeArtifactKind::Header,
                    relative_path: "include/api.h".to_string(),
                },
                PackageNativeArtifact {
                    alias: "math_obj".to_string(),
                    kind: PackageNativeArtifactKind::Object,
                    relative_path: "build/math.o".to_string(),
                },
                PackageNativeArtifact {
                    alias: "c_runtime".to_string(),
                    kind: PackageNativeArtifactKind::StaticLibrary,
                    relative_path: "build/libc_runtime.a".to_string(),
                },
                PackageNativeArtifact {
                    alias: "plugin".to_string(),
                    kind: PackageNativeArtifactKind::SharedLibrary,
                    relative_path: "build/libplugin.so".to_string(),
                },
            ],
        );
        assert!(build.dependencies().is_empty());
        assert!(build.exports().is_empty());

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
        let origin = error
            .origin()
            .expect("Invalid package export extraction should keep syntax origins");
        assert_eq!(origin.file.as_deref(), build_path.to_str());
        assert_eq!(origin.line, 1);
        assert_eq!(origin.column, 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_keeps_exact_origins_for_parse_failures() {
        let temp_root = unique_temp_root("build_parse_origin");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, "def root: loc = {\n")
            .expect("Should write the malformed build fixture");

        let error = parse_package_build(&build_path)
            .expect_err("Malformed build files should preserve parse-error origins");

        assert_eq!(error.kind(), PackageErrorKind::InvalidInput);
        let origin = error
            .origin()
            .expect("Malformed build parse failures should keep exact origins");
        assert_eq!(origin.file.as_deref(), build_path.to_str());
        assert_eq!(origin.line, 1);
        assert!(origin.column >= 1);

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

        let build = parse_package_build(&build_path)
            .expect("Build files should parse ordinary use declarations even though they do not define package edges");

        assert!(build.dependencies().is_empty());
        assert!(build.exports().is_empty());
        assert!(build.native_artifacts().is_empty());

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_ignores_unrelated_top_level_fol_declarations() {
        let temp_root = unique_temp_root("build_var_decl");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, "var name: str = \"json\";\n")
            .expect("Should write the invalid build declaration fixture");

        let build = parse_package_build(&build_path)
            .expect("Build files should allow unrelated top-level FOL declarations in phase one");

        assert!(build.dependencies().is_empty());
        assert!(build.exports().is_empty());
        assert!(build.native_artifacts().is_empty());

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_extractor_keeps_recognized_defs_from_mixed_ordinary_build_files() {
        let temp_root = unique_temp_root("mixed_build_file");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(
            &build_path,
            concat!(
                "var name: str = \"json\";\n",
                "use fmt: loc = {\"../fmt\"};\n",
                "fun[] helper(): int = { return 1; }\n",
                "def core: pkg = \"core\";\n",
                "def root: loc = \"src\";\n",
                "def build(graph: int): int = graph;\n",
            ),
        )
        .expect("Should write the mixed build fixture");
        let mut stream = FileStream::from_file(
            build_path
                .to_str()
                .expect("Temporary build fixture path should be valid UTF-8"),
        )
        .expect("Should open the mixed build fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let parsed = parser
            .parse_package(&mut lexer)
            .expect("Mixed build fixture should parse as ordinary FOL");

        let build = extract_package_build_definition(&parsed)
            .expect("Package extraction should keep only recognized build definitions");

        assert_eq!(
            build,
            PackageBuildDefinition {
                compatibility: PackageBuildCompatibility {
                    dependencies: vec![BuildDependency {
                        alias: "core".to_string(),
                        locator: PackageLocator::installed_store("core", vec!["core".to_string()]),
                    }],
                    exports: vec![BuildExport {
                        alias: "root".to_string(),
                        relative_path: "src".to_string(),
                    }],
                    native_artifacts: Vec::new(),
                },
                entry_point: Some(PackageBuildEntryPoint {
                    kind: PackageBuildEntryPointKind::BuildFunction,
                    name: "build".to_string(),
                }),
            }
        );

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_rejects_pkg_defs_with_parameters() {
        let temp_root = unique_temp_root("pkg_def_params");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, "def core(name: str): pkg = \"core\";\n")
            .expect("Should write the unsupported build dependency fixture");

        let error = parse_package_build(&build_path)
            .expect_err("Phase-one package extraction should reject pkg defs with parameters");

        assert_eq!(error.kind(), PackageErrorKind::InvalidInput);
        assert!(error
            .to_string()
            .contains("package build dependency definitions do not accept parameters"));
        let origin = error
            .origin()
            .expect("Unsupported package dependency defs should keep exact origins");
        assert_eq!(origin.file.as_deref(), build_path.to_str());
        assert_eq!(origin.line, 1);
        assert_eq!(origin.column, 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_rejects_loc_defs_with_options() {
        let temp_root = unique_temp_root("loc_def_options");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, "def[exp] root: loc = \"src\";\n")
            .expect("Should write the unsupported build export fixture");

        let error = parse_package_build(&build_path)
            .expect_err("Phase-one package extraction should reject loc defs with options");

        assert_eq!(error.kind(), PackageErrorKind::InvalidInput);
        assert!(error
            .to_string()
            .contains("package build export definitions do not accept declaration options"));
        let origin = error
            .origin()
            .expect("Unsupported package export defs should keep exact origins");
        assert_eq!(origin.file.as_deref(), build_path.to_str());
        assert_eq!(origin.line, 1);
        assert_eq!(origin.column, 1);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn package_build_parser_detects_canonical_build_entry_definitions() {
        let temp_root = unique_temp_root("build_entry");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(&build_path, "def build(graph: int): int = graph;\n")
            .expect("Should write the build entry fixture");

        let build = parse_package_build(&build_path).expect("Build entry fixture should parse");

        assert_eq!(
            build.entry_point(),
            Some(&PackageBuildEntryPoint {
                kind: PackageBuildEntryPointKind::BuildFunction,
                name: "build".to_string(),
            })
        );
        assert!(!build.has_compatibility_controls());
        assert_eq!(build.mode(), PackageBuildMode::ModernOnly);

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }

    #[test]
    fn fallback_build_parser_recovers_hybrid_build_metadata_from_raw_source() {
        let build = extract_package_build_definition_from_source_fallback(
            concat!(
                "def core: pkg = \"core\";\n",
                "def root: loc = \"src\";\n",
                "def build(graph: int): int = graph;\n",
            ),
        )
        .expect("fallback build extraction should not fail")
        .expect("fallback build extraction should recover hybrid metadata");

        assert_eq!(build.mode(), PackageBuildMode::Hybrid);
        assert_eq!(build.dependencies().len(), 1);
        assert_eq!(build.exports().len(), 1);
        assert!(build.has_entry_point());
    }

    #[test]
    fn package_build_mode_classifies_empty_and_compatibility_builds() {
        let empty = PackageBuildDefinition::default();
        assert_eq!(empty.mode(), PackageBuildMode::Empty);

        let compatibility = PackageBuildDefinition {
            compatibility: PackageBuildCompatibility {
                dependencies: vec![BuildDependency {
                    alias: "core".to_string(),
                    locator: PackageLocator::installed_store("core", vec!["core".to_string()]),
                }],
                exports: Vec::new(),
                native_artifacts: Vec::new(),
            },
            entry_point: None,
        };

        assert_eq!(compatibility.mode(), PackageBuildMode::CompatibilityOnly);
    }

    #[test]
    fn package_build_mode_classifies_hybrid_build_files() {
        let temp_root = unique_temp_root("hybrid_build");
        fs::create_dir_all(&temp_root).expect("Should create temporary build fixture root");
        let build_path = temp_root.join("build.fol");
        fs::write(
            &build_path,
            concat!(
                "def core: pkg = \"core\";\n",
                "def root: loc = \"src\";\n",
                "def build(graph: int): int = graph;\n",
            ),
        )
        .expect("Should write the hybrid build fixture");

        let build = parse_package_build(&build_path).expect("Hybrid build fixture should parse");

        assert_eq!(build.mode(), PackageBuildMode::Hybrid);
        assert_eq!(build.dependencies().len(), 1);
        assert_eq!(build.exports().len(), 1);
        assert!(build.has_entry_point());

        fs::remove_dir_all(&temp_root)
            .expect("Temporary build fixture root should be removable after the test");
    }
}
