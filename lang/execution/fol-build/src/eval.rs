use crate::api::{CopyFileRequest, WriteFileRequest};
use crate::codegen::{CodegenRequest, SystemToolRequest};
use crate::executor::{BuildBodyExecutor, ExecutionOutput};
use crate::graph::BuildGraph;
use crate::option::{
    BuildOptimizeMode, BuildOptionDeclaration, BuildOptionDeclarationSet, BuildTargetTriple,
    ResolvedBuildOptionSet, StandardOptimizeDeclaration, StandardTargetDeclaration,
    UserOptionDeclaration,
};
use crate::runtime::{
    BuildExecutionRepresentation, BuildRuntimeArtifact, BuildRuntimeArtifactKind,
    BuildRuntimeDependency, BuildRuntimeDependencyQuery, BuildRuntimeGeneratedFile,
    BuildRuntimeProgram, BuildRuntimeStepBinding, BuildRuntimeStepBindingKind,
};
use crate::api::{
    BuildApi, DependencyRequest, ExecutableRequest, InstallDirRequest, InstallFileRequest,
    SharedLibraryRequest, StandardOptimizeRequest, StandardTargetRequest, StaticLibraryRequest,
    TestArtifactRequest, UserOptionRequest,
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
    Unsupported { label: String },
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
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvaluationResult {
    pub boundary: BuildEvaluationBoundary,
    pub capabilities: BuildRuntimeCapabilityModel,
    pub package_root: String,
    pub option_declarations: BuildOptionDeclarationSet,
    pub resolved_options: ResolvedBuildOptionSet,
    pub dependency_requests: Vec<DependencyRequest>,
    pub graph: BuildGraph,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedBuildSource {
    pub evaluated: EvaluatedBuildProgram,
    pub result: BuildEvaluationResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluatedBuildProgram {
    pub program: BuildRuntimeProgram,
    pub artifacts: Vec<BuildRuntimeArtifact>,
    pub generated_files: Vec<BuildRuntimeGeneratedFile>,
    pub dependencies: Vec<BuildRuntimeDependency>,
    pub dependency_queries: Vec<BuildRuntimeDependencyQuery>,
    pub step_bindings: Vec<BuildRuntimeStepBinding>,
    pub result: BuildEvaluationResult,
}

impl BuildEvaluationResult {
    pub fn new(
        boundary: BuildEvaluationBoundary,
        capabilities: BuildRuntimeCapabilityModel,
        package_root: impl Into<String>,
        option_declarations: BuildOptionDeclarationSet,
        resolved_options: ResolvedBuildOptionSet,
        dependency_requests: Vec<DependencyRequest>,
        graph: BuildGraph,
    ) -> Self {
        Self {
            boundary,
            capabilities,
            package_root: package_root.into(),
            option_declarations,
            resolved_options,
            dependency_requests,
            graph,
        }
    }
}

pub fn evaluate_build_plan(
    request: &BuildEvaluationRequest,
) -> Result<BuildEvaluationResult, BuildEvaluationError> {
    let mut step_names = BTreeMap::new();
    let mut artifact_names = BTreeMap::new();
    let mut dependency_requests = Vec::new();
    let mut option_declarations = BuildOptionDeclarationSet::new();
    let raw_option_overrides = request.inputs.options.clone();
    let mut resolved_options = ResolvedBuildOptionSet::new();
    let mut graph = BuildGraph::new();
    let mut api = BuildApi::new(&mut graph);

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
                    kind: operation_request.kind,
                    default: operation_request.default.clone(),
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
                let artifact = artifact_names
                    .get(&operation_request.artifact)
                    .cloned()
                    .ok_or_else(|| {
                        evaluation_invalid_input(
                            format!("unknown run artifact '{}'", operation_request.artifact),
                            operation.origin.clone(),
                        )
                    })?;
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
                let artifact = artifact_names
                    .get(&operation_request.artifact)
                    .cloned()
                    .ok_or_else(|| {
                        evaluation_invalid_input(
                            format!("unknown install artifact '{}'", operation_request.artifact),
                            operation.origin.clone(),
                        )
                    })?;
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
                    .install(crate::InstallArtifactRequest {
                        name: operation_request.name.clone(),
                        artifact,
                        depends_on,
                    })
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
                step_names.insert(operation_request.name.clone(), handle.step_id);
            }
            BuildEvaluationOperationKind::InstallFile(operation_request) => {
                let handle = api
                    .install_file(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
                step_names.insert(operation_request.name.clone(), handle.step_id);
            }
            BuildEvaluationOperationKind::InstallDir(operation_request) => {
                let handle = api
                    .install_dir(operation_request.clone())
                    .map_err(|error| evaluation_api_error(error, operation.origin.clone()))?;
                step_names.insert(operation_request.name.clone(), handle.step_id);
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
                dependency_requests.push(operation_request.clone());
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

    if let Some(target) = &request.inputs.target {
        resolved_options.insert("target", target.render());
    }
    if let Some(optimize) = request.inputs.optimize {
        resolved_options.insert("optimize", optimize.as_str());
    }
    for declaration in option_declarations.declarations() {
        if resolved_options.get(declaration.name()).is_none() {
            if let Some(default) = declaration.default_raw_value() {
                resolved_options.insert(declaration.name(), default);
            }
        }
    }
    for (name, raw_value) in &raw_option_overrides {
        let Some(declaration) = option_declarations.find(name) else {
            resolved_options.insert(name.clone(), raw_value.clone());
            continue;
        };
        let Some(coerced) = declaration.coerce_raw_value(raw_value) else {
            return Err(evaluation_invalid_input(
                format!("build option '{name}' cannot coerce value '{raw_value}'"),
                None,
            ));
        };
        resolved_options.insert(name.clone(), coerced);
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
        dependency_requests,
        graph,
    ))
}

pub fn evaluate_build_source(
    request: &BuildEvaluationRequest,
    build_path: &Path,
    _source: &str,
) -> Result<Option<EvaluatedBuildSource>, BuildEvaluationError> {
    let Some((executor, body)) = BuildBodyExecutor::from_file(build_path)? else {
        return Ok(None);
    };
    let exec_output = executor.execute(&body)?;
    if exec_output.operations.is_empty() {
        return Ok(None);
    }
    let result = evaluate_build_plan(&BuildEvaluationRequest {
        package_root: request.package_root.clone(),
        inputs: request.inputs.clone(),
        operations: exec_output.operations.clone(),
    })?;
    let evaluated = build_evaluated_program(&exec_output, &result);
    Ok(Some(EvaluatedBuildSource { evaluated, result }))
}

fn build_evaluated_program(
    exec_output: &ExecutionOutput,
    result: &BuildEvaluationResult,
) -> EvaluatedBuildProgram {
    let mut step_bindings = exec_output
        .run_steps
        .iter()
        .map(|(step_name, artifact_name)| {
            let kind = if step_name == "run" {
                BuildRuntimeStepBindingKind::DefaultRun
            } else {
                BuildRuntimeStepBindingKind::NamedRun
            };
            BuildRuntimeStepBinding::new(step_name.clone(), kind, Some(artifact_name.clone()))
        })
        .collect::<Vec<_>>();
    if exec_output.executable_artifacts.len() == 1
        && !step_bindings
            .iter()
            .any(|binding| binding.step_name == "build")
    {
        step_bindings.push(BuildRuntimeStepBinding::new(
            "build",
            BuildRuntimeStepBindingKind::DefaultBuild,
            Some(exec_output.executable_artifacts[0].name.clone()),
        ));
    }
    if exec_output.test_artifacts.len() == 1
        && !step_bindings
            .iter()
            .any(|binding| binding.step_name == "test")
    {
        step_bindings.push(BuildRuntimeStepBinding::new(
            "test",
            BuildRuntimeStepBindingKind::DefaultTest,
            Some(exec_output.test_artifacts[0].name.clone()),
        ));
    }
    let artifacts = exec_output
        .executable_artifacts
        .iter()
        .map(|artifact| {
            BuildRuntimeArtifact::new(
                artifact.name.clone(),
                BuildRuntimeArtifactKind::Executable,
                artifact.root_module.resolve(&result.resolved_options),
            )
            .with_target_config(
                artifact
                    .target
                    .as_ref()
                    .map(|v| v.resolve(&result.resolved_options)),
                artifact
                    .optimize
                    .as_ref()
                    .map(|v| v.resolve(&result.resolved_options)),
            )
        })
        .chain(exec_output.test_artifacts.iter().map(|artifact| {
            BuildRuntimeArtifact::new(
                artifact.name.clone(),
                BuildRuntimeArtifactKind::Test,
                artifact.root_module.resolve(&result.resolved_options),
            )
            .with_target_config(
                artifact
                    .target
                    .as_ref()
                    .map(|v| v.resolve(&result.resolved_options)),
                artifact
                    .optimize
                    .as_ref()
                    .map(|v| v.resolve(&result.resolved_options)),
            )
        }))
        .collect::<Vec<_>>();
    let dependencies = result
        .dependency_requests
        .iter()
        .map(|request| BuildRuntimeDependency {
            alias: request.alias.clone(),
            package: request.package.clone(),
            evaluation_mode: request.evaluation_mode,
        })
        .collect::<Vec<_>>();
    EvaluatedBuildProgram {
        program: BuildRuntimeProgram::new(BuildExecutionRepresentation::RestrictedRuntimeIr),
        artifacts,
        generated_files: exec_output.generated_files.clone(),
        dependencies,
        dependency_queries: exec_output.dependency_queries.clone(),
        step_bindings,
        result: result.clone(),
    }
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
        BuildEnvironmentSelectionPolicy, BuildEvaluationBoundary, BuildEvaluationError,
        BuildEvaluationErrorKind, BuildEvaluationInputEnvelope, BuildEvaluationInputs,
        BuildEvaluationInstallArtifactRequest, BuildEvaluationOperation,
        BuildEvaluationOperationKind, BuildEvaluationRequest, BuildEvaluationResult,
        BuildEvaluationRunRequest, BuildEvaluationStepRequest, BuildRuntimeCapabilityModel,
        EvaluatedBuildProgram, ForbiddenBuildTimeOperation,
    };
    use crate::api::{
        CopyFileRequest, DependencyRequest, ExecutableRequest, InstallDirRequest,
        StandardOptimizeRequest, StandardTargetRequest, UserOptionRequest, WriteFileRequest,
    };
    use crate::codegen::{CodegenKind, CodegenRequest, SystemToolRequest};
    use crate::graph::BuildGraph;
    use crate::option::{
        BuildOptimizeMode, BuildOptionDeclaration, BuildOptionDeclarationSet, BuildTargetTriple,
        ResolvedBuildOptionSet,
    };
    use crate::runtime::{BuildRuntimeDependencyQueryKind, BuildRuntimeGeneratedFileKind};
    use fol_diagnostics::{DiagnosticCode, ToDiagnostic};
    use fol_parser::ast::SyntaxOrigin;
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    fn temp_build_package(source: &str) -> (PathBuf, PathBuf) {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let package_root = std::env::temp_dir().join(format!(
            "fol_build_eval_{}_{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&package_root).expect("temp package root should be created");
        fs::write(
            package_root.join("package.yaml"),
            "name: build-eval\nversion: 1.0.0\n",
        )
        .expect("package metadata should be written");
        fs::write(package_root.join("build.fol"), source).expect("build source should be written");
        (package_root.clone(), package_root.join("build.fol"))
    }

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
            Vec::new(),
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
            Vec::new(),
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
        let selected = policy.select(
            BTreeMap::from([
                ("CC".to_string(), "clang".to_string()),
                ("AR".to_string(), "llvm-ar".to_string()),
                ("HOME".to_string(), "/tmp/home".to_string()),
            ])
            .iter(),
        );

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
            Vec::new(),
            BuildGraph::new(),
        );

        assert_eq!(result.option_declarations.declarations().len(), 1);
        assert_eq!(
            result.resolved_options.get("optimize"),
            Some("release-fast")
        );
    }

    #[test]
    fn build_evaluation_result_keeps_declared_dependency_requests() {
        let dependencies = vec![DependencyRequest {
            alias: "core".to_string(),
            package: "org/core".to_string(),
            evaluation_mode: Some(crate::DependencyBuildEvaluationMode::Lazy),
            surface: None,
        }];
        let result = BuildEvaluationResult::new(
            BuildEvaluationBoundary::GraphConstructionSubset,
            BuildRuntimeCapabilityModel::new(
                vec![AllowedBuildTimeOperation::GraphMutation],
                Vec::new(),
            ),
            "pkg",
            BuildOptionDeclarationSet::new(),
            ResolvedBuildOptionSet::new(),
            dependencies.clone(),
            BuildGraph::new(),
        );

        assert_eq!(result.dependency_requests, dependencies);
    }

    #[test]
    fn build_plan_seeds_declared_option_defaults_into_resolved_values() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs::default(),
            operations: vec![
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::StandardTarget(
                        StandardTargetRequest::new("target").with_default("x86_64-linux-gnu"),
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
                    kind: BuildEvaluationOperationKind::Option(UserOptionRequest::int("jobs", 8)),
                },
            ],
        };

        let result = evaluate_build_plan(&request).expect("declared defaults should seed");

        assert_eq!(
            result.resolved_options.get("target"),
            Some("x86_64-linux-gnu")
        );
        assert_eq!(result.resolved_options.get("optimize"), Some("debug"));
        assert_eq!(result.resolved_options.get("jobs"), Some("8"));
    }

    #[test]
    fn build_plan_rejects_raw_overrides_that_do_not_match_declared_option_kinds() {
        let mut inputs = BuildEvaluationInputs::default();
        inputs
            .options
            .insert("jobs".to_string(), "fast".to_string());
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs,
            operations: vec![BuildEvaluationOperation {
                origin: None,
                kind: BuildEvaluationOperationKind::Option(UserOptionRequest::int("jobs", 8)),
            }],
        };

        let error = evaluate_build_plan(&request)
            .expect_err("invalid raw overrides should fail against declared kinds");

        assert_eq!(error.kind(), BuildEvaluationErrorKind::InvalidInput);
        assert!(error.message().contains("jobs"));
        assert!(error.message().contains("fast"));
    }

    #[test]
    fn build_source_evaluator_supports_object_style_dependency_configs() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var core = graph.dependency({ alias = \"core\", package = \"org/core\", mode = \"lazy\" });\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("dependency configs should evaluate")
            .expect("build body should produce a graph");

        assert_eq!(evaluated.result.dependency_requests.len(), 1);
        assert_eq!(evaluated.result.dependency_requests[0].alias, "core");
        assert_eq!(evaluated.result.dependency_requests[0].package, "org/core");
        assert_eq!(
            evaluated.result.dependency_requests[0].evaluation_mode,
            Some(crate::DependencyBuildEvaluationMode::Lazy)
        );
        assert_eq!(evaluated.evaluated.dependencies.len(), 1);
        assert_eq!(evaluated.evaluated.dependencies[0].alias, "core");
    }

    #[test]
    fn build_source_evaluator_supports_object_style_write_file_configs() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var version = graph.write_file({ name = \"version\", path = \"gen/version.fol\", contents = \"generated\" });\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("write-file configs should evaluate")
            .expect("build body should produce a graph");

        assert!(matches!(
            evaluated.result.graph.generated_files()[0].kind,
            crate::BuildGeneratedFileKind::Write
        ));
        assert_eq!(
            evaluated.result.graph.generated_files()[0].name,
            "gen/version.fol"
        );
    }

    #[test]
    fn build_source_evaluator_supports_object_style_copy_file_configs() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var asset = graph.copy_file({ name = \"asset\", source = \"assets/logo.svg\", path = \"gen/logo.svg\" });\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("copy-file configs should evaluate")
            .expect("build body should produce a graph");

        assert!(matches!(
            evaluated.result.graph.generated_files()[0].kind,
            crate::BuildGeneratedFileKind::Copy
        ));
        assert_eq!(
            evaluated.result.graph.generated_files()[0].name,
            "gen/logo.svg"
        );
    }

    #[test]
    fn build_source_evaluator_supports_object_style_system_tool_configs() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var bindings = graph.add_system_tool({ tool = \"flatc\", output = \"gen/schema.fol\" });\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("system-tool configs should evaluate")
            .expect("build body should produce a graph");

        assert!(matches!(
            evaluated.result.graph.generated_files()[0].kind,
            crate::BuildGeneratedFileKind::CaptureOutput
        ));
        assert_eq!(
            evaluated.result.graph.generated_files()[0].name,
            "gen/schema.fol"
        );
    }

    #[test]
    fn build_source_evaluator_supports_object_style_codegen_configs() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var schema = graph.add_codegen({ kind = \"schema\", input = \"schema/api.yaml\", output = \"gen/api.fol\" });\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("codegen configs should evaluate")
            .expect("build body should produce a graph");

        assert!(matches!(
            evaluated.result.graph.generated_files()[0].kind,
            crate::BuildGeneratedFileKind::Write
        ));
        assert_eq!(
            evaluated.result.graph.generated_files()[0].name,
            "gen/api.fol"
        );
    }

    #[test]
    fn build_source_evaluator_keeps_generated_outputs_in_evaluated_programs() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var version = graph.write_file({ name = \"version\", path = \"gen/version.fol\", contents = \"generated\" });\n",
            "    var asset = graph.copy_file({ name = \"asset\", source = \"assets/logo.svg\", path = \"gen/logo.svg\" });\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("generated outputs should evaluate")
            .expect("build body should produce a graph");

        assert_eq!(evaluated.evaluated.generated_files.len(), 2);
        assert!(evaluated
            .evaluated
            .generated_files
            .iter()
            .any(|file| file.relative_path == "gen/version.fol"
                && file.kind == BuildRuntimeGeneratedFileKind::Write));
        assert!(evaluated
            .evaluated
            .generated_files
            .iter()
            .any(|file| file.relative_path == "gen/logo.svg"
                && file.kind == BuildRuntimeGeneratedFileKind::Copy));
    }

    #[test]
    fn build_source_evaluator_keeps_mixed_generated_output_families() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var version = graph.write_file({ name = \"version\", path = \"gen/version.fol\", contents = \"generated\" });\n",
            "    var asset = graph.copy_file({ name = \"asset\", source = \"assets/logo.svg\", path = \"gen/logo.svg\" });\n",
            "    var tool = graph.add_system_tool({ tool = \"flatc\", output = \"gen/schema.fol\" });\n",
            "    var codegen = graph.add_codegen({ kind = \"schema\", input = \"schema/api.yaml\", output = \"gen/api.fol\" });\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("mixed generated outputs should evaluate")
            .expect("build body should produce a graph");
        let kinds = evaluated
            .evaluated
            .generated_files
            .iter()
            .map(|file| file.kind)
            .collect::<Vec<_>>();

        assert_eq!(evaluated.evaluated.generated_files.len(), 4);
        assert!(kinds.contains(&BuildRuntimeGeneratedFileKind::Write));
        assert!(kinds.contains(&BuildRuntimeGeneratedFileKind::Copy));
        assert!(kinds.contains(&BuildRuntimeGeneratedFileKind::ToolOutput));
        assert!(kinds.contains(&BuildRuntimeGeneratedFileKind::CodegenOutput));
    }

    #[test]
    fn build_source_evaluator_records_dependency_module_and_artifact_queries() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var core = graph.dependency({ alias = \"core\", package = \"org/core\" });\n",
            "    var module = core.module(\"root\");\n",
            "    var artifact = core.artifact(\"corelib\");\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("dependency queries should evaluate")
            .expect("build body should produce a graph");

        assert_eq!(evaluated.evaluated.dependency_queries.len(), 2);
        assert!(evaluated
            .evaluated
            .dependency_queries
            .iter()
            .any(|query| query.dependency_alias == "core"
                && query.query_name == "root"
                && query.kind == BuildRuntimeDependencyQueryKind::Module));
        assert!(evaluated
            .evaluated
            .dependency_queries
            .iter()
            .any(|query| query.dependency_alias == "core"
                && query.query_name == "corelib"
                && query.kind == BuildRuntimeDependencyQueryKind::Artifact));
    }

    #[test]
    fn build_source_evaluator_records_dependency_step_and_generated_queries() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var core = graph.dependency({ alias = \"core\", package = \"org/core\" });\n",
            "    var step = core.step(\"check\");\n",
            "    var generated = core.generated(\"bindings\");\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("dependency queries should evaluate")
            .expect("build body should produce a graph");

        assert_eq!(evaluated.evaluated.dependency_queries.len(), 2);
        assert!(evaluated
            .evaluated
            .dependency_queries
            .iter()
            .any(|query| query.dependency_alias == "core"
                && query.query_name == "check"
                && query.kind == BuildRuntimeDependencyQueryKind::Step));
        assert!(evaluated
            .evaluated
            .dependency_queries
            .iter()
            .any(|query| query.dependency_alias == "core"
                && query.query_name == "bindings"
                && query.kind == BuildRuntimeDependencyQueryKind::GeneratedOutput));
    }

    #[test]
    fn build_source_evaluator_keeps_full_dependency_surface_usage_together() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var dep = graph.dependency({ alias = \"core\", package = \"org/core\", mode = \"on-demand\" });\n",
            "    var module = dep.module(\"root\");\n",
            "    var artifact = dep.artifact(\"corelib\");\n",
            "    var step = dep.step(\"check\");\n",
            "    var generated = dep.generated(\"bindings\");\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("dependency surface should evaluate")
            .expect("build body should produce a graph");
        let query_kinds = evaluated
            .evaluated
            .dependency_queries
            .iter()
            .map(|query| query.kind)
            .collect::<Vec<_>>();

        assert_eq!(evaluated.evaluated.dependencies.len(), 1);
        assert_eq!(
            evaluated.evaluated.dependencies[0].evaluation_mode,
            Some(crate::DependencyBuildEvaluationMode::OnDemand)
        );
        assert_eq!(evaluated.evaluated.dependency_queries.len(), 4);
        assert!(query_kinds.contains(&BuildRuntimeDependencyQueryKind::Module));
        assert!(query_kinds.contains(&BuildRuntimeDependencyQueryKind::Artifact));
        assert!(query_kinds.contains(&BuildRuntimeDependencyQueryKind::Step));
        assert!(query_kinds.contains(&BuildRuntimeDependencyQueryKind::GeneratedOutput));
    }

    #[test]
    fn build_source_evaluator_resolves_deferred_artifact_option_values_into_runtime_metadata() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var root = graph.option({ name = \"root\", kind = \"path\", default = \"src/demo.fol\" });\n",
            "    var target = graph.standard_target();\n",
            "    var optimize = graph.standard_optimize();\n",
            "    graph.add_exe({ name = \"demo\", root = root, target = target, optimize = optimize });\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                target: BuildTargetTriple::parse("x86_64-linux-gnu"),
                optimize: BuildOptimizeMode::parse("release-fast"),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("deferred artifact configs should evaluate")
            .expect("build body should produce operations");

        let artifact = evaluated
            .evaluated
            .artifacts
            .iter()
            .find(|artifact| artifact.name == "demo")
            .expect("artifact should exist");

        assert_eq!(artifact.root_module, "src/demo.fol");
        assert_eq!(artifact.target.as_deref(), Some("x86_64-linux-gnu"));
        assert_eq!(artifact.optimize.as_deref(), Some("release-fast"));
    }

    #[test]
    fn build_source_evaluator_applies_build_inputs_and_option_overrides_to_artifact_metadata() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var root = graph.option({ name = \"root\", kind = \"path\", default = \"src/default.fol\" });\n",
            "    var target = graph.standard_target();\n",
            "    var optimize = graph.standard_optimize();\n",
            "    var app = graph.add_exe({ name = \"demo\", root = root, target = target, optimize = optimize });\n",
            "    graph.add_run(app);\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let mut inputs = BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            target: BuildTargetTriple::parse("aarch64-macos-gnu"),
            optimize: BuildOptimizeMode::parse("release-small"),
            ..BuildEvaluationInputs::default()
        };
        inputs
            .options
            .insert("root".to_string(), "src/cli-selected.fol".to_string());
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs,
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("build inputs should flow into artifact metadata")
            .expect("build body should produce operations");

        let artifact = evaluated
            .evaluated
            .artifacts
            .iter()
            .find(|artifact| artifact.name == "demo")
            .expect("artifact should exist");

        assert_eq!(artifact.root_module, "src/cli-selected.fol");
        assert_eq!(artifact.target.as_deref(), Some("aarch64-macos-gnu"));
        assert_eq!(artifact.optimize.as_deref(), Some("release-small"));
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
        assert!(
            forbidden_capability_message(ForbiddenBuildTimeOperation::ArbitraryFilesystemRead)
                .contains("filesystem reads")
        );
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
        assert_eq!(diagnostic.labels.len(), 1);
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
                    kind: BuildEvaluationOperationKind::StandardTarget(StandardTargetRequest::new(
                        "target",
                    )),
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
                            depends_on: vec!["build".to_string()],
                        },
                    ),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::InstallDir(InstallDirRequest {
                        name: "install-assets".to_string(),
                        path: "share/assets".to_string(),
                        depends_on: Vec::new(),
                    }),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::Dependency(DependencyRequest {
                        alias: "logtiny".to_string(),
                        package: "org/logtiny".to_string(),
                        evaluation_mode: None,
                        surface: None,
                    }),
                },
            ],
        };

        let result = evaluate_build_plan(&request).expect("graph replay should succeed");

        assert_eq!(result.graph.artifacts().len(), 1);
        assert_eq!(result.graph.steps().len(), 4);
        assert_eq!(result.graph.installs().len(), 2);
        assert_eq!(result.graph.modules().len(), 2);
        let install_app = result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "install-app")
            .expect("install-app step should exist");
        let build = result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "build")
            .expect("build step should exist");
        assert_eq!(
            result
                .graph
                .step_dependencies_for(install_app.id)
                .collect::<Vec<_>>(),
            vec![build.id]
        );
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
        assert_eq!(
            error.origin().and_then(|origin| origin.file.as_deref()),
            Some("build.fol")
        );
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
                    depends_on: Vec::new(),
                }),
            }],
        };

        let error = evaluate_build_plan(&request).expect_err("invalid install dirs should fail");

        assert_eq!(error.kind(), BuildEvaluationErrorKind::ValidationFailed);
        assert!(error
            .message()
            .contains("directory target must not be empty"));
    }

    #[test]
    fn build_evaluator_replays_option_declarations_and_input_overrides() {
        let request = BuildEvaluationRequest {
            package_root: "/pkg".to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: "/tmp/pkg".to_string(),
                target: None,
                optimize: None,
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
                        StandardTargetRequest::new("target").with_default("x86_64-linux-gnu"),
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
                        "strip", false,
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
        assert_eq!(
            result.resolved_options.get("target"),
            Some("aarch64-macos-gnu")
        );
        assert_eq!(
            result.resolved_options.get("optimize"),
            Some("release-fast")
        );
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
                    kind: BuildEvaluationOperationKind::StandardTarget(StandardTargetRequest::new(
                        "target",
                    )),
                },
                BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::StandardOptimize(
                        StandardOptimizeRequest::new("optimize"),
                    ),
                },
            ],
        };

        let result = evaluate_build_plan(&request)
            .expect("typed target/optimize inputs should seed resolved options");

        assert_eq!(
            result.resolved_options.get("target"),
            Some("x86_64-linux-gnu")
        );
        assert_eq!(
            result.resolved_options.get("optimize"),
            Some("release-safe")
        );
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
            "pro[] build(graph: Graph): non = {\n",
            "    graph.add_exe(\"app\", \"src/app.fol\");\n",
            "    graph.add_test(\"app_test\", \"test/app.fol\");\n",
            "    graph.add_run(\"serve\", \"app\");\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("restricted build body should evaluate")
            .expect("build body should produce operations");

        assert_eq!(evaluated.evaluated.artifacts.len(), 2);
        assert!(evaluated
            .evaluated
            .artifacts
            .iter()
            .any(|artifact| artifact.root_module == "src/app.fol"));
        assert!(evaluated
            .evaluated
            .step_bindings
            .iter()
            .any(|binding| binding.step_name == "serve"));
        assert_eq!(evaluated.result.graph.artifacts().len(), 2);
        assert_eq!(evaluated.result.graph.steps().len(), 1);
    }

    #[test]
    fn build_source_evaluator_supports_object_style_artifacts_and_handle_calls() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
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
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("object style build body should evaluate")
            .expect("build body should produce operations");

        assert_eq!(evaluated.evaluated.artifacts.len(), 1);
        assert!(evaluated
            .evaluated
            .artifacts
            .iter()
            .any(|artifact| artifact.name == "demo" && artifact.root_module == "src/demo.fol"));
        assert!(evaluated
            .evaluated
            .step_bindings
            .iter()
            .any(|binding| binding.step_name == "run"));
        assert_eq!(evaluated.result.graph.artifacts().len(), 1);
        assert_eq!(evaluated.result.graph.installs().len(), 1);
        let mut step_names = evaluated
            .result
            .graph
            .steps()
            .iter()
            .map(|step| step.name.as_str())
            .collect::<Vec<_>>();
        step_names.sort_unstable();
        assert_eq!(step_names, vec!["install", "run"]);
    }

    #[test]
    fn build_source_evaluator_supports_user_option_record_configs() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var strip = graph.option({ name = \"strip\", kind = \"bool\", default = false });\n",
            "    var jobs = graph.option({ name = \"jobs\", kind = \"int\", default = 8 });\n",
            "    var flavor = graph.option({ name = \"flavor\", kind = \"enum\", default = \"fast\" });\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("user option configs should evaluate")
            .expect("build body should produce operations");

        assert_eq!(evaluated.result.option_declarations.declarations().len(), 3);
        assert_eq!(
            evaluated.result.resolved_options.get("strip"),
            Some("false")
        );
        assert_eq!(evaluated.result.resolved_options.get("jobs"), Some("8"));
        assert_eq!(
            evaluated.result.resolved_options.get("flavor"),
            Some("fast")
        );
    }

    #[test]
    fn build_source_evaluator_reuses_bound_run_and_install_handles_as_step_dependencies() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var app = graph.add_exe(\"demo\", \"src/demo.fol\");\n",
            "    var run_app = graph.add_run(app);\n",
            "    var install_app = graph.install(app);\n",
            "    graph.step(\"bundle\", run_app, install_app);\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("bound step-like handles should evaluate")
            .expect("build body should produce operations");

        let bundle = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "bundle")
            .expect("bundle step should exist");
        let dependencies = evaluated
            .result
            .graph
            .step_dependencies_for(bundle.id)
            .collect::<Vec<_>>();
        let dependency_names = dependencies
            .iter()
            .filter_map(|id| evaluated.result.graph.steps().get(id.index()))
            .map(|step| step.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(dependency_names, vec!["run", "install"]);
    }

    #[test]
    fn build_source_evaluator_rejects_unknown_handle_methods_explicitly() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var docs = graph.step(\"docs\");\n",
            "    docs.finish(docs);\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let error = evaluate_build_source(&request, &build_path, source)
            .expect_err("unsupported handle methods should fail explicitly");

        assert_eq!(error.kind(), BuildEvaluationErrorKind::Unsupported);
        assert!(error.message().contains("finish"));
    }

    #[test]
    fn build_source_evaluator_supports_step_handle_depend_on_chains() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var lint = graph.step(\"lint\");\n",
            "    graph.step(\"docs\").depend_on(lint);\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("step-handle chaining should evaluate")
            .expect("build body should produce operations");

        let docs = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "docs")
            .expect("docs step should exist");
        let lint = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "lint")
            .expect("lint step should exist");
        assert_eq!(
            evaluated
                .result
                .graph
                .step_dependencies_for(docs.id)
                .collect::<Vec<_>>(),
            vec![lint.id]
        );
    }

    #[test]
    fn build_source_evaluator_supports_run_handle_depend_on_chains() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var lint = graph.step(\"lint\");\n",
            "    var app = graph.add_exe({ name = \"app\", root = \"src/app.fol\" });\n",
            "    graph.add_run(app).depend_on(lint);\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("run-handle chaining should evaluate")
            .expect("build body should produce operations");

        let run = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "run")
            .expect("run step should exist");
        let lint = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "lint")
            .expect("lint step should exist");
        assert_eq!(
            evaluated
                .result
                .graph
                .step_dependencies_for(run.id)
                .collect::<Vec<_>>(),
            vec![lint.id]
        );
    }

    #[test]
    fn build_source_evaluator_supports_install_handle_depend_on_chains() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var lint = graph.step(\"lint\");\n",
            "    var app = graph.add_exe({ name = \"app\", root = \"src/app.fol\" });\n",
            "    graph.install(app).depend_on(lint);\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("install-handle chaining should evaluate")
            .expect("build body should produce operations");

        let install = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "install")
            .expect("install step should exist");
        let lint = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "lint")
            .expect("lint step should exist");
        assert_eq!(
            evaluated
                .result
                .graph
                .step_dependencies_for(install.id)
                .collect::<Vec<_>>(),
            vec![lint.id]
        );
    }

    #[test]
    fn build_source_evaluator_keeps_step_like_handle_chains_stable() {
        let source = concat!(
            "pro[] build(graph: Graph): non = {\n",
            "    var lint = graph.step(\"lint\");\n",
            "    var app = graph.add_exe({ name = \"app\", root = \"src/app.fol\" });\n",
            "    var run_app = graph.add_run(app);\n",
            "    var install_app = graph.install(app);\n",
            "    run_app.depend_on(lint);\n",
            "    install_app.depend_on(lint);\n",
            "    graph.step(\"bundle\", run_app, install_app, run_app).depend_on(lint);\n",
            "    return graph\n",
            "}\n",
        );
        let (package_root, build_path) = temp_build_package(source);
        let request = BuildEvaluationRequest {
            package_root: package_root.display().to_string(),
            inputs: BuildEvaluationInputs {
                working_directory: package_root.display().to_string(),
                ..BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        };

        let evaluated = evaluate_build_source(&request, &build_path, source)
            .expect("combined handle chaining should evaluate")
            .expect("build body should produce operations");

        let lint = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "lint")
            .expect("lint step should exist");
        let run = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "run")
            .expect("run step should exist");
        let install = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "install")
            .expect("install step should exist");
        let bundle = evaluated
            .result
            .graph
            .steps()
            .iter()
            .find(|step| step.name == "bundle")
            .expect("bundle step should exist");

        assert_eq!(
            evaluated
                .result
                .graph
                .step_dependencies_for(run.id)
                .collect::<Vec<_>>(),
            vec![lint.id]
        );
        assert_eq!(
            evaluated
                .result
                .graph
                .step_dependencies_for(install.id)
                .collect::<Vec<_>>(),
            vec![lint.id]
        );
        assert_eq!(
            evaluated
                .result
                .graph
                .step_dependencies_for(bundle.id)
                .collect::<Vec<_>>(),
            vec![run.id, install.id, lint.id]
        );
    }

    #[test]
    fn evaluated_build_program_surface_keeps_runtime_metadata_and_graph_result() {
        let result = BuildEvaluationResult::new(
            BuildEvaluationBoundary::GraphConstructionSubset,
            canonical_graph_construction_capabilities(),
            "/pkg",
            BuildOptionDeclarationSet::new(),
            ResolvedBuildOptionSet::new(),
            Vec::new(),
            BuildGraph::new(),
        );
        let evaluated = EvaluatedBuildProgram {
            program: crate::runtime::BuildRuntimeProgram::new(
                crate::runtime::BuildExecutionRepresentation::RestrictedRuntimeIr,
            ),
            artifacts: vec![crate::runtime::BuildRuntimeArtifact::new(
                "app",
                crate::runtime::BuildRuntimeArtifactKind::Executable,
                "src/app.fol",
            )],
            generated_files: vec![crate::runtime::BuildRuntimeGeneratedFile::new(
                "version",
                "gen/version.fol",
                crate::runtime::BuildRuntimeGeneratedFileKind::Write,
            )],
            dependencies: vec![crate::runtime::BuildRuntimeDependency {
                alias: "core".to_string(),
                package: "org/core".to_string(),
                evaluation_mode: None,
            }],
            dependency_queries: Vec::new(),
            step_bindings: vec![crate::runtime::BuildRuntimeStepBinding::new(
                "run",
                crate::runtime::BuildRuntimeStepBindingKind::DefaultRun,
                Some("app"),
            )],
            result,
        };

        assert_eq!(evaluated.artifacts.len(), 1);
        assert_eq!(evaluated.generated_files.len(), 1);
        assert_eq!(evaluated.dependencies.len(), 1);
        assert_eq!(evaluated.step_bindings.len(), 1);
        assert_eq!(evaluated.result.package_root, "/pkg");
    }
}
