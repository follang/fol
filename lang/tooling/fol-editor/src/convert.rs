//! Diagnostic-to-LSP conversion contract.
//!
//! This module defines the stable bridge between `fol_diagnostics::Diagnostic`
//! (compiler-owned) and `LspDiagnostic` (editor wire format).
//!
//! ## Contract
//!
//! - **Severity**: `Error` → `Error(1)`, `Warning` → `Warning(2)`, `Info` → `Information(3)`
//! - **Code**: `diagnostic.code.as_str()` copied verbatim
//! - **Source**: always `"fol"`
//! - **Message**: `[{code}] {message}`, followed by `\nnotes:\n- ...` and `\nhelps:\n- ...` when present
//! - **Range**: 1-indexed `(line, column, length)` → 0-indexed LSP `(line, character)` range on a single line
//! - **Related information**: secondary labels with file paths → `LspDiagnosticRelatedInformation`
//!
//! ## Dedup layers
//!
//! Two dedup layers exist by design:
//!
//! - **`DiagnosticReport` (compiler)**: consecutive same-code + same-line suppression, hard cap at 50
//! - **`dedup_diagnostics` (editor)**: HashSet-based `(line, code)` dedup across all diagnostics
//!
//! The compiler layer catches cascades at production time. The editor layer catches
//! cross-stage duplicates after conversion (e.g., parser and resolver both flagging the same line).

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
    fn contract_source_is_always_fol() {
        let diagnostic = Diagnostic::error("P1001", "syntax error");
        let lsp = diagnostic_to_lsp(&diagnostic);
        assert_eq!(lsp.source, "fol");
    }

    #[test]
    fn contract_severity_maps_error_warning_info() {
        let err = diagnostic_to_lsp(&Diagnostic::error("E0001", "e"));
        let warn = diagnostic_to_lsp(&Diagnostic::warning("W0001", "w"));
        let info = diagnostic_to_lsp(&Diagnostic::info("I0001", "i"));
        assert_eq!(err.severity, LspDiagnosticSeverity::Error);
        assert_eq!(warn.severity, LspDiagnosticSeverity::Warning);
        assert_eq!(info.severity, LspDiagnosticSeverity::Information);
    }

    #[test]
    fn contract_code_is_diagnostic_code_verbatim() {
        let diagnostic = Diagnostic::error("R1003", "unresolved");
        let lsp = diagnostic_to_lsp(&diagnostic);
        assert_eq!(lsp.code, "R1003");
    }

    #[test]
    fn contract_message_prefixed_with_code_in_brackets() {
        let diagnostic = Diagnostic::error("T1003", "type mismatch");
        let lsp = diagnostic_to_lsp(&diagnostic);
        assert_eq!(lsp.message, "[T1003] type mismatch");
    }

    #[test]
    fn contract_notes_appended_after_message() {
        let diagnostic = Diagnostic::error("R1003", "unresolved")
            .with_note("no matching declaration found");
        let lsp = diagnostic_to_lsp(&diagnostic);
        assert!(lsp.message.contains("\nnotes:\n- no matching declaration found"));
    }

    #[test]
    fn contract_helps_appended_after_notes() {
        let diagnostic = Diagnostic::error("R1003", "unresolved")
            .with_note("context")
            .with_help("add an import");
        let lsp = diagnostic_to_lsp(&diagnostic);
        let note_pos = lsp.message.find("notes:").unwrap();
        let help_pos = lsp.message.find("helps:").unwrap();
        assert!(help_pos > note_pos);
        assert!(lsp.message.contains("\nhelps:\n- add an import"));
    }

    #[test]
    fn contract_missing_location_defaults_to_line_0_char_0() {
        let diagnostic = Diagnostic::error("P1001", "no location");
        let lsp = diagnostic_to_lsp(&diagnostic);
        assert_eq!(lsp.range.start.line, 0);
        assert_eq!(lsp.range.start.character, 0);
    }

    #[test]
    fn contract_secondary_labels_without_file_excluded_from_related() {
        let diagnostic = Diagnostic::error("R1003", "test")
            .with_secondary_label(
                DiagnosticLocation { file: None, line: 1, column: 1, length: Some(1) },
                "no file",
            );
        let lsp = diagnostic_to_lsp(&diagnostic);
        assert!(lsp.related_information.is_empty());
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
        assert!(lsp.message.starts_with("[R1003] "));
        assert!(lsp.message.contains("unresolved value"));
        assert!(lsp.message.contains("check imports"));
        assert!(lsp.message.contains("add a matching binding"));
        assert_eq!(lsp.related_information.len(), 1);
        assert_eq!(lsp.related_information[0].message, "related");
    }
}
