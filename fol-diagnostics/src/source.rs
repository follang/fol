use crate::DiagnosticLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceSnippetError {
    MissingFilePath,
    ReadFailed,
    MissingLine,
}

pub fn load_source_line(location: &DiagnosticLocation) -> Result<String, SourceSnippetError> {
    let file = location
        .file
        .as_ref()
        .ok_or(SourceSnippetError::MissingFilePath)?;
    let contents = std::fs::read_to_string(file).map_err(|_| SourceSnippetError::ReadFailed)?;
    contents
        .lines()
        .nth(location.line.saturating_sub(1))
        .map(|line| line.to_string())
        .ok_or(SourceSnippetError::MissingLine)
}

pub fn primary_underline(location: &DiagnosticLocation) -> String {
    let start = location.column.saturating_sub(1);
    let width = location.length.unwrap_or(1).max(1);
    format!("{}{}", " ".repeat(start), "^".repeat(width))
}
