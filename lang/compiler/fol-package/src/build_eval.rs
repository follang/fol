use crate::build_graph::BuildGraph;
use fol_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticLocation, ToDiagnostic, ToDiagnosticLocation,
};
use fol_parser::ast::SyntaxOrigin;
use fol_types::Glitch;

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

#[cfg(test)]
mod tests {
    use super::{
        AllowedBuildTimeOperation, BuildEvaluationBoundary, BuildEvaluationError,
        BuildEvaluationErrorKind, BuildEvaluationRequest, BuildEvaluationResult,
    };
    use crate::build_graph::BuildGraph;
    use fol_diagnostics::{DiagnosticCode, ToDiagnostic};
    use fol_parser::ast::SyntaxOrigin;

    #[test]
    fn build_evaluation_request_defaults_to_an_empty_package_root() {
        let request = BuildEvaluationRequest::default();

        assert!(request.package_root.is_empty());
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
}
