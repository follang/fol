//! Editor tooling foundations for the FOL language.
//!
//! `fol-editor` will host both the Tree-sitter-facing editor syntax layer and
//! the compiler-backed language-server layer.

mod error;
mod paths;

pub use error::{EditorError, EditorErrorKind, EditorResult};
pub use paths::{EditorDocumentPath, EditorDocumentUri};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Editor;

pub const CRATE_NAME: &str = "fol-editor";

impl Editor {
    pub fn new() -> Self {
        Self
    }
}

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

#[cfg(test)]
mod tests {
    use super::{
        crate_name, Editor, EditorDocumentPath, EditorDocumentUri, EditorError,
        EditorErrorKind, EditorResult, CRATE_NAME,
    };
    use std::path::PathBuf;

    #[test]
    fn crate_name_matches_editor_identity() {
        assert_eq!(crate_name(), CRATE_NAME);
    }

    #[test]
    fn public_editor_shell_is_constructible() {
        assert_eq!(Editor::new(), Editor);
    }

    #[test]
    fn public_editor_types_are_constructible() {
        let error = EditorError::new(EditorErrorKind::Internal, "boom");
        let uri = EditorDocumentUri::from_file_path(PathBuf::from("/tmp/demo.fol")).unwrap();
        let path = EditorDocumentPath::new(PathBuf::from("/tmp/demo.fol"));
        let result: EditorResult<()> = Ok(());

        assert_eq!(error.kind, EditorErrorKind::Internal);
        assert_eq!(uri.as_str(), "file:///tmp/demo.fol");
        assert_eq!(path.as_path(), PathBuf::from("/tmp/demo.fol").as_path());
        assert!(result.is_ok());
    }
}
