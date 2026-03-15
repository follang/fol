use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendErrorKind {
    InvalidInput,
    Unsupported,
    EmissionFailure,
    BuildFailure,
    Internal,
}

impl BackendErrorKind {
    fn label(self) -> &'static str {
        match self {
            Self::InvalidInput => "BackendInvalidInput",
            Self::Unsupported => "BackendUnsupported",
            Self::EmissionFailure => "BackendEmissionFailure",
            Self::BuildFailure => "BackendBuildFailure",
            Self::Internal => "BackendInternal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    kind: BackendErrorKind,
    message: String,
}

impl BackendError {
    pub fn new(kind: BackendErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> BackendErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind.label(), self.message)
    }
}

impl std::error::Error for BackendError {}
