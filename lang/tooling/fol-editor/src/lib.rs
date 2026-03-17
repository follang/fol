//! Editor tooling foundations for the FOL language.
//!
//! `fol-editor` will host both the Tree-sitter-facing editor syntax layer and
//! the compiler-backed language-server layer.

mod documents;
mod error;
mod paths;
mod session;

pub use documents::{EditorDocument, EditorDocumentStore};
pub use error::{EditorError, EditorErrorKind, EditorResult};
pub use paths::{EditorDocumentPath, EditorDocumentUri};
pub use session::{EditorConfig, EditorSession};

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
        crate_name, Editor, EditorConfig, EditorDocument, EditorDocumentPath, EditorDocumentStore,
        EditorDocumentUri, EditorError, EditorErrorKind, EditorResult, EditorSession, CRATE_NAME,
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

    #[test]
    fn document_store_and_session_shells_are_constructible() {
        let uri = EditorDocumentUri::from_file_path(PathBuf::from("/tmp/demo.fol")).unwrap();
        let document = EditorDocument::new(uri, 1, "fun main(): int = 0".to_string()).unwrap();
        let mut store = EditorDocumentStore::default();
        let config = EditorConfig::default();
        let mut session = EditorSession::new(config.clone());

        store.open(document.clone());
        session.documents.open(document);

        assert_eq!(store.len(), 1);
        assert_eq!(session.config, config);
        assert_eq!(session.documents.len(), 1);
    }
}
