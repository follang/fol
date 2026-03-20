use crate::{
    EditorDocumentPath, EditorDocumentUri, EditorError, EditorErrorKind, EditorResult, LspPosition,
    LspRange,
};
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
        let document = self.documents.get_mut(uri.as_str()).ok_or_else(|| {
            EditorError::new(
                EditorErrorKind::DocumentNotOpen,
                format!("document '{}' is not open", uri.as_str()),
            )
        })?;
        document.version = version;
        document.text = text;
        Ok(())
    }

    pub fn apply_incremental_change(
        &mut self,
        uri: &EditorDocumentUri,
        version: i32,
        range: LspRange,
        text: String,
    ) -> EditorResult<()> {
        let document = self.documents.get_mut(uri.as_str()).ok_or_else(|| {
            EditorError::new(
                EditorErrorKind::DocumentNotOpen,
                format!("document '{}' is not open", uri.as_str()),
            )
        })?;
        let start = position_to_offset(&document.text, range.start).ok_or_else(|| {
            EditorError::new(
                EditorErrorKind::InvalidInput,
                format!(
                    "invalid incremental change start {}:{} for '{}'",
                    range.start.line, range.start.character, uri.as_str()
                ),
            )
        })?;
        let end = position_to_offset(&document.text, range.end).ok_or_else(|| {
            EditorError::new(
                EditorErrorKind::InvalidInput,
                format!(
                    "invalid incremental change end {}:{} for '{}'",
                    range.end.line, range.end.character, uri.as_str()
                ),
            )
        })?;
        if end < start {
            return Err(EditorError::new(
                EditorErrorKind::InvalidInput,
                format!(
                    "incremental change end precedes start for '{}'",
                    uri.as_str()
                ),
            ));
        }

        document.text.replace_range(start..end, &text);
        document.version = version;
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

fn position_to_offset(text: &str, position: LspPosition) -> Option<usize> {
    let mut line = 0u32;
    let mut character = 0u32;
    for (offset, ch) in text.char_indices() {
        if line == position.line && character == position.character {
            return Some(offset);
        }
        if ch == '\n' {
            line += 1;
            character = 0;
            if line == position.line && position.character == 0 {
                return Some(offset + ch.len_utf8());
            }
        } else if line == position.line {
            character += 1;
        }
    }

    (line == position.line && character == position.character).then_some(text.len())
}

#[cfg(test)]
mod tests {
    use super::{EditorDocument, EditorDocumentStore};
    use crate::{EditorDocumentUri, EditorErrorKind, LspPosition, LspRange};
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

    #[test]
    fn document_store_applies_incremental_insertions() {
        let uri = EditorDocumentUri::from_file_path(PathBuf::from("/tmp/demo.fol")).unwrap();
        let document = EditorDocument::new(uri.clone(), 1, "fun[] main(): int = {\n    return 0\n}\n".to_string()).unwrap();
        let mut store = EditorDocumentStore::default();

        store.open(document);
        store
            .apply_incremental_change(
                &uri,
                2,
                LspRange {
                    start: LspPosition {
                        line: 1,
                        character: 11,
                    },
                    end: LspPosition {
                        line: 1,
                        character: 11,
                    },
                },
                "value + ".to_string(),
            )
            .unwrap();

        let current = store.get(&uri).unwrap();
        assert_eq!(current.version, 2);
        assert_eq!(current.text, "fun[] main(): int = {\n    return value + 0\n}\n");
    }

    #[test]
    fn document_store_applies_incremental_replacements_across_lines() {
        let uri = EditorDocumentUri::from_file_path(PathBuf::from("/tmp/demo.fol")).unwrap();
        let document = EditorDocument::new(
            uri.clone(),
            1,
            "fun[] main(): int = {\n    var value: int = 7\n    return value\n}\n".to_string(),
        )
        .unwrap();
        let mut store = EditorDocumentStore::default();

        store.open(document);
        store
            .apply_incremental_change(
                &uri,
                2,
                LspRange {
                    start: LspPosition {
                        line: 1,
                        character: 4,
                    },
                    end: LspPosition {
                        line: 2,
                        character: 16,
                    },
                },
                "return 9".to_string(),
            )
            .unwrap();

        assert_eq!(
            store.get(&uri).unwrap().text,
            "fun[] main(): int = {\n    return 9\n}\n"
        );
    }

    #[test]
    fn document_store_rejects_invalid_incremental_ranges() {
        let uri = EditorDocumentUri::from_file_path(PathBuf::from("/tmp/demo.fol")).unwrap();
        let document = EditorDocument::new(uri.clone(), 1, "fun[] main(): int = 0\n".to_string()).unwrap();
        let mut store = EditorDocumentStore::default();

        store.open(document);
        let error = store
            .apply_incremental_change(
                &uri,
                2,
                LspRange {
                    start: LspPosition {
                        line: 10,
                        character: 0,
                    },
                    end: LspPosition {
                        line: 10,
                        character: 1,
                    },
                },
                "x".to_string(),
            )
            .unwrap_err();

        assert_eq!(error.kind, EditorErrorKind::InvalidInput);
    }
}
