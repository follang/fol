use crate::DiagnosticLocation;

pub fn load_source_line(location: &DiagnosticLocation) -> Option<String> {
    let file = location.file.as_ref()?;
    let contents = std::fs::read_to_string(file).ok()?;
    contents
        .lines()
        .nth(location.line.saturating_sub(1))
        .map(|line| line.to_string())
}

pub fn primary_underline(location: &DiagnosticLocation) -> String {
    let start = location.column.saturating_sub(1);
    let width = location.length.unwrap_or(1).max(1);
    format!("{}{}", " ".repeat(start), "^".repeat(width))
}
