use crate::build_graph::BuildGraph;
use crate::{
    BuildApi, DependencyRequest, ExecutableRequest, InstallDirRequest, InstallFileRequest,
    SharedLibraryRequest, StandardOptimizeRequest, StandardTargetRequest, StaticLibraryRequest,
    TestArtifactRequest, UserOptionRequest,
};
use crate::build_api::{CopyFileRequest, WriteFileRequest};
use crate::build_codegen::{CodegenRequest, SystemToolRequest};
use crate::build_option::{
    BuildOptionDeclaration, BuildOptionDeclarationSet, BuildOptimizeMode, BuildTargetTriple,
    ResolvedBuildOptionSet, StandardOptimizeDeclaration, StandardTargetDeclaration,
    UserOptionDeclaration,
};
use fol_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticLocation, ToDiagnostic, ToDiagnosticLocation,
};
use fol_parser::ast::SyntaxOrigin;
use fol_types::Glitch;
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEvaluationBoundary {
    GraphConstructionSubset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedBuildTimeOperation {
    GraphMutation,
    OptionRead,
    PathJoin,
    PathNormalize,
    StringBasic,
    ContainerBasic,
    ControlledFileGeneration,
    ControlledProcessExecution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEvaluationErrorKind {
    InvalidInput,
    Unsupported,
    ValidationFailed,
    Internal,
}

impl BuildEvaluationErrorKind {
    pub fn diagnostic_code(self) -> DiagnosticCode {
        match self {
            Self::InvalidInput => DiagnosticCode::new("K1101"),
            Self::Unsupported => DiagnosticCode::new("K1102"),
            Self::ValidationFailed => DiagnosticCode::new("K1103"),
            Self::Internal => DiagnosticCode::new("K1199"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvaluationError {
    kind: BuildEvaluationErrorKind,
    message: String,
    origin: Option<SyntaxOrigin>,
}

impl BuildEvaluationError {
    pub fn new(kind: BuildEvaluationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            origin: None,
        }
    }

    pub fn with_origin(
        kind: BuildEvaluationErrorKind,
        message: impl Into<String>,
        origin: SyntaxOrigin,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            origin: Some(origin),
        }
    }

    pub fn kind(&self) -> BuildEvaluationErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn origin(&self) -> Option<&SyntaxOrigin> {
        self.origin.as_ref()
    }

    pub fn diagnostic_location(&self) -> Option<DiagnosticLocation> {
        self.origin.as_ref().map(|origin| DiagnosticLocation {
            file: origin.file.clone(),
            line: origin.line,
            column: origin.column,
            length: Some(origin.length),
        })
    }
}

impl std::fmt::Display for BuildEvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BuildEvaluation{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for BuildEvaluationError {}

impl Glitch for BuildEvaluationError {
    fn clone_box(&self) -> Box<dyn Glitch> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ToDiagnosticLocation for BuildEvaluationError {
    fn to_diagnostic_location(&self, file: Option<String>) -> DiagnosticLocation {
        if let Some(origin) = &self.origin {
            DiagnosticLocation {
                file: file.or_else(|| origin.file.clone()),
                line: origin.line,
                column: origin.column,
                length: Some(origin.length),
            }
        } else {
            DiagnosticLocation {
                file,
                line: 1,
                column: 1,
                length: None,
            }
        }
    }
}

impl ToDiagnostic for BuildEvaluationError {
    fn to_diagnostic(&self) -> Diagnostic {
        let mut diagnostic = Diagnostic::error(self.kind.diagnostic_code(), self.to_string());
        if let Some(location) = self.diagnostic_location() {
            diagnostic = diagnostic.with_primary_label(location);
        }
        diagnostic
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildEvaluationRequest {
    pub package_root: String,
    pub inputs: BuildEvaluationInputs,
    pub operations: Vec<BuildEvaluationOperation>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildEvaluationInputs {
    pub working_directory: String,
    pub options: BTreeMap<String, String>,
    pub environment: BTreeMap<String, String>,
}

impl BuildEvaluationInputs {
    pub fn determinism_key(&self) -> String {
        let options = self
            .options
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",");
        let environment = self
            .environment
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "cwd={};options=[{}];env=[{}]",
            self.working_directory, options, environment
        )
    }
}

impl BuildEvaluationRequest {
    pub fn determinism_key(&self) -> String {
        format!(
            "root={};{};ops={}",
            self.package_root,
            self.inputs.determinism_key(),
            self.operations.len()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvaluationOperation {
    pub origin: Option<SyntaxOrigin>,
    pub kind: BuildEvaluationOperationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildEvaluationOperationKind {
    StandardTarget(StandardTargetRequest),
    StandardOptimize(StandardOptimizeRequest),
    Option(UserOptionRequest),
    AddExe(ExecutableRequest),
    AddStaticLib(StaticLibraryRequest),
    AddSharedLib(SharedLibraryRequest),
    AddTest(TestArtifactRequest),
    Step(BuildEvaluationStepRequest),
    AddRun(BuildEvaluationRunRequest),
    InstallArtifact(BuildEvaluationInstallArtifactRequest),
    InstallFile(InstallFileRequest),
    InstallDir(InstallDirRequest),
    WriteFile(WriteFileRequest),
    CopyFile(CopyFileRequest),
    SystemTool(SystemToolRequest),
    Codegen(CodegenRequest),
    Dependency(DependencyRequest),
    Unsupported {
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvaluationStepRequest {
    pub name: String,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvaluationRunRequest {
    pub name: String,
    pub artifact: String,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvaluationInstallArtifactRequest {
    pub name: String,
    pub artifact: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvaluationResult {
    pub boundary: BuildEvaluationBoundary,
    pub allowed_operations: Vec<AllowedBuildTimeOperation>,
    pub package_root: String,
    pub option_declarations: BuildOptionDeclarationSet,
    pub resolved_options: ResolvedBuildOptionSet,
    pub graph: BuildGraph,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedBuildArtifact {
    pub name: String,
    pub root_module: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtractedBuildProgram {
    pub operations: Vec<BuildEvaluationOperation>,
    pub executable_artifacts: Vec<ExtractedBuildArtifact>,
    pub test_artifacts: Vec<ExtractedBuildArtifact>,
    pub run_steps: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedBuildSource {
    pub extracted: ExtractedBuildProgram,
    pub result: BuildEvaluationResult,
}

impl BuildEvaluationResult {
    pub fn new(
        boundary: BuildEvaluationBoundary,
        allowed_operations: Vec<AllowedBuildTimeOperation>,
        package_root: impl Into<String>,
        option_declarations: BuildOptionDeclarationSet,
        resolved_options: ResolvedBuildOptionSet,
        graph: BuildGraph,
    ) -> Self {
        Self {
            boundary,
            allowed_operations,
            package_root: package_root.into(),
            option_declarations,
            resolved_options,
            graph,
        }
    }
}

pub fn evaluate_build_plan(
    request: &BuildEvaluationRequest,
) -> Result<BuildEvaluationResult, BuildEvaluationError> {
    let mut step_names = BTreeMap::new();
    let mut artifact_names = BTreeMap::new();
    let mut option_declarations = BuildOptionDeclarationSet::new();
    let mut resolved_options = ResolvedBuildOptionSet::new();
    let mut graph = BuildGraph::new();
    let mut api = BuildApi::new(&mut graph);

    for (name, value) in &request.inputs.options {
        resolved_options.insert(name.clone(), value.clone());
    }

    for operation in &request.operations {
        match &operation.kind {
            BuildEvaluationOperationKind::StandardTarget(operation_request) => {
                option_declarations.add(BuildOptionDeclaration::StandardTarget(
                    StandardTargetDeclaration {
                        name: operation_request.name.clone(),
                        default: operation_request
                            .default
                            .as_deref()
                            .and_then(BuildTargetTriple::parse),
                    },
                ));
                api.standard_target(operation_request.clone());
            }
            BuildEvaluationOperationKind::StandardOptimize(operation_request) => {
                option_declarations.add(BuildOptionDeclaration::StandardOptimize(
                    StandardOptimizeDeclaration {
                        name: operation_request.name.clone(),
                        default: operation_request
                            .default
                            .as_deref()
                            .and_then(BuildOptimizeMode::parse),
                    },
                ));
                api.standard_optimize(operation_request.clone());
            }
            BuildEvaluationOperationKind::Option(operation_request) => {
                option_declarations.add(BuildOptionDeclaration::User(UserOptionDeclaration {
                    name: operation_request.name.clone(),
                    help: None,
                }));
                api.option(operation_request.clone());
            }
            BuildEvaluationOperationKind::AddExe(operation_request) => {
                let handle = api
                    .add_exe(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
                artifact_names.insert(operation_request.name.clone(), handle);
            }
            BuildEvaluationOperationKind::AddStaticLib(operation_request) => {
                let handle = api
                    .add_static_lib(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
                artifact_names.insert(operation_request.name.clone(), handle);
            }
            BuildEvaluationOperationKind::AddSharedLib(operation_request) => {
                let handle = api
                    .add_shared_lib(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
                artifact_names.insert(operation_request.name.clone(), handle);
            }
            BuildEvaluationOperationKind::AddTest(operation_request) => {
                let handle = api
                    .add_test(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
                artifact_names.insert(operation_request.name.clone(), handle);
            }
            BuildEvaluationOperationKind::Step(operation_request) => {
                let depends_on = operation_request
                    .depends_on
                    .iter()
                    .map(|name| {
                        step_names.get(name).copied().ok_or_else(|| {
                            evaluation_invalid_input(
                                format!("unknown step dependency '{name}'"),
                                operation.origin.clone(),
                            )
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let handle = api
                    .step(crate::StepRequest {
                        name: operation_request.name.clone(),
                        depends_on,
                    })
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
                step_names.insert(operation_request.name.clone(), handle.step_id);
            }
            BuildEvaluationOperationKind::AddRun(operation_request) => {
                let artifact = artifact_names.get(&operation_request.artifact).cloned().ok_or_else(
                    || {
                        evaluation_invalid_input(
                            format!("unknown run artifact '{}'", operation_request.artifact),
                            operation.origin.clone(),
                        )
                    },
                )?;
                let depends_on = operation_request
                    .depends_on
                    .iter()
                    .map(|name| {
                        step_names.get(name).copied().ok_or_else(|| {
                            evaluation_invalid_input(
                                format!("unknown step dependency '{name}'"),
                                operation.origin.clone(),
                            )
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let handle = api
                    .add_run(crate::RunRequest {
                        name: operation_request.name.clone(),
                        artifact,
                        depends_on,
                    })
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
                step_names.insert(operation_request.name.clone(), handle.step_id);
            }
            BuildEvaluationOperationKind::InstallArtifact(operation_request) => {
                let artifact = artifact_names.get(&operation_request.artifact).cloned().ok_or_else(
                    || {
                        evaluation_invalid_input(
                            format!("unknown install artifact '{}'", operation_request.artifact),
                            operation.origin.clone(),
                        )
                    },
                )?;
                api.install(crate::InstallArtifactRequest {
                    name: operation_request.name.clone(),
                    artifact,
                })
                .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
            }
            BuildEvaluationOperationKind::InstallFile(operation_request) => {
                api.install_file(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
            }
            BuildEvaluationOperationKind::InstallDir(operation_request) => {
                api.install_dir(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
            }
            BuildEvaluationOperationKind::WriteFile(operation_request) => {
                api.write_file(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
            }
            BuildEvaluationOperationKind::CopyFile(operation_request) => {
                api.copy_file(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
            }
            BuildEvaluationOperationKind::SystemTool(operation_request) => {
                api.add_system_tool(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
            }
            BuildEvaluationOperationKind::Codegen(operation_request) => {
                api.add_codegen(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
            }
            BuildEvaluationOperationKind::Dependency(operation_request) => {
                api.dependency(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
            }
            _ => {
                return Err(evaluation_error(
                    BuildEvaluationErrorKind::Unsupported,
                    "build evaluator does not support this operation yet",
                    operation.origin.clone(),
                ));
            }
        }
    }

    if let Some(validation_error) = graph.validate().into_iter().next() {
        return Err(evaluation_error(
            BuildEvaluationErrorKind::ValidationFailed,
            validation_error.message,
            None,
        ));
    }

    Ok(BuildEvaluationResult::new(
        BuildEvaluationBoundary::GraphConstructionSubset,
        vec![
            AllowedBuildTimeOperation::GraphMutation,
            AllowedBuildTimeOperation::OptionRead,
        ],
        request.package_root.clone(),
        option_declarations,
        resolved_options,
        graph,
    ))
}

pub fn extract_build_program_from_source(
    build_path: &Path,
    source: &str,
) -> Result<ExtractedBuildProgram, BuildEvaluationError> {
    let Some((param_name, body, body_line)) = extract_build_body(source) else {
        return Ok(ExtractedBuildProgram::default());
    };
    let mut extracted = ExtractedBuildProgram::default();
    let mut executable_artifacts = BTreeMap::new();
    let mut test_artifacts = BTreeMap::new();
    for (offset, raw_line) in body.lines().enumerate() {
        let line_number = body_line + offset;
        let line = raw_line
            .split_once("//")
            .map_or(raw_line, |(prefix, _)| prefix)
            .trim()
            .trim_end_matches(';')
            .trim();
        if line.is_empty() || line == "return ." {
            continue;
        }
        let Some(call) = line.strip_prefix(&format!("{param_name}.")) else {
            continue;
        };
        let Some((method, raw_args)) = call.split_once('(') else {
            continue;
        };
        let args_text = raw_args.trim_end_matches(')').trim();
        let args = parse_build_string_args(args_text);
        let origin = SyntaxOrigin {
            file: Some(build_path.display().to_string()),
            line: line_number,
            column: 1,
            length: raw_line.len(),
        };
        let kind = match (method.trim(), args.as_slice()) {
            ("standard_target", [name]) => {
                BuildEvaluationOperationKind::StandardTarget(StandardTargetRequest::new(name.clone()))
            }
            ("standard_optimize", [name]) => BuildEvaluationOperationKind::StandardOptimize(
                StandardOptimizeRequest::new(name.clone()),
            ),
            ("add_exe", [name, root_module]) => {
                executable_artifacts.insert(
                    name.clone(),
                    ExtractedBuildArtifact {
                        name: name.clone(),
                        root_module: root_module.clone(),
                    },
                );
                BuildEvaluationOperationKind::AddExe(ExecutableRequest {
                    name: name.clone(),
                    root_module: root_module.clone(),
                })
            }
            ("add_static_lib", [name, root_module]) => {
                BuildEvaluationOperationKind::AddStaticLib(StaticLibraryRequest {
                    name: name.clone(),
                    root_module: root_module.clone(),
                })
            }
            ("add_shared_lib", [name, root_module]) => {
                BuildEvaluationOperationKind::AddSharedLib(SharedLibraryRequest {
                    name: name.clone(),
                    root_module: root_module.clone(),
                })
            }
            ("add_test", [name, root_module]) => {
                test_artifacts.insert(
                    name.clone(),
                    ExtractedBuildArtifact {
                        name: name.clone(),
                        root_module: root_module.clone(),
                    },
                );
                BuildEvaluationOperationKind::AddTest(TestArtifactRequest {
                    name: name.clone(),
                    root_module: root_module.clone(),
                })
            }
            ("step", [name, depends_on @ ..]) => {
                BuildEvaluationOperationKind::Step(BuildEvaluationStepRequest {
                    name: name.clone(),
                    depends_on: depends_on.to_vec(),
                })
            }
            ("add_run", [name, artifact, depends_on @ ..]) => {
                extracted.run_steps.insert(name.clone(), artifact.clone());
                BuildEvaluationOperationKind::AddRun(BuildEvaluationRunRequest {
                    name: name.clone(),
                    artifact: artifact.clone(),
                    depends_on: depends_on.to_vec(),
                })
            }
            ("install", [name, artifact]) => {
                BuildEvaluationOperationKind::InstallArtifact(
                    BuildEvaluationInstallArtifactRequest {
                        name: name.clone(),
                        artifact: artifact.clone(),
                    },
                )
            }
            ("dependency", [alias, package]) => {
                BuildEvaluationOperationKind::Dependency(DependencyRequest {
                    alias: alias.clone(),
                    package: package.clone(),
                })
            }
            _ => {
                return Err(BuildEvaluationError::with_origin(
                    BuildEvaluationErrorKind::Unsupported,
                    format!(
                        "unsupported build API call in '{}': {}",
                        build_path.display(),
                        raw_line.trim()
                    ),
                    origin,
                ))
            }
        };
        extracted.operations.push(BuildEvaluationOperation {
            origin: Some(origin),
            kind,
        });
    }
    extracted.executable_artifacts = executable_artifacts.into_values().collect();
    extracted.test_artifacts = test_artifacts.into_values().collect();
    Ok(extracted)
}

pub fn evaluate_build_source(
    request: &BuildEvaluationRequest,
    build_path: &Path,
    source: &str,
) -> Result<Option<EvaluatedBuildSource>, BuildEvaluationError> {
    let extracted = extract_build_program_from_source(build_path, source)?;
    if extracted.operations.is_empty() {
        return Ok(None);
    }
    let result = evaluate_build_plan(&BuildEvaluationRequest {
        package_root: request.package_root.clone(),
        inputs: request.inputs.clone(),
        operations: extracted.operations.clone(),
    })?;
    Ok(Some(EvaluatedBuildSource { extracted, result }))
}

fn extract_build_body(source: &str) -> Option<(String, String, usize)> {
    let start = source.find("def build(")?;
    let rest = &source[start + "def build(".len()..];
    let param_end = rest.find(':')?;
    let param_name = rest[..param_end].trim().to_string();
    if param_name.is_empty() {
        return None;
    }
    let after_equals = rest.find('=')?;
    let body_start = start + "def build(".len() + after_equals + 1;
    let body_source = source[body_start..].trim_start();
    let body_line = source[..body_start].chars().filter(|ch| *ch == '\n').count() + 1;
    if let Some(stripped) = body_source.strip_prefix('{') {
        let block_end = stripped.rfind('}')?;
        return Some((param_name, stripped[..block_end].to_string(), body_line + 1));
    }
    Some((param_name, body_source.trim_end_matches(';').to_string(), body_line))
}

fn parse_build_string_args(args: &str) -> Vec<String> {
    args.split(',')
        .map(str::trim)
        .filter_map(|arg| arg.strip_prefix('"').and_then(|arg| arg.strip_suffix('"')))
        .map(str::to_string)
        .collect()
}

fn evaluation_invalid_input(
    message: impl Into<String>,
    origin: Option<SyntaxOrigin>,
) -> BuildEvaluationError {
    evaluation_error(BuildEvaluationErrorKind::InvalidInput, message, origin)
}

fn evaluation_api_error(
    error: crate::BuildApiError,
    origin: Option<SyntaxOrigin>,
) -> BuildEvaluationError {
    evaluation_invalid_input(error.to_string(), origin)
}

fn evaluation_error(
    kind: BuildEvaluationErrorKind,
    message: impl Into<String>,
    origin: Option<SyntaxOrigin>,
) -> BuildEvaluationError {
    match origin {
        Some(origin) => BuildEvaluationError::with_origin(kind, message, origin),
        None => BuildEvaluationError::new(kind, message),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_build_plan, evaluate_build_source, AllowedBuildTimeOperation, BuildEvaluationBoundary,
        BuildEvaluationError, BuildEvaluationErrorKind, BuildEvaluationInputs,
        BuildEvaluationInstallArtifactRequest, BuildEvaluationOperation,
        BuildEvaluationOperationKind, BuildEvaluationRequest, BuildEvaluationResult,
        BuildEvaluationRunRequest, BuildEvaluationStepRequest,
    };
    use crate::build_option::{BuildOptimizeMode, BuildOptionDeclaration, BuildTargetTriple};
    use crate::build_graph::BuildGraph;
    use crate::{
        CodegenKind, CodegenRequest, DependencyRequest, ExecutableRequest, InstallDirRequest,
        StandardOptimizeRequest, StandardTargetRequest, SystemToolRequest, UserOptionRequest,
    };
    use crate::build_api::{CopyFileRequest, WriteFileRequest};
    use fol_diagnostics::{DiagnosticCode, ToDiagnostic};
    use fol_parser::ast::SyntaxOrigin;
    use std::{collections::BTreeMap, path::Path};

    #[test]
    fn build_evaluation_request_defaults_to_an_empty_package_root() {
        let request = BuildEvaluationRequest::default();

        assert!(request.package_root.is_empty());
        assert!(request.inputs.working_directory.is_empty());
        assert!(request.operations.is_empty());
    }

    #[test]
    fn build_evaluation_result_carries_the_constructed_graph() {
        let graph = BuildGraph::new();
        let result = BuildEvaluationResult::new(
            BuildEvaluationBoundary::GraphConstructionSubset,
            vec![AllowedBuildTimeOperation::GraphMutation],
            "app",
            crate::BuildOptionDeclarationSet::new(),
            crate::ResolvedBuildOptionSet::new(),
            graph.clone(),
        );

        assert_eq!(result.package_root, "app");
        assert_eq!(result.graph, graph);
    }

    #[test]
    fn build_evaluation_result_keeps_boundary_and_allowed_operation_metadata() {
        let result = BuildEvaluationResult::new(
            BuildEvaluationBoundary::GraphConstructionSubset,
            vec![
                AllowedBuildTimeOperation::GraphMutation,
                AllowedBuildTimeOperation::OptionRead,
            ],
            "pkg",
            crate::BuildOptionDeclarationSet::new(),
            crate::ResolvedBuildOptionSet::new(),
            BuildGraph::new(),
        );

        assert_eq!(
            result.boundary,
            BuildEvaluationBoundary::GraphConstructionSubset
        );
        assert_eq!(
            result.allowed_operations,
            vec![
                AllowedBuildTimeOperation::GraphMutation,
                AllowedBuildTimeOperation::OptionRead,
            ]
        );
    }

    #[test]
    fn build_evaluation_result_keeps_declared_and_resolved_options() {
        let mut declarations = crate::BuildOptionDeclarationSet::new();
        declarations.add(BuildOptionDeclaration::StandardOptimize(
            crate::StandardOptimizeDeclaration {
                name: "optimize".to_string(),
                default: Some(BuildOptimizeMode::Debug),
            },
        ));
        let mut resolved = crate::ResolvedBuildOptionSet::new();
        resolved.insert("optimize", "release-fast");
        let result = BuildEvaluationResult::new(
            BuildEvaluationBoundary::GraphConstructionSubset,
            vec![AllowedBuildTimeOperation::OptionRead],
            "pkg",
            declarations,
            resolved,
            BuildGraph::new(),
        );

        assert_eq!(result.option_declarations.declarations().len(), 1);
        assert_eq!(result.resolved_options.get("optimize"), Some("release-fast"));
    }

    #[test]
    fn build_evaluation_error_exposes_origin_as_a_diagnostic_location() {
        let error = BuildEvaluationError::with_origin(
            BuildEvaluationErrorKind::Unsupported,
            "random clock reads are outside the build evaluator subset",
            SyntaxOrigin {
                file: Some("app/build.fol".to_string()),
                line: 4,
                column: 2,
                length: 5,
            },
        );

        let location = error
            .diagnostic_location()
            .expect("evaluation errors with origins should expose locations");

        assert_eq!(location.file.as_deref(), Some("app/build.fol"));
        assert_eq!(location.line, 4);
        assert_eq!(location.column, 2);
        assert_eq!(location.length, Some(5));
    }

    #[test]
    fn build_evaluation_errors_lower_to_stable_diagnostics() {
        let diagnostic = BuildEvaluationError::new(
            BuildEvaluationErrorKind::ValidationFailed,
            "graph validation failed",
        )
        .to_diagnostic();

        assert_eq!(diagnostic.code, DiagnosticCode::new("K1103"));
        assert!(diagnostic.message.contains("graph validation failed"));
    }

    #[test]
    fn build_evaluation_input_determinism_key_is_stable_for_sorted_inputs() {
        let mut options = BTreeMap::new();
        options.insert("optimize".to_string(), "debug".to_string());
        options.insert("target".to_string(), "native".to_string());
        let mut environment = BTreeMap::new();
        environment.insert("CC".to_string(), "clang".to_string());
        environment.insert("AR".to_string(), "llvm-ar".to_string());
        let inputs = BuildEvaluationInputs {
            working_directory: "/work/app".to_string(),
            options,
            environment,
        };

        assert_eq!(
            inputs.determinism_key(),
            "cwd=/work/app;options=[optimize=debug,target=native];env=[AR=llvm-ar,CC=clang]"
        );
    }

    #[test]
    fn build_evaluation_request_determinism_key_includes_root_and_inputs() {
        let mut request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs::default(),
            operations: Vec::new(),
        };
        request
            .inputs
            .options
            .insert("target".to_string(), "native".to_string());

        assert_eq!(
            request.determinism_key(),
            "root=/pkg;cwd=;options=[target=native];env=[];ops=0"
        );
    }

    #[test]
    fn build_evaluation_request_determinism_key_counts_operations() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs::default(),
            operations: vec![BuildEvaluationOperation {
                origin: None,
                kind: BuildEvaluationOperationKind::StandardTarget(StandardTargetRequest::new(
                    "target",
                )),
            }],
        };

        assert_eq!(request.determinism_key(), "root=/pkg;cwd=;options=[];env=[];ops=1");
    }

    #[test]
    fn build_evaluation_operations_keep_origin_and_payload_shape() {
        let operation = BuildEvaluationOperation {
            origin: Some(SyntaxOrigin {
                file: Some("build.fol".to_string()),
                line: 2,
                column: 1,
                length: 3,
            }),
                kind: BuildEvaluationOperationKind::AddExe(ExecutableRequest {
                    name: "app".to_string(),
                    root_module: "src/app.fol".to_string(),
                }),
        };

        assert_eq!(operation.origin.as_ref().map(|origin| origin.line), Some(2));
        match operation.kind {
            BuildEvaluationOperationKind::AddExe(request) => {
                assert_eq!(request.name, "app");
                assert_eq!(request.root_module, "src/app.fol");
            }
            other => panic!("unexpected operation kind: {other:?}"),
        }
    }

    #[test]
    fn build_evaluator_replays_standard_and_user_option_operations() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs::default(),
            operations: vec![
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::StandardTarget(
                        StandardTargetRequest::new("target"),
                    ),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::StandardOptimize(
                        StandardOptimizeRequest::new("optimize"),
                    ),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::Option(UserOptionRequest::bool(
                        "strip", false,
                    )),
                },
            ],
        };

        let result = evaluate_build_plan(&request).expect("option replay should succeed");

        assert_eq!(result.graph.options().len(), 3);
        assert_eq!(result.package_root, "/pkg");
    }

    #[test]
    fn build_evaluator_replays_graph_building_operations_into_a_validated_graph() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs::default(),
            operations: vec![
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::AddExe(ExecutableRequest {
                        name: "app".to_string(),
                        root_module: "src/app.fol".to_string(),
                    }),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::Step(BuildEvaluationStepRequest {
                        name: "build".to_string(),
                        depends_on: Vec::new(),
                    }),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::AddRun(BuildEvaluationRunRequest {
                        name: "run".to_string(),
                        artifact: "app".to_string(),
                        depends_on: vec!["build".to_string()],
                    }),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::InstallArtifact(
                        BuildEvaluationInstallArtifactRequest {
                            name: "install-app".to_string(),
                            artifact: "app".to_string(),
                        },
                    ),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::InstallDir(InstallDirRequest {
                        name: "install-assets".to_string(),
                        path: "share/assets".to_string(),
                    }),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::Dependency(DependencyRequest {
                        alias: "logtiny".to_string(),
                        package: "org/logtiny".to_string(),
                    }),
                },
            ],
        };

        let result = evaluate_build_plan(&request).expect("graph replay should succeed");

        assert_eq!(result.graph.artifacts().len(), 1);
        assert_eq!(result.graph.steps().len(), 2);
        assert_eq!(result.graph.installs().len(), 2);
        assert_eq!(result.graph.modules().len(), 2);
    }

    #[test]
    fn build_evaluator_rejects_unsupported_operations_with_source_origins() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs::default(),
            operations: vec![BuildEvaluationOperation {
                origin: Some(SyntaxOrigin {
                    file: Some("build.fol".to_string()),
                    line: 8,
                    column: 1,
                    length: 4,
                }),
                kind: BuildEvaluationOperationKind::Unsupported {
                    label: "clock_now".to_string(),
                },
            }],
        };

        let error = evaluate_build_plan(&request).expect_err("unsupported operations should fail");

        assert_eq!(error.kind(), BuildEvaluationErrorKind::Unsupported);
        assert_eq!(error.origin().and_then(|origin| origin.file.as_deref()), Some("build.fol"));
        assert_eq!(error.origin().map(|origin| origin.line), Some(8));
    }

    #[test]
    fn build_evaluator_surfaces_graph_validation_failures_as_evaluation_errors() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs::default(),
            operations: vec![BuildEvaluationOperation {
                origin: None,
                kind: BuildEvaluationOperationKind::InstallDir(InstallDirRequest {
                    name: "install-assets".to_string(),
                    path: String::new(),
                }),
            }],
        };

        let error = evaluate_build_plan(&request).expect_err("invalid install dirs should fail");

        assert_eq!(error.kind(), BuildEvaluationErrorKind::ValidationFailed);
        assert!(error.message().contains("directory target must not be empty"));
    }

    #[test]
    fn build_evaluator_replays_option_declarations_and_input_overrides() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: "/tmp/pkg".to_string(),
                options: BTreeMap::from([
                    ("target".to_string(), "aarch64-macos-gnu".to_string()),
                    ("optimize".to_string(), "release-fast".to_string()),
                ]),
                environment: BTreeMap::new(),
            },
            operations: vec![
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::StandardTarget(
                        StandardTargetRequest::new("target")
                            .with_default("x86_64-linux-gnu"),
                    ),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::StandardOptimize(
                        StandardOptimizeRequest::new("optimize").with_default("debug"),
                    ),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::Option(UserOptionRequest::bool(
                        "strip",
                        false,
                    )),
                },
            ],
        };

        let result = evaluate_build_plan(&request).expect("option replay should succeed");

        assert_eq!(result.option_declarations.declarations().len(), 3);
        assert!(matches!(
            &result.option_declarations.declarations()[0],
            BuildOptionDeclaration::StandardTarget(declaration)
            if declaration.default == BuildTargetTriple::parse("x86_64-linux-gnu")
        ));
        assert_eq!(result.resolved_options.get("target"), Some("aarch64-macos-gnu"));
        assert_eq!(result.resolved_options.get("optimize"), Some("release-fast"));
    }

    #[test]
    fn build_evaluator_replays_generated_file_and_codegen_operations() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs::default(),
            operations: vec![
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::WriteFile(WriteFileRequest {
                        name: "version".to_string(),
                        path: "gen/version.fol".to_string(),
                        contents: "generated".to_string(),
                    }),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::CopyFile(CopyFileRequest {
                        name: "config".to_string(),
                        source_path: "assets/config.json".to_string(),
                        destination_path: "gen/config.json".to_string(),
                    }),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::SystemTool(SystemToolRequest {
                        tool: "schema-gen".to_string(),
                        args: vec!["api.yaml".to_string()],
                        outputs: vec!["gen/api.fol".to_string()],
                    }),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::Codegen(CodegenRequest {
                        kind: CodegenKind::Schema,
                        input: "api.yaml".to_string(),
                        output: "gen/api_bindings.fol".to_string(),
                    }),
                },
            ],
        };

        let result = evaluate_build_plan(&request).expect("generated-file replay should succeed");

        assert_eq!(result.graph.generated_files().len(), 4);
    }

    #[test]
    fn build_source_evaluator_extracts_and_replays_restricted_build_bodies() {
        let source = concat!(
            "def build(graph: int): int = {\n",
            "    graph.add_exe(\"app\", \"src/app.fol\");\n",
            "    graph.add_test(\"app_test\", \"test/app.fol\");\n",
            "    graph.add_run(\"serve\", \"app\");\n",
            "    return .\n",
            "}\n",
        );
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: "/pkg".to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, Path::new("/pkg/build.fol"), source)
            .expect("restricted build body should evaluate")
            .expect("build body should produce operations");

        assert_eq!(evaluated.extracted.operations.len(), 3);
        assert_eq!(evaluated.extracted.executable_artifacts.len(), 1);
        assert_eq!(evaluated.extracted.executable_artifacts[0].root_module, "src/app.fol");
        assert_eq!(evaluated.extracted.test_artifacts.len(), 1);
        assert_eq!(evaluated.extracted.run_steps.get("serve"), Some(&"app".to_string()));
        assert_eq!(evaluated.result.graph.artifacts().len(), 2);
        assert_eq!(evaluated.result.graph.steps().len(), 1);
    }
}
