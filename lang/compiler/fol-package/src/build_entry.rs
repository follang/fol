#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEntrySignatureExpectation {
    pub parameter_type_names: Vec<String>,
    pub return_type_names: Vec<String>,
}

impl BuildEntrySignatureExpectation {
    pub fn canonical() -> Self {
        Self {
            parameter_type_names: vec!["Graph".to_string(), "build::Graph".to_string()],
            return_type_names: vec!["Graph".to_string(), "build::Graph".to_string()],
        }
    }

    pub fn accepts_parameter_type(&self, name: &str) -> bool {
        self.parameter_type_names.iter().any(|candidate| candidate == name)
    }

    pub fn accepts_return_type(&self, name: &str) -> bool {
        self.return_type_names.iter().any(|candidate| candidate == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEntryCandidate {
    pub source_unit_path: String,
    pub syntax_id: fol_parser::ast::SyntaxNodeId,
    pub name: String,
    pub parameter_names: Vec<String>,
    pub parameter_type_names: Vec<Option<String>>,
    pub return_type_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedBuildEntry {
    pub candidate: BuildEntryCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEntryValidationErrorKind {
    MissingEntry,
    MultipleEntries,
    WrongParameterCount,
    WrongParameterType,
    WrongReturnType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEntryValidationError {
    pub kind: BuildEntryValidationErrorKind,
    pub message: String,
    pub origin: Option<fol_parser::ast::SyntaxOrigin>,
}

impl BuildEntryValidationError {
    pub fn new(kind: BuildEntryValidationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            origin: None,
        }
    }

    pub fn with_origin(
        kind: BuildEntryValidationErrorKind,
        message: impl Into<String>,
        origin: fol_parser::ast::SyntaxOrigin,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            origin: Some(origin),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildEntryCandidate, BuildEntrySignatureExpectation, BuildEntryValidationError,
        BuildEntryValidationErrorKind, ValidatedBuildEntry,
    };

    #[test]
    fn canonical_build_entry_signature_expectation_keeps_graph_names() {
        let expectation = BuildEntrySignatureExpectation::canonical();

        assert!(expectation.accepts_parameter_type("Graph"));
        assert!(expectation.accepts_parameter_type("build::Graph"));
        assert!(expectation.accepts_return_type("Graph"));
        assert!(expectation.accepts_return_type("build::Graph"));
        assert!(!expectation.accepts_parameter_type("int"));
    }

    #[test]
    fn build_entry_candidate_and_validation_error_models_capture_core_metadata() {
        let candidate = BuildEntryCandidate {
            source_unit_path: "build.fol".to_string(),
            syntax_id: fol_parser::ast::SyntaxNodeId(7),
            name: "build".to_string(),
            parameter_names: vec!["graph".to_string()],
            parameter_type_names: vec![Some("Graph".to_string())],
            return_type_name: Some("Graph".to_string()),
        };
        let validated = ValidatedBuildEntry {
            candidate: candidate.clone(),
        };
        let error = BuildEntryValidationError::new(
            BuildEntryValidationErrorKind::WrongReturnType,
            "wrong return type",
        );

        assert_eq!(validated.candidate, candidate);
        assert_eq!(error.kind, BuildEntryValidationErrorKind::WrongReturnType);
        assert!(error.origin.is_none());
    }
}
