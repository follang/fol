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
pub enum ForbiddenBuildTimeOperation {
    ArbitraryFilesystemRead,
    ArbitraryFilesystemWrite,
    ArbitraryNetworkAccess,
    WallClockAccess,
    AmbientEnvironmentAccess,
    UncontrolledProcessExecution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRuntimeCapabilityModel {
    pub allowed_operations: Vec<AllowedBuildTimeOperation>,
    pub forbidden_operations: Vec<ForbiddenBuildTimeOperation>,
}

impl BuildRuntimeCapabilityModel {
    pub fn new(
        allowed_operations: Vec<AllowedBuildTimeOperation>,
        forbidden_operations: Vec<ForbiddenBuildTimeOperation>,
    ) -> Self {
        Self {
            allowed_operations,
            forbidden_operations,
        }
    }
}

pub fn canonical_graph_construction_capabilities() -> BuildRuntimeCapabilityModel {
    BuildRuntimeCapabilityModel::new(
        vec![
            AllowedBuildTimeOperation::GraphMutation,
            AllowedBuildTimeOperation::OptionRead,
            AllowedBuildTimeOperation::PathJoin,
            AllowedBuildTimeOperation::PathNormalize,
            AllowedBuildTimeOperation::StringBasic,
            AllowedBuildTimeOperation::ContainerBasic,
            AllowedBuildTimeOperation::ControlledFileGeneration,
            AllowedBuildTimeOperation::ControlledProcessExecution,
        ],
        vec![
            ForbiddenBuildTimeOperation::ArbitraryFilesystemRead,
            ForbiddenBuildTimeOperation::ArbitraryFilesystemWrite,
            ForbiddenBuildTimeOperation::ArbitraryNetworkAccess,
            ForbiddenBuildTimeOperation::WallClockAccess,
            ForbiddenBuildTimeOperation::AmbientEnvironmentAccess,
            ForbiddenBuildTimeOperation::UncontrolledProcessExecution,
        ],
    )
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

pub fn forbidden_capability_message(operation: ForbiddenBuildTimeOperation) -> &'static str {
    match operation {
        ForbiddenBuildTimeOperation::ArbitraryFilesystemRead => {
            "build evaluation forbids arbitrary filesystem reads"
        }
        ForbiddenBuildTimeOperation::ArbitraryFilesystemWrite => {
            "build evaluation forbids arbitrary filesystem writes"
        }
        ForbiddenBuildTimeOperation::ArbitraryNetworkAccess => {
            "build evaluation forbids arbitrary network access"
        }
        ForbiddenBuildTimeOperation::WallClockAccess => {
            "build evaluation forbids wall-clock access"
        }
        ForbiddenBuildTimeOperation::AmbientEnvironmentAccess => {
            "build evaluation forbids ambient environment access outside declared inputs"
        }
        ForbiddenBuildTimeOperation::UncontrolledProcessExecution => {
            "build evaluation forbids uncontrolled process execution"
        }
    }
}

pub fn forbidden_capability_error(
    operation: ForbiddenBuildTimeOperation,
    origin: Option<SyntaxOrigin>,
) -> BuildEvaluationError {
    evaluation_error(
        BuildEvaluationErrorKind::Unsupported,
        forbidden_capability_message(operation),
        origin,
    )
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
    pub target: Option<BuildTargetTriple>,
    pub optimize: Option<BuildOptimizeMode>,
    pub options: BTreeMap<String, String>,
    pub environment_policy: BuildEnvironmentSelectionPolicy,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildEvaluationInputEnvelope {
    pub working_directory: String,
    pub target: Option<BuildTargetTriple>,
    pub optimize: Option<BuildOptimizeMode>,
    pub options: BTreeMap<String, String>,
    pub declared_environment: Vec<String>,
    pub selected_environment: BTreeMap<String, String>,
}

impl BuildEvaluationInputEnvelope {
    pub fn determinism_key(&self) -> String {
        let options = self
            .options
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",");
        let declared_environment = self.declared_environment.join(",");
        let target = self
            .target
            .as_ref()
            .map(BuildTargetTriple::render)
            .unwrap_or_default();
        let optimize = self
            .optimize
            .as_ref()
            .map(|mode| mode.as_str().to_string())
            .unwrap_or_default();
        let selected_environment = self
            .selected_environment
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "cwd={};target={};optimize={};options=[{}];declared_env=[{}];env=[{}]",
            self.working_directory,
            target,
            optimize,
            options,
            declared_environment,
            selected_environment
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildEnvironmentSelectionPolicy {
    declared_names: Vec<String>,
}

impl BuildEnvironmentSelectionPolicy {
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut declared_names = names.into_iter().map(Into::into).collect::<Vec<_>>();
        declared_names.sort();
        declared_names.dedup();
        Self { declared_names }
    }

    pub fn declared_names(&self) -> &[String] {
        &self.declared_names
    }

    pub fn allows(&self, name: &str) -> bool {
        self.declared_names.iter().any(|declared| declared == name)
    }

    pub fn select<'a>(
        &self,
        environment: impl IntoIterator<Item = (&'a String, &'a String)>,
    ) -> BTreeMap<String, String> {
        environment
            .into_iter()
            .filter(|(name, _)| self.allows(name))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect()
    }

    pub fn determinism_key(&self) -> String {
        self.declared_names.join(",")
    }
}

impl BuildEvaluationInputs {
    pub fn explicit_envelope(&self) -> BuildEvaluationInputEnvelope {
        BuildEvaluationInputEnvelope {
            working_directory: self.working_directory.clone(),
            target: self.target.clone(),
            optimize: self.optimize,
            options: self.options.clone(),
            declared_environment: self.environment_policy.declared_names().to_vec(),
            selected_environment: self.environment_policy.select(self.environment.iter()),
        }
    }

