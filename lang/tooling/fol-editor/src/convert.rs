use fol_diagnostics::{Diagnostic, DiagnosticLabelKind, DiagnosticLocation, Severity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LspDiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspDiagnosticRelatedInformation {
    pub location: LspLocation,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspLocation {
    pub uri: String,
    pub range: LspRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub severity: LspDiagnosticSeverity,
    pub code: String,
    pub source: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_information: Vec<LspDiagnosticRelatedInformation>,
}

pub fn location_to_range(location: &DiagnosticLocation) -> LspRange {
    let line = location.line.saturating_sub(1) as u32;
    let start_character = location.column.saturating_sub(1) as u32;
    let end_character = start_character + location.length.unwrap_or(1).max(1) as u32;
    LspRange {
        start: LspPosition {
            line,
            character: start_character,
        },
        end: LspPosition {
            line,
            character: end_character,
        },
    }
}

pub fn diagnostic_to_lsp(diagnostic: &Diagnostic) -> LspDiagnostic {
    let primary = diagnostic
        .primary_location()
        .cloned()
        .or_else(|| {
            diagnostic
                .labels
                .first()
                .map(|label| label.location.clone())
        })
        .unwrap_or(DiagnosticLocation {
            file: None,
            line: 1,
            column: 1,
            length: Some(1),
        });

    let mut message = diagnostic.message.clone();
    if !diagnostic.notes.is_empty() {
        message.push_str("\nnotes:");
        for note in &diagnostic.notes {
            message.push_str("\n- ");
            message.push_str(note);
        }
    }
    if !diagnostic.helps.is_empty() {
        message.push_str("\nhelps:");
        for help in &diagnostic.helps {
            message.push_str("\n- ");
            message.push_str(help);
        }
    }

    let related_information = diagnostic
        .labels
        .iter()
        .filter(|label| label.kind == DiagnosticLabelKind::Secondary)
        .filter_map(|label| {
            label
                .location
                .file
                .as_ref()
                .map(|file| LspDiagnosticRelatedInformation {
                    location: LspLocation {
                        uri: format!("file://{file}"),
                        range: location_to_range(&label.location),
                    },
                    message: label
                        .message
                        .clone()
                        .unwrap_or_else(|| "related".to_string()),
                })
        })
        .collect::<Vec<_>>();

    LspDiagnostic {
        range: location_to_range(&primary),
        severity: match diagnostic.severity {
            Severity::Error => LspDiagnosticSeverity::Error,
            Severity::Warning => LspDiagnosticSeverity::Warning,
            Severity::Info => LspDiagnosticSeverity::Information,
        },
        code: diagnostic.code.as_str().to_string(),
        source: "fol".to_string(),
        message,
        related_information,
    }
}

#[cfg(test)]
mod tests {
    use super::{diagnostic_to_lsp, location_to_range, LspDiagnosticSeverity};
    use fol_diagnostics::{Diagnostic, DiagnosticLocation};

    #[test]
    fn locations_convert_to_zero_based_lsp_ranges() {
        let range = location_to_range(&DiagnosticLocation {
            file: Some("/tmp/demo.fol".to_string()),
            line: 4,
            column: 3,
            length: Some(5),
        });

        assert_eq!(range.start.line, 3);
        assert_eq!(range.start.character, 2);
        assert_eq!(range.end.character, 7);
    }

    #[test]
    fn diagnostics_convert_notes_helps_and_related_labels() {
        let diagnostic = Diagnostic::error("R1003", "unresolved value")
            .with_primary_label_message(
                DiagnosticLocation {
                    file: Some("/tmp/demo.fol".to_string()),
                    line: 2,
                    column: 5,
                    length: Some(6),
                },
                "here",
            )
            .with_secondary_label(
                DiagnosticLocation {
                    file: Some("/tmp/lib.fol".to_string()),
                    line: 1,
                    column: 1,
                    length: Some(3),
                },
                "related",
            )
            .with_note("check imports")
            .with_help("add a matching binding");

        let lsp = diagnostic_to_lsp(&diagnostic);

        assert_eq!(lsp.severity, LspDiagnosticSeverity::Error);
        assert_eq!(lsp.code, "R1003");
        assert!(lsp.message.contains("check imports"));
        assert!(lsp.message.contains("add a matching binding"));
        assert_eq!(lsp.related_information.len(), 1);
        assert_eq!(lsp.related_information[0].message, "related");
    }
}
