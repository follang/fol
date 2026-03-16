use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendErrorKind {
    InvalidInput,
    WorkspaceNotFound,
    CommandFailed,
    Internal,
}

impl FrontendErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "FrontendInvalidInput",
            Self::WorkspaceNotFound => "FrontendWorkspaceNotFound",
            Self::CommandFailed => "FrontendCommandFailed",
            Self::Internal => "FrontendInternal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendError {
    kind: FrontendErrorKind,
    message: String,
}

impl FrontendError {
    pub fn new(kind: FrontendErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> FrontendErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
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

#[cfg(test)]
mod tests {
    use super::{FrontendError, FrontendErrorKind};

    #[test]
    fn frontend_error_formats_with_stable_kind_prefix() {
        let error = FrontendError::new(FrontendErrorKind::WorkspaceNotFound, "missing root");

        assert_eq!(error.kind(), FrontendErrorKind::WorkspaceNotFound);
        assert_eq!(error.message(), "missing root");
        assert_eq!(
            error.to_string(),
            "FrontendWorkspaceNotFound: missing root"
        );
    }
}
