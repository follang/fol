use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorErrorKind {
    InvalidInput,
    InvalidDocumentUri,
    InvalidDocumentPath,
    DocumentNotOpen,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorError {
    pub kind: EditorErrorKind,
    pub message: String,
    pub notes: Vec<String>,
}

pub type EditorResult<T> = Result<T, EditorError>;

impl EditorError {
    pub fn new(kind: EditorErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            notes: Vec::new(),
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for EditorError {}

#[cfg(test)]
mod tests {
    use super::{EditorError, EditorErrorKind};

    #[test]
    fn editor_error_can_carry_notes() {
        let error = EditorError::new(EditorErrorKind::InvalidInput, "bad request")
            .with_note("check the document uri");

        assert_eq!(error.kind, EditorErrorKind::InvalidInput);
        assert_eq!(error.notes, vec!["check the document uri".to_string()]);
    }

    #[test]
    fn editor_error_formats_with_stable_kind_prefix() {
        let error = EditorError::new(EditorErrorKind::Internal, "boom");
        assert_eq!(error.to_string(), "Internal: boom");
    }
}
