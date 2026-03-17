use crate::build_graph::BuildGraph;
use crate::{
    BuildApi, DependencyRequest, ExecutableRequest, InstallDirRequest, InstallFileRequest,
    SharedLibraryRequest, StandardOptimizeRequest, StandardTargetRequest, StaticLibraryRequest,
    TestArtifactRequest, UserOptionRequest,
};
use fol_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticLocation, ToDiagnostic, ToDiagnosticLocation,
};
use fol_parser::ast::SyntaxOrigin;
use fol_types::Glitch;
use std::collections::BTreeMap;

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
    pub graph: BuildGraph,
}

impl BuildEvaluationResult {
    pub fn new(
        boundary: BuildEvaluationBoundary,
        allowed_operations: Vec<AllowedBuildTimeOperation>,
        package_root: impl Into<String>,
        graph: BuildGraph,
    ) -> Self {
        Self {
            boundary,
            allowed_operations,
            package_root: package_root.into(),
            graph,
        }
    }
}

pub fn evaluate_build_plan(
    request: &BuildEvaluationRequest,
) -> Result<BuildEvaluationResult, BuildEvaluationError> {
    let mut step_names = BTreeMap::new();
    let mut artifact_names = BTreeMap::new();
    let mut graph = BuildGraph::new();
    let mut api = BuildApi::new(&mut graph);

    for operation in &request.operations {
        match &operation.kind {
            BuildEvaluationOperationKind::StandardTarget(operation_request) => {
                api.standard_target(operation_request.clone());
            }
            BuildEvaluationOperationKind::StandardOptimize(operation_request) => {
                api.standard_optimize(operation_request.clone());
            }
            BuildEvaluationOperationKind::Option(operation_request) => {
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
        graph,
    ))
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
        evaluate_build_plan, AllowedBuildTimeOperation, BuildEvaluationBoundary,
        BuildEvaluationError, BuildEvaluationErrorKind, BuildEvaluationInputs,
        BuildEvaluationInstallArtifactRequest, BuildEvaluationOperation,
        BuildEvaluationOperationKind, BuildEvaluationRequest, BuildEvaluationResult,
        BuildEvaluationRunRequest, BuildEvaluationStepRequest,
    };
    use crate::build_graph::BuildGraph;
    use crate::{ExecutableRequest, StandardOptimizeRequest, StandardTargetRequest, UserOptionRequest};
    use fol_diagnostics::{DiagnosticCode, ToDiagnostic};
    use fol_parser::ast::SyntaxOrigin;
    use std::collections::BTreeMap;

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
}
