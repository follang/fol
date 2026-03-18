use crate::{EditorError, EditorErrorKind, EditorResult};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditorDocumentUri(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDocumentPath(PathBuf);

impl EditorDocumentUri {
    pub fn parse(input: &str) -> EditorResult<Self> {
        if !input.starts_with("file://") {
            return Err(EditorError::new(
                EditorErrorKind::InvalidDocumentUri,
                format!("unsupported document uri scheme in '{input}'"),
            ));
        }
        let raw_path = &input["file://".len()..];
        let decoded = percent_decode(raw_path).ok_or_else(|| {
            EditorError::new(
                EditorErrorKind::InvalidDocumentUri,
                format!("document uri '{input}' has invalid percent-encoding"),
            )
        })?;
        if !decoded.starts_with('/') {
            return Err(EditorError::new(
                EditorErrorKind::InvalidDocumentUri,
                format!("document uri '{input}' does not map to an absolute file path"),
            ));
        }
        Ok(Self(format!("file://{}", percent_encode(&decoded))))
    }

    pub fn from_file_path(path: PathBuf) -> EditorResult<Self> {
        if !path.is_absolute() {
            Err(EditorError::new(
                EditorErrorKind::InvalidDocumentPath,
                format!(
                    "path '{}' cannot be represented as a file uri",
                    path.display()
                ),
            ))
        } else {
            Ok(Self(format!(
                "file://{}",
                percent_encode(&path.to_string_lossy())
            )))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_file_path(&self) -> EditorResult<PathBuf> {
        if !self.0.starts_with("file://") {
            return Err(EditorError::new(
                EditorErrorKind::InvalidDocumentUri,
                format!("document uri '{}' does not use the file scheme", self.0),
            ));
        }
        let raw_path = &self.0["file://".len()..];
        let decoded = percent_decode(raw_path).ok_or_else(|| {
            EditorError::new(
                EditorErrorKind::InvalidDocumentUri,
                format!(
                    "document uri '{}' does not map to a local file path",
                    self.0
                ),
            )
        })?;
        Ok(PathBuf::from(decoded))
    }
}

impl EditorDocumentPath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn to_uri(&self) -> EditorResult<EditorDocumentUri> {
        EditorDocumentUri::from_file_path(self.0.clone())
    }
}

fn percent_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

fn percent_decode(input: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.as_bytes().iter().copied();
    while let Some(byte) = chars.next() {
        if byte == b'%' {
            let hi = chars.next()?;
            let lo = chars.next()?;
            let hi = from_hex(hi)?;
            let lo = from_hex(lo)?;
            bytes.push((hi << 4) | lo);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{EditorDocumentPath, EditorDocumentUri};
    use crate::EditorErrorKind;
    use std::path::PathBuf;

    #[test]
    fn file_paths_round_trip_through_document_uris() {
        let path = PathBuf::from("/tmp/demo.fol");
        let uri = EditorDocumentUri::from_file_path(path.clone()).unwrap();

        assert_eq!(uri.to_file_path().unwrap(), path);
    }

    #[test]
    fn uri_parser_rejects_non_file_schemes() {
        let error = EditorDocumentUri::parse("https://example.com/demo.fol").unwrap_err();
        assert_eq!(error.kind, EditorErrorKind::InvalidDocumentUri);
    }

    #[test]
    fn document_paths_convert_into_uris() {
        let path = EditorDocumentPath::new(PathBuf::from("/tmp/demo.fol"));
        assert_eq!(path.to_uri().unwrap().as_str(), "file:///tmp/demo.fol");
    }

    #[test]
    fn uri_parser_round_trips_percent_encoded_paths() {
        let uri = EditorDocumentUri::parse("file:///tmp/demo%20file.fol").unwrap();
        assert_eq!(
            uri.to_file_path().unwrap(),
            PathBuf::from("/tmp/demo file.fol")
        );
    }
}
