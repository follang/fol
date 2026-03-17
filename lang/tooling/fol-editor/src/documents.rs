use crate::{EditorDocumentPath, EditorDocumentUri, EditorError, EditorErrorKind, EditorResult};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDocument {
    pub uri: EditorDocumentUri,
    pub path: EditorDocumentPath,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EditorDocumentStore {
    documents: BTreeMap<String, EditorDocument>,
}

impl EditorDocument {
    pub fn new(uri: EditorDocumentUri, version: i32, text: String) -> EditorResult<Self> {
        let path = EditorDocumentPath::new(uri.to_file_path()?);
        Ok(Self {
            uri,
            path,
            version,
            text,
        })
    }
}

impl EditorDocumentStore {
    pub fn open(&mut self, document: EditorDocument) {
        self.documents
            .insert(document.uri.as_str().to_string(), document);
    }

    pub fn apply_full_change(
        &mut self,
        uri: &EditorDocumentUri,
        version: i32,
        text: String,
    ) -> EditorResult<()> {
        let document = self
            .documents
            .get_mut(uri.as_str())
            .ok_or_else(|| {
                EditorError::new(
                    EditorErrorKind::DocumentNotOpen,
                    format!("document '{}' is not open", uri.as_str()),
                )
            })?;
        document.version = version;
        document.text = text;
        Ok(())
    }

    pub fn close(&mut self, uri: &EditorDocumentUri) -> Option<EditorDocument> {
        self.documents.remove(uri.as_str())
    }

    pub fn get(&self, uri: &EditorDocumentUri) -> Option<&EditorDocument> {
        self.documents.get(uri.as_str())
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{EditorDocument, EditorDocumentStore};
    use crate::{EditorDocumentUri, EditorErrorKind};
    use std::path::PathBuf;

    #[test]
    fn document_store_tracks_open_change_and_close() {
        let uri = EditorDocumentUri::from_file_path(PathBuf::from("/tmp/demo.fol")).unwrap();
        let document = EditorDocument::new(uri.clone(), 1, "old".to_string()).unwrap();
        let mut store = EditorDocumentStore::default();

        store.open(document);
        store
            .apply_full_change(&uri, 2, "new".to_string())
            .expect("open documents should update");

        let current = store.get(&uri).unwrap();
        assert_eq!(current.version, 2);
        assert_eq!(current.text, "new");
        assert!(store.close(&uri).is_some());
        assert!(store.is_empty());
    }

    #[test]
    fn document_store_reports_missing_documents_explicitly() {
        let uri = EditorDocumentUri::from_file_path(PathBuf::from("/tmp/demo.fol")).unwrap();
        let mut store = EditorDocumentStore::default();

        let error = store
            .apply_full_change(&uri, 1, "text".to_string())
            .unwrap_err();
        assert_eq!(error.kind, EditorErrorKind::DocumentNotOpen);
    }
}
