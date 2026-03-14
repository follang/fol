#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringError {
    message: String,
}

impl LoweringError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for LoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LoweringError: {}", self.message)
    }
}

impl std::error::Error for LoweringError {}
