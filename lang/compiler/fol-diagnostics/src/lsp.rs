//! Compiler-owned LSP diagnostic adapter.
//!
//! This module provides the canonical conversion from `Diagnostic` to
//! LSP-compatible wire types. Editor crates consume these types instead
//! of building their own conversion logic.
//!
//! ## Contract
//!
//! - **Severity**: `Error` → 1, `Warning` → 2, `Info` → 3
//! - **Code**: `diagnostic.code.as_str()` copied verbatim
//! - **Source**: always `"fol"`
//! - **Message**: `[{code}] {message}`, with notes and helps appended
//! - **Range**: 1-indexed `(line, column, length)` → 0-indexed `(line, character)`
//! - **Related info**: secondary labels with file paths

use crate::{Diagnostic, DiagnosticLabelKind, DiagnosticLocation, Severity};
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

    let mut message = format!("[{}] {}", diagnostic.code.as_str(), diagnostic.message);
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

/// Deduplicate LSP diagnostics by (line, code), keeping only the first
/// diagnostic for each unique pair.
pub fn dedup_lsp_diagnostics(diagnostics: Vec<LspDiagnostic>) -> Vec<LspDiagnostic> {
    let mut seen = std::collections::HashSet::new();
    diagnostics
        .into_iter()
        .filter(|d| seen.insert((d.range.start.line, d.code.clone())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DiagnosticLocation;

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
    fn contract_source_is_always_fol() {
        let lsp = diagnostic_to_lsp(&Diagnostic::error("P1001", "test"));
        assert_eq!(lsp.source, "fol");
    }

    #[test]
    fn contract_severity_maps_all_variants() {
        assert_eq!(
            diagnostic_to_lsp(&Diagnostic::error("E0001", "e")).severity,
            LspDiagnosticSeverity::Error
        );
        assert_eq!(
            diagnostic_to_lsp(&Diagnostic::warning("W0001", "w")).severity,
            LspDiagnosticSeverity::Warning
        );
        assert_eq!(
            diagnostic_to_lsp(&Diagnostic::info("I0001", "i")).severity,
            LspDiagnosticSeverity::Information
        );
    }

    #[test]
    fn contract_code_is_verbatim() {
        let lsp = diagnostic_to_lsp(&Diagnostic::error("R1003", "unresolved"));
        assert_eq!(lsp.code, "R1003");
    }

    #[test]
    fn contract_message_prefixed_with_code() {
        let lsp = diagnostic_to_lsp(&Diagnostic::error("T1003", "type mismatch"));
        assert_eq!(lsp.message, "[T1003] type mismatch");
    }

    #[test]
    fn contract_notes_and_helps_appended() {
        let diagnostic = Diagnostic::error("R1003", "unresolved")
            .with_note("context")
            .with_help("add import");
        let lsp = diagnostic_to_lsp(&diagnostic);
        assert!(lsp.message.contains("\nnotes:\n- context"));
        assert!(lsp.message.contains("\nhelps:\n- add import"));
    }

    #[test]
    fn contract_missing_location_defaults_to_origin() {
        let lsp = diagnostic_to_lsp(&Diagnostic::error("P1001", "no location"));
        assert_eq!(lsp.range.start.line, 0);
        assert_eq!(lsp.range.start.character, 0);
    }

    #[test]
    fn contract_secondary_labels_become_related_info() {
        let diagnostic = Diagnostic::error("R1003", "test")
            .with_primary_label_message(
                DiagnosticLocation {
                    file: Some("/tmp/a.fol".to_string()),
                    line: 2,
                    column: 5,
                    length: Some(6),
                },
                "here",
            )
            .with_secondary_label(
                DiagnosticLocation {
                    file: Some("/tmp/b.fol".to_string()),
                    line: 1,
                    column: 1,
                    length: Some(3),
                },
                "related",
            );
        let lsp = diagnostic_to_lsp(&diagnostic);
        assert_eq!(lsp.related_information.len(), 1);
        assert_eq!(lsp.related_information[0].message, "related");
    }

    #[test]
    fn contract_secondary_without_file_excluded() {
        let diagnostic = Diagnostic::error("R1003", "test").with_secondary_label(
            DiagnosticLocation {
                file: None,
                line: 1,
                column: 1,
                length: Some(1),
            },
            "no file",
        );
        let lsp = diagnostic_to_lsp(&diagnostic);
        assert!(lsp.related_information.is_empty());
    }

    #[test]
    fn dedup_removes_same_line_and_code() {
        let d1 = diagnostic_to_lsp(
            &Diagnostic::error("P1001", "first").with_primary_label(DiagnosticLocation {
                file: Some("a.fol".to_string()),
                line: 5,
                column: 1,
                length: Some(1),
            }),
        );
        let d2 = diagnostic_to_lsp(
            &Diagnostic::error("P1001", "second").with_primary_label(DiagnosticLocation {
                file: Some("a.fol".to_string()),
                line: 5,
                column: 3,
                length: Some(2),
            }),
        );
        let result = dedup_lsp_diagnostics(vec![d1, d2]);
        assert_eq!(result.len(), 1);
        assert!(result[0].message.contains("first"));
    }
}