    pub fn determinism_key(&self) -> String {
        self.explicit_envelope().determinism_key()
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
    pub capabilities: BuildRuntimeCapabilityModel,
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
        capabilities: BuildRuntimeCapabilityModel,
        package_root: impl Into<String>,
        option_declarations: BuildOptionDeclarationSet,
        resolved_options: ResolvedBuildOptionSet,
        graph: BuildGraph,
    ) -> Self {
        Self {
            boundary,
            capabilities,
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
    if resolved_options.get("target").is_none() {
        if let Some(target) = &request.inputs.target {
            resolved_options.insert("target", target.render());
        }
    }
    if resolved_options.get("optimize").is_none() {
        if let Some(optimize) = request.inputs.optimize {
            resolved_options.insert("optimize", optimize.as_str());
        }
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
        canonical_graph_construction_capabilities(),
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
    let mut scope = BuildExtractionScope::default();
    for statement in split_build_statements(&body, body_line) {
        let line = statement.text.trim();
        if line.is_empty() || line == "return ." {
            continue;
        }
        parse_build_statement(
            &mut extracted,
            &mut scope,
            build_path,
            &param_name,
            line,
            statement.line,
            statement.length,
        )?;
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BuildStatement {
    line: usize,
    length: usize,
    text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BuildExtractionScope {
    values: BTreeMap<String, BuildExtractionValue>,
    next_run_index: usize,
    next_install_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BuildExtractionValue {
    OptionName(String),
    Artifact(ExtractedBuildArtifact),
    StepName(String),
}

fn split_build_statements(body: &str, body_line: usize) -> Vec<BuildStatement> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut current_line = body_line;
    let mut line = body_line;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut in_string = false;
    let mut chars = body.chars().peekable();

    while let Some(ch) = chars.next() {
        if current.is_empty() && !ch.is_whitespace() {
            current_line = line;
        }
        if ch == '\n' {
            line += 1;
        }
        if !in_string && ch == '/' && chars.peek() == Some(&'/') {
            for comment_ch in chars.by_ref() {
                if comment_ch == '\n' {
                    line += 1;
                    break;
                }
            }
            current.push(' ');
            continue;
        }
        current.push(ch);
        match ch {
            '"' => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string && paren_depth > 0 => paren_depth -= 1,
            '{' if !in_string => brace_depth += 1,
            '}' if !in_string && brace_depth > 0 => brace_depth -= 1,
            ';' if !in_string && paren_depth == 0 && brace_depth == 0 => {
                let text = current.trim().trim_end_matches(';').trim().to_string();
                if !text.is_empty() {
                    statements.push(BuildStatement {
                        line: current_line,
                        length: text.len(),
                        text,
                    });
                }
                current.clear();
                current_line = line;
            }
            _ => {}
        }
    }

    let tail = current.trim().trim_end_matches(';').trim();
    if !tail.is_empty() {
        statements.push(BuildStatement {
            line: current_line,
            length: tail.len(),
            text: tail.to_string(),
        });
    }

    statements
}

fn parse_build_statement(
    extracted: &mut ExtractedBuildProgram,
    scope: &mut BuildExtractionScope,
    build_path: &Path,
    graph_name: &str,
    statement: &str,
    line: usize,
    length: usize,
) -> Result<(), BuildEvaluationError> {
    if let Some(rest) = statement.strip_prefix("var ") {
        let Some((name, expr)) = rest.split_once('=') else {
            return Err(build_source_unsupported(build_path, statement, line, length));
        };
        let value = parse_build_expression(
            extracted,
            scope,
            build_path,
            graph_name,
            expr.trim(),
            line,
            length,
        )?;
        if let Some(value) = value {
            scope.values.insert(name.trim().to_string(), value);
        }
        return Ok(());
    }
    parse_build_expression(extracted, scope, build_path, graph_name, statement, line, length)?;
    Ok(())
}

fn parse_build_expression(
    extracted: &mut ExtractedBuildProgram,
    scope: &mut BuildExtractionScope,
    build_path: &Path,
    graph_name: &str,
    expr: &str,
    line: usize,
    length: usize,
) -> Result<Option<BuildExtractionValue>, BuildEvaluationError> {
    if expr.trim() == graph_name {
        return Ok(None);
    }
    let origin = SyntaxOrigin {
        file: Some(build_path.display().to_string()),
        line,
        column: 1,
        length,
    };
    let Some(call) = expr.strip_prefix(&format!("{graph_name}.")) else {
        return Ok(None);
    };
    let Some((method, raw_args)) = call.split_once('(') else {
        return Err(build_source_unsupported(build_path, expr, line, length));
    };
    let args_text = raw_args.trim_end_matches(')').trim();
    match method.trim() {
        "standard_target" => {
            let name = parse_optional_single_string_arg(args_text).unwrap_or_else(|| "target".to_string());
            extracted.operations.push(BuildEvaluationOperation {
                origin: Some(origin),
                kind: BuildEvaluationOperationKind::StandardTarget(StandardTargetRequest::new(name.clone())),
            });
            Ok(Some(BuildExtractionValue::OptionName(name)))
        }
        "standard_optimize" => {
            let name =
                parse_optional_single_string_arg(args_text).unwrap_or_else(|| "optimize".to_string());
            extracted.operations.push(BuildEvaluationOperation {
                origin: Some(origin),
                kind: BuildEvaluationOperationKind::StandardOptimize(
                    StandardOptimizeRequest::new(name.clone()),
                ),
            });
            Ok(Some(BuildExtractionValue::OptionName(name)))
        }
        "add_exe" => parse_named_artifact_call(
            extracted,
            scope,
            origin,
            method.trim(),
            args_text,
            BuildNamedArtifactKind::Executable,
        ),
        "add_static_lib" => parse_named_artifact_call(
            extracted,
            scope,
            origin,
            method.trim(),
            args_text,
            BuildNamedArtifactKind::StaticLibrary,
        ),
        "add_shared_lib" => parse_named_artifact_call(
            extracted,
            scope,
            origin,
            method.trim(),
            args_text,
            BuildNamedArtifactKind::SharedLibrary,
        ),
        "add_test" => parse_named_artifact_call(
            extracted,
            scope,
            origin,
            method.trim(),
            args_text,
            BuildNamedArtifactKind::Test,
        ),
        "step" => {
            let args = parse_top_level_args(args_text);
            let Some(name) = resolve_build_string_arg(&args[0], scope) else {
                return Err(build_source_unsupported(build_path, expr, line, length));
            };
            let depends_on = args
                .iter()
                .skip(1)
                .filter_map(|arg| resolve_step_reference(arg, scope))
                .collect::<Vec<_>>();
            extracted.operations.push(BuildEvaluationOperation {
                origin: Some(origin),
                kind: BuildEvaluationOperationKind::Step(BuildEvaluationStepRequest {
                    name: name.clone(),
                    depends_on,
                }),
            });
            Ok(Some(BuildExtractionValue::StepName(name)))
        }
        "add_run" => {
            let args = parse_top_level_args(args_text);
            let (name, artifact) = match args.as_slice() {
                [artifact] => {
                    let Some(artifact) = resolve_artifact_reference(artifact, scope) else {
                        return Err(build_source_unsupported(build_path, expr, line, length));
                    };
                    let name = if scope.next_run_index == 0 {
                        "run".to_string()
                    } else {
                        format!("run-{}", artifact.name)
                    };
                    scope.next_run_index += 1;
                    (name, artifact.name)
                }
                [name, artifact, depends_on @ ..] => {
                    let Some(name) = resolve_build_string_arg(name, scope) else {
                        return Err(build_source_unsupported(build_path, expr, line, length));
                    };
                    let Some(artifact) = resolve_artifact_reference(artifact, scope) else {
                        return Err(build_source_unsupported(build_path, expr, line, length));
                    };
                    let _ = depends_on;
                    (name, artifact.name)
                }
                _ => return Err(build_source_unsupported(build_path, expr, line, length)),
            };
            extracted.run_steps.insert(name.clone(), artifact.clone());
            extracted.operations.push(BuildEvaluationOperation {
                origin: Some(origin),
                kind: BuildEvaluationOperationKind::AddRun(BuildEvaluationRunRequest {
                    name: name.clone(),
                    artifact,
                    depends_on: Vec::new(),
                }),
            });
            Ok(Some(BuildExtractionValue::StepName(name)))
        }
        "install" => {
            let args = parse_top_level_args(args_text);
            let (name, artifact) = match args.as_slice() {
                [artifact] => {
                    let Some(artifact) = resolve_artifact_reference(artifact, scope) else {
                        return Err(build_source_unsupported(build_path, expr, line, length));
                    };
                    let name = if scope.next_install_index == 0 {
                        "install".to_string()
                    } else {
                        format!("install-{}", artifact.name)
                    };
                    scope.next_install_index += 1;
                    (name, artifact.name)
                }
                [name, artifact] => {
                    let Some(name) = resolve_build_string_arg(name, scope) else {
                        return Err(build_source_unsupported(build_path, expr, line, length));
                    };
                    let Some(artifact) = resolve_artifact_reference(artifact, scope) else {
                        return Err(build_source_unsupported(build_path, expr, line, length));
                    };
                    (name, artifact.name)
                }
                _ => return Err(build_source_unsupported(build_path, expr, line, length)),
            };
            extracted.operations.push(BuildEvaluationOperation {
                origin: Some(origin),
                kind: BuildEvaluationOperationKind::InstallArtifact(
                    BuildEvaluationInstallArtifactRequest { name, artifact },
                ),
            });
            Ok(None)
        }
        "dependency" => {
            let args = parse_top_level_args(args_text);
            let [alias, package] = args.as_slice() else {
                return Err(build_source_unsupported(build_path, expr, line, length));
            };
            let Some(alias) = resolve_build_string_arg(alias, scope) else {
                return Err(build_source_unsupported(build_path, expr, line, length));
            };
            let Some(package) = resolve_build_string_arg(package, scope) else {
                return Err(build_source_unsupported(build_path, expr, line, length));
            };
            extracted.operations.push(BuildEvaluationOperation {
                origin: Some(origin),
                kind: BuildEvaluationOperationKind::Dependency(DependencyRequest { alias, package }),
            });
            Ok(None)
        }
        _ => Err(build_source_unsupported(build_path, expr, line, length)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildNamedArtifactKind {
    Executable,
    StaticLibrary,
    SharedLibrary,
    Test,
}

fn parse_named_artifact_call(
    extracted: &mut ExtractedBuildProgram,
    scope: &BuildExtractionScope,
    origin: SyntaxOrigin,
    method: &str,
    args_text: &str,
    kind: BuildNamedArtifactKind,
) -> Result<Option<BuildExtractionValue>, BuildEvaluationError> {
    let (name, root_module) = if args_text.trim_start().starts_with('{') {
        let fields = parse_build_object_fields(args_text)?;
        let name = fields
            .get("name")
            .cloned()
            .ok_or_else(|| evaluation_invalid_input("build artifact object is missing a 'name' field", Some(origin.clone())))?;
        let root_module = fields
            .get("root")
            .or_else(|| fields.get("root_module"))
            .cloned()
            .ok_or_else(|| evaluation_invalid_input("build artifact object is missing a 'root' field", Some(origin.clone())))?;
        (name, root_module)
    } else {
        let args = parse_top_level_args(args_text);
        let [name, root_module] = args.as_slice() else {
            return Err(evaluation_invalid_input(
                format!("unsupported {method} arguments"),
                Some(origin),
            ));
        };
        let name = resolve_build_string_arg(name, scope)
            .ok_or_else(|| evaluation_invalid_input(format!("unsupported {method} name"), Some(origin.clone())))?;
        let root_module = resolve_build_string_arg(root_module, scope)
            .ok_or_else(|| evaluation_invalid_input(format!("unsupported {method} root"), Some(origin.clone())))?;
        (name, root_module)
    };
    let artifact = ExtractedBuildArtifact {
        name: name.clone(),
        root_module: root_module.clone(),
    };
    match kind {
        BuildNamedArtifactKind::Executable => extracted.executable_artifacts.push(artifact.clone()),
        BuildNamedArtifactKind::Test => extracted.test_artifacts.push(artifact.clone()),
        BuildNamedArtifactKind::StaticLibrary | BuildNamedArtifactKind::SharedLibrary => {}
    }
    let kind = match kind {
        BuildNamedArtifactKind::Executable => {
            BuildEvaluationOperationKind::AddExe(ExecutableRequest { name, root_module })
        }
        BuildNamedArtifactKind::StaticLibrary => BuildEvaluationOperationKind::AddStaticLib(
            StaticLibraryRequest { name, root_module },
        ),
        BuildNamedArtifactKind::SharedLibrary => BuildEvaluationOperationKind::AddSharedLib(
            SharedLibraryRequest { name, root_module },
        ),
        BuildNamedArtifactKind::Test => {
            BuildEvaluationOperationKind::AddTest(TestArtifactRequest { name, root_module })
        }
    };
    extracted.operations.push(BuildEvaluationOperation {
        origin: Some(origin),
        kind,
    });
    Ok(Some(BuildExtractionValue::Artifact(artifact)))
}

fn parse_build_object_fields(
    text: &str,
) -> Result<BTreeMap<String, String>, BuildEvaluationError> {
    let inner = text.trim().trim_start_matches('{').trim_end_matches('}');
    let mut fields = BTreeMap::new();
    for field in parse_top_level_args(inner) {
        let Some((name, value)) = field.split_once('=') else {
            continue;
        };
        let key = name.trim().to_string();
        if let Some(value) = parse_quoted_string(value.trim()) {
            fields.insert(key, value);
        }
    }
    Ok(fields)
}

fn parse_top_level_args(args: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut in_string = false;
    for ch in args.chars() {
        match ch {
            '"' => {
                in_string = !in_string;
                current.push(ch);
            }
            '(' if !in_string => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' if !in_string => {
                paren_depth = paren_depth.saturating_sub(1);
                current.push(ch);
            }
            '{' if !in_string => {
                brace_depth += 1;
                current.push(ch);
            }
            '}' if !in_string => {
                brace_depth = brace_depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if !in_string && paren_depth == 0 && brace_depth == 0 => {
                let part = current.trim();
                if !part.is_empty() {
                    parts.push(part.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let tail = current.trim();
    if !tail.is_empty() {
        parts.push(tail.to_string());
    }
    parts
}

fn parse_optional_single_string_arg(args: &str) -> Option<String> {
    let args = parse_top_level_args(args);
    match args.as_slice() {
        [] => None,
        [arg] => parse_quoted_string(arg),
        _ => None,
    }
}

fn parse_quoted_string(text: &str) -> Option<String> {
    text.strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_string)
}

fn resolve_build_string_arg(
    text: &str,
    _scope: &BuildExtractionScope,
) -> Option<String> {
    parse_quoted_string(text.trim())
}

fn resolve_artifact_reference(
    text: &str,
    scope: &BuildExtractionScope,
) -> Option<ExtractedBuildArtifact> {
    if let Some(string) = parse_quoted_string(text.trim()) {
        return Some(ExtractedBuildArtifact {
            name: string.clone(),
            root_module: String::new(),
        });
    }
    match scope.values.get(text.trim()) {
        Some(BuildExtractionValue::Artifact(artifact)) => Some(artifact.clone()),
        _ => None,
    }
}

fn resolve_step_reference(
    text: &str,
    scope: &BuildExtractionScope,
) -> Option<String> {
    if let Some(string) = parse_quoted_string(text.trim()) {
        return Some(string);
    }
    match scope.values.get(text.trim()) {
        Some(BuildExtractionValue::StepName(name)) => Some(name.clone()),
        _ => None,
    }
}

fn build_source_unsupported(
    build_path: &Path,
    statement: &str,
    line: usize,
    length: usize,
) -> BuildEvaluationError {
    BuildEvaluationError::with_origin(
        BuildEvaluationErrorKind::Unsupported,
        format!(
            "unsupported build API call in '{}': {}",
            build_path.display(),
            statement.trim()
        ),
        SyntaxOrigin {
            file: Some(build_path.display().to_string()),
            line,
            column: 1,
            length,
        },
    )
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
        canonical_graph_construction_capabilities, evaluate_build_plan, evaluate_build_source,
        forbidden_capability_error, forbidden_capability_message, AllowedBuildTimeOperation,
        BuildEvaluationBoundary, BuildEvaluationError, BuildEnvironmentSelectionPolicy,
        BuildEvaluationErrorKind, BuildEvaluationInputEnvelope, BuildEvaluationInputs,
        BuildRuntimeCapabilityModel, ForbiddenBuildTimeOperation,
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
        assert!(request.inputs.target.is_none());
        assert!(request.inputs.optimize.is_none());
        assert!(request.operations.is_empty());
    }

    #[test]
    fn build_evaluation_result_carries_the_constructed_graph() {
        let graph = BuildGraph::new();
        let result = BuildEvaluationResult::new(
            BuildEvaluationBoundary::GraphConstructionSubset,
            BuildRuntimeCapabilityModel::new(
                vec![AllowedBuildTimeOperation::GraphMutation],
                Vec::new(),
            ),
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
            BuildRuntimeCapabilityModel::new(
                vec![
                    AllowedBuildTimeOperation::GraphMutation,
                    AllowedBuildTimeOperation::OptionRead,
                ],
                vec![ForbiddenBuildTimeOperation::ArbitraryNetworkAccess],
            ),
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
            result.capabilities.allowed_operations,
            vec![
                AllowedBuildTimeOperation::GraphMutation,
                AllowedBuildTimeOperation::OptionRead,
            ]
        );
        assert_eq!(
            result.capabilities.forbidden_operations,
            vec![ForbiddenBuildTimeOperation::ArbitraryNetworkAccess]
        );
    }

    #[test]
    fn forbidden_build_time_operations_cover_phase_four_runtime_gaps() {
        let forbidden = vec![
            ForbiddenBuildTimeOperation::ArbitraryFilesystemRead,
            ForbiddenBuildTimeOperation::ArbitraryFilesystemWrite,
            ForbiddenBuildTimeOperation::ArbitraryNetworkAccess,
            ForbiddenBuildTimeOperation::WallClockAccess,
            ForbiddenBuildTimeOperation::AmbientEnvironmentAccess,
            ForbiddenBuildTimeOperation::UncontrolledProcessExecution,
        ];

        assert_eq!(forbidden.len(), 6);
        assert!(forbidden.contains(&ForbiddenBuildTimeOperation::ArbitraryNetworkAccess));
        assert!(forbidden.contains(&ForbiddenBuildTimeOperation::WallClockAccess));
    }

    #[test]
    fn runtime_capability_models_keep_allowed_and_forbidden_sets_together() {
        let model = BuildRuntimeCapabilityModel::new(
            vec![AllowedBuildTimeOperation::GraphMutation],
            vec![ForbiddenBuildTimeOperation::WallClockAccess],
        );

        assert_eq!(
            model.allowed_operations,
            vec![AllowedBuildTimeOperation::GraphMutation]
        );
        assert_eq!(
            model.forbidden_operations,
            vec![ForbiddenBuildTimeOperation::WallClockAccess]
        );
    }

    #[test]
    fn canonical_graph_construction_capabilities_cover_the_declared_runtime_surface() {
        let capabilities = canonical_graph_construction_capabilities();

        assert!(capabilities
            .allowed_operations
            .contains(&AllowedBuildTimeOperation::GraphMutation));
        assert!(capabilities
            .allowed_operations
            .contains(&AllowedBuildTimeOperation::ControlledFileGeneration));
        assert!(capabilities
            .allowed_operations
            .contains(&AllowedBuildTimeOperation::ControlledProcessExecution));
        assert!(capabilities
            .forbidden_operations
            .contains(&ForbiddenBuildTimeOperation::ArbitraryNetworkAccess));
        assert!(capabilities
            .forbidden_operations
            .contains(&ForbiddenBuildTimeOperation::AmbientEnvironmentAccess));
    }

    #[test]
    fn environment_selection_policy_sorts_and_filters_declared_variables() {
        let policy = BuildEnvironmentSelectionPolicy::new(["CC", "AR", "CC"]);
        let selected = policy.select(BTreeMap::from([
            ("CC".to_string(), "clang".to_string()),
            ("AR".to_string(), "llvm-ar".to_string()),
            ("HOME".to_string(), "/tmp/home".to_string()),
        ]).iter());

        assert_eq!(
            policy.declared_names(),
            &vec!["AR".to_string(), "CC".to_string()]
        );
        assert_eq!(
            selected,
            BTreeMap::from([
                ("AR".to_string(), "llvm-ar".to_string()),
                ("CC".to_string(), "clang".to_string()),
            ])
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
            BuildRuntimeCapabilityModel::new(
                vec![AllowedBuildTimeOperation::OptionRead],
                Vec::new(),
            ),
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
            target: BuildTargetTriple::parse("x86_64-linux-gnu"),
            optimize: BuildOptimizeMode::parse("debug"),
            options,
            environment_policy: BuildEnvironmentSelectionPolicy::new(["CC", "AR"]),
            environment,
        };

        assert_eq!(
            inputs.determinism_key(),
            "cwd=/work/app;target=x86_64-linux-gnu;optimize=debug;options=[optimize=debug,target=native];declared_env=[AR,CC];env=[AR=llvm-ar,CC=clang]"
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
        request.inputs.target = BuildTargetTriple::parse("aarch64-macos-gnu");
        request.inputs.optimize = BuildOptimizeMode::parse("release-fast");

        assert_eq!(
            request.determinism_key(),
            "root=/pkg;cwd=;target=aarch64-macos-gnu;optimize=release-fast;options=[target=native];declared_env=[];env=[];ops=0"
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

        assert_eq!(
            request.determinism_key(),
            "root=/pkg;cwd=;target=;optimize=;options=[];declared_env=[];env=[];ops=1"
        );
    }

    #[test]
    fn explicit_input_envelope_filters_ambient_environment_before_keying() {
        let inputs = BuildEvaluationInputs {
            working_directory: "/pkg".to_string(),
            target: BuildTargetTriple::parse("x86_64-linux-gnu"),
            optimize: BuildOptimizeMode::parse("release-safe"),
            options: BTreeMap::from([("strip".to_string(), "true".to_string())]),
            environment_policy: BuildEnvironmentSelectionPolicy::new(["CC"]),
            environment: BTreeMap::from([
                ("CC".to_string(), "clang".to_string()),
                ("HOME".to_string(), "/tmp/home".to_string()),
            ]),
        };

        let envelope = inputs.explicit_envelope();

        assert_eq!(
            envelope,
            BuildEvaluationInputEnvelope {
                working_directory: "/pkg".to_string(),
                target: BuildTargetTriple::parse("x86_64-linux-gnu"),
                optimize: BuildOptimizeMode::parse("release-safe"),
                options: BTreeMap::from([("strip".to_string(), "true".to_string())]),
                declared_environment: vec!["CC".to_string()],
                selected_environment: BTreeMap::from([("CC".to_string(), "clang".to_string())]),
            }
        );
        assert_eq!(
            envelope.determinism_key(),
            "cwd=/pkg;target=x86_64-linux-gnu;optimize=release-safe;options=[strip=true];declared_env=[CC];env=[CC=clang]"
        );
    }

    #[test]
    fn forbidden_capability_messages_are_specific_to_the_runtime_surface() {
        assert!(forbidden_capability_message(
            ForbiddenBuildTimeOperation::ArbitraryFilesystemRead
        )
        .contains("filesystem reads"));
        assert!(forbidden_capability_message(
            ForbiddenBuildTimeOperation::AmbientEnvironmentAccess
        )
        .contains("declared inputs"));
    }

    #[test]
    fn forbidden_capability_errors_lower_to_unsupported_diagnostics() {
        let error = forbidden_capability_error(
            ForbiddenBuildTimeOperation::ArbitraryNetworkAccess,
            Some(SyntaxOrigin {
                file: Some("build.fol".to_string()),
                line: 4,
                column: 2,
                length: 5,
            }),
        );
        let diagnostic = error.to_diagnostic();

        assert_eq!(error.kind(), BuildEvaluationErrorKind::Unsupported);
        assert!(error.message().contains("network access"));
        assert_eq!(diagnostic.code, DiagnosticCode::new("K1102"));
        assert_eq!(diagnostic.primary_labels.len(), 1);
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
                environment_policy: BuildEnvironmentSelectionPolicy::default(),
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
    fn build_evaluator_uses_typed_target_and_optimize_inputs_without_duplicate_option_overrides() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: "/tmp/pkg".to_string(),
                target: BuildTargetTriple::parse("x86_64-linux-gnu"),
                optimize: BuildOptimizeMode::parse("release-safe"),
                ..BuildEvaluationInputs::default()
            },
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
            ],
        };

        let result = evaluate_build_plan(&request).expect("typed target/optimize inputs should seed resolved options");

        assert_eq!(result.resolved_options.get("target"), Some("x86_64-linux-gnu"));
        assert_eq!(result.resolved_options.get("optimize"), Some("release-safe"));
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

    #[test]
    fn build_source_evaluator_supports_object_style_artifacts_and_handle_calls() {
        let source = concat!(
            "def build(graph: int): int = {\n",
            "    var target = graph.standard_target();\n",
            "    var optimize = graph.standard_optimize();\n",
            "    var app = graph.add_exe({\n",
            "        name = \"demo\",\n",
            "        root = \"src/demo.fol\",\n",
            "        target = target,\n",
            "        optimize = optimize,\n",
            "    });\n",
            "    graph.install(app);\n",
            "    graph.add_run(app);\n",
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
            .expect("object style build body should evaluate")
            .expect("build body should produce operations");

        assert_eq!(evaluated.extracted.operations.len(), 5);
        assert_eq!(evaluated.extracted.executable_artifacts.len(), 1);
        assert_eq!(evaluated.extracted.executable_artifacts[0].name, "demo");
        assert_eq!(evaluated.extracted.executable_artifacts[0].root_module, "src/demo.fol");
        assert_eq!(evaluated.extracted.run_steps.get("run"), Some(&"demo".to_string()));
        assert_eq!(evaluated.result.graph.artifacts().len(), 1);
        assert_eq!(evaluated.result.graph.installs().len(), 1);
        assert_eq!(evaluated.result.graph.steps().len(), 1);
    }
}
