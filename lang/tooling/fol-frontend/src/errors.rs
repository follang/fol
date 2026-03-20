use fol_diagnostics::{Diagnostic, ToDiagnostic};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendErrorKind {
    InvalidInput,
    WorkspaceNotFound,
    PackageFailed,
    CommandFailed,
    Internal,
}

impl FrontendErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "FrontendInvalidInput",
            Self::WorkspaceNotFound => "FrontendWorkspaceNotFound",
            Self::PackageFailed => "FrontendPackageFailed",
            Self::CommandFailed => "FrontendCommandFailed",
            Self::Internal => "FrontendInternal",
        }
    }

    pub fn diagnostic_code(self) -> &'static str {
        match self {
            Self::InvalidInput => "F1001",
            Self::WorkspaceNotFound => "F1002",
            Self::PackageFailed => "F1003",
            Self::CommandFailed => "F1004",
            Self::Internal => "F1099",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendError {
    kind: FrontendErrorKind,
    message: String,
    notes: Vec<String>,
    diagnostics: Vec<Diagnostic>,
}

impl FrontendError {
    pub fn new(kind: FrontendErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            notes: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn from_errors<E: ToDiagnostic>(errors: Vec<E>) -> Self {
        let diagnostics: Vec<Diagnostic> = errors.iter().map(|e| e.to_diagnostic()).collect();
        let message = format!("compilation failed with {} error(s)", diagnostics.len());
        Self {
            kind: FrontendErrorKind::CommandFailed,
            message,
            notes: Vec::new(),
            diagnostics,
        }
    }

    pub fn kind(&self) -> FrontendErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.as_str(), self.message)
    }
}

impl std::error::Error for FrontendError {}

impl ToDiagnostic for FrontendError {
    fn to_diagnostic(&self) -> Diagnostic {
        let mut diagnostic =
            Diagnostic::error(self.kind.diagnostic_code(), self.message.clone());
        for note in &self.notes {
            diagnostic = diagnostic.with_note(note.clone());
        }
        diagnostic
    }
}

pub type FrontendResult<T> = Result<T, FrontendError>;

impl From<std::io::Error> for FrontendError {
    fn from(error: std::io::Error) -> Self {
        Self::new(FrontendErrorKind::CommandFailed, error.to_string())
    }
}

impl From<fol_package::PackageError> for FrontendError {
    fn from(error: fol_package::PackageError) -> Self {
        Self::new(FrontendErrorKind::PackageFailed, error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{FrontendError, FrontendErrorKind};

    #[test]
    fn frontend_error_formats_with_stable_kind_prefix() {
        let error = FrontendError::new(FrontendErrorKind::WorkspaceNotFound, "missing root");

        assert_eq!(error.kind(), FrontendErrorKind::WorkspaceNotFound);
        assert_eq!(error.message(), "missing root");
        assert_eq!(error.to_string(), "FrontendWorkspaceNotFound: missing root");
        assert!(error.notes().is_empty());
    }

    #[test]
    fn package_errors_lower_into_frontend_package_failed_kind() {
        let package_error = fol_package::PackageError::new(
            fol_package::PackageErrorKind::InvalidInput,
            "bad package",
        );
        let error = FrontendError::from(package_error);

        assert_eq!(error.kind(), FrontendErrorKind::PackageFailed);
        assert!(error.to_string().starts_with("FrontendPackageFailed:"));
    }

    #[test]
    fn frontend_error_can_carry_guidance_notes() {
        let error = FrontendError::new(FrontendErrorKind::InvalidInput, "bad input")
            .with_note("check package.yaml")
            .with_note("run `fol work info`");

        assert_eq!(
            error.notes(),
            &[
                "check package.yaml".to_string(),
                "run `fol work info`".to_string()
            ]
        );
    }

    #[test]
    fn frontend_error_to_diagnostic_carries_stable_code() {
        use fol_diagnostics::ToDiagnostic;

        let error = FrontendError::new(FrontendErrorKind::WorkspaceNotFound, "missing root")
            .with_note("check your working directory");

        let diagnostic = error.to_diagnostic();

        assert_eq!(diagnostic.code.as_str(), "F1002");
        assert_eq!(diagnostic.message, "missing root");
        assert_eq!(diagnostic.notes, vec!["check your working directory".to_string()]);
    }

    #[test]
    fn frontend_error_kind_diagnostic_codes_are_stable() {
        assert_eq!(FrontendErrorKind::InvalidInput.diagnostic_code(), "F1001");
        assert_eq!(FrontendErrorKind::WorkspaceNotFound.diagnostic_code(), "F1002");
        assert_eq!(FrontendErrorKind::PackageFailed.diagnostic_code(), "F1003");
        assert_eq!(FrontendErrorKind::CommandFailed.diagnostic_code(), "F1004");
        assert_eq!(FrontendErrorKind::Internal.diagnostic_code(), "F1099");
    }
}
