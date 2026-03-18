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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendError {
    kind: FrontendErrorKind,
    message: String,
    notes: Vec<String>,
}

impl FrontendError {
    pub fn new(kind: FrontendErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            notes: Vec::new(),
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
}
