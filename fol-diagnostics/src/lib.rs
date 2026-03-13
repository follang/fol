// FOL Diagnostics - Error formatting and output
mod codes;
mod model;
mod render_human;
mod render_json;
mod source;

pub use codes::DiagnosticCode;
pub use model::{
    Diagnostic, DiagnosticLabel, DiagnosticLabelKind, DiagnosticLocation, DiagnosticReport,
    DiagnosticSuggestion, Severity,
};

/// Output format for diagnostics
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Human,
    Json,
}

impl DiagnosticReport {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
            Severity::Info => {}
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn add_error(&mut self, error: &dyn fol_types::Glitch, location: Option<DiagnosticLocation>) {
        let diagnostic = Diagnostic::from_glitch(error, Severity::Error, location);
        self.add_diagnostic(diagnostic);
    }

    pub fn add_coded_error(
        &mut self,
        code: impl Into<DiagnosticCode>,
        message: impl Into<String>,
        location: Option<DiagnosticLocation>,
    ) {
        let mut diagnostic = Diagnostic::error(code, message);
        if let Some(location) = location {
            diagnostic = diagnostic.with_primary_label(location);
        }
        self.add_diagnostic(diagnostic);
    }

    pub fn add_coded_warning(
        &mut self,
        code: impl Into<DiagnosticCode>,
        message: impl Into<String>,
        location: Option<DiagnosticLocation>,
    ) {
        let mut diagnostic = Diagnostic::warning(code, message);
        if let Some(location) = location {
            diagnostic = diagnostic.with_primary_label(location);
        }
        self.add_diagnostic(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Output the report in the specified format
    pub fn output(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => render_json::render_report(self),
            OutputFormat::Human => render_human::render_report(self),
        }
    }
}

impl Default for DiagnosticReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait to convert locations to diagnostic locations
pub trait ToDiagnosticLocation {
    fn to_diagnostic_location(&self, file: Option<String>) -> DiagnosticLocation;
}

impl DiagnosticLocation {
    pub fn from_point_location(loc: &impl PointLocationLike) -> Self {
        Self {
            file: loc.get_file_path(),
            line: loc.get_row(),
            column: loc.get_col(),
            length: Some(loc.get_len()),
        }
    }
}

/// Trait to abstract over point::Location without importing fol-lexer
pub trait PointLocationLike {
    fn get_file_path(&self) -> Option<String>;
    fn get_row(&self) -> usize;
    fn get_col(&self) -> usize;
    fn get_len(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;
    use fol_types::BasicError;

    #[test]
    fn test_diagnostic_report_json() {
        let mut report = DiagnosticReport::new();
        let error = BasicError {
            message: "Test error".to_string(),
        };

        report.add_error(&error, None);

        let json = report.output(OutputFormat::Json);
        assert!(json.contains("Test error"));
        assert!(json.contains("error_count"));
    }

    #[test]
    fn test_diagnostic_report_human() {
        let mut report = DiagnosticReport::new();
        let error = BasicError {
            message: "Test error".to_string(),
        };

        report.add_error(&error, None);

        let human = report.output(OutputFormat::Human);
        assert!(human.contains("Test error"));
        assert!(human.contains("found 1 error"));
    }

    #[test]
    fn test_diagnostic_report_json_keeps_location_fields_for_cli_consumers() {
        let mut report = DiagnosticReport::new();
        let error = BasicError {
            message: "Test error".to_string(),
        };

        report.add_error(
            &error,
            Some(DiagnosticLocation {
                file: Some("pkg/main.fol".to_string()),
                line: 7,
                column: 3,
                length: Some(4),
            }),
        );

        let json = report.output(OutputFormat::Json);

        assert!(json.contains("\"message\": \"Test error\""));
        assert!(json.contains("\"file\": \"pkg/main.fol\""));
        assert!(json.contains("\"line\": 7"));
        assert!(json.contains("\"column\": 3"));
        assert!(json.contains("\"length\": 4"));
    }

    #[test]
    fn test_diagnostic_report_human_keeps_location_arrow_shape() {
        let mut report = DiagnosticReport::new();
        let error = BasicError {
            message: "Test error".to_string(),
        };

        report.add_error(
            &error,
            Some(DiagnosticLocation {
                file: Some("pkg/main.fol".to_string()),
                line: 7,
                column: 3,
                length: Some(4),
            }),
        );

        let human = report.output(OutputFormat::Human);

        assert!(human.contains("error: Test error"));
        assert!(human.contains("--> pkg/main.fol:7:3"));
        assert!(human.contains("found 1 error"));
    }

    #[test]
    fn test_diagnostic_rich_model_keeps_primary_location_and_first_help_compatibility() {
        let diagnostic = Diagnostic::new(Severity::Error, "E9000", "rich model")
            .with_primary_label(DiagnosticLocation {
                file: Some("pkg/main.fol".to_string()),
                line: 4,
                column: 2,
                length: Some(3),
            })
            .with_help("first help")
            .with_help("second help")
            .with_note("note text")
            .with_secondary_label(
                DiagnosticLocation {
                    file: Some("pkg/lib.fol".to_string()),
                    line: 8,
                    column: 1,
                    length: Some(5),
                },
                "related declaration",
            );

        assert_eq!(
            diagnostic.primary_location(),
            Some(&DiagnosticLocation {
                file: Some("pkg/main.fol".to_string()),
                line: 4,
                column: 2,
                length: Some(3),
            })
        );
        assert_eq!(diagnostic.first_help(), Some("first help"));
        assert_eq!(diagnostic.labels.len(), 2);
        assert_eq!(diagnostic.notes, vec!["note text".to_string()]);
        assert_eq!(diagnostic.helps.len(), 2);
    }

    #[test]
    fn test_diagnostic_code_is_a_stable_model_type() {
        let diagnostic = Diagnostic::new(
            Severity::Warning,
            DiagnosticCode::new("R1001"),
            "structured code",
        );

        assert_eq!(diagnostic.code.as_str(), "R1001");
        let json = serde_json::to_string(&diagnostic).expect("Diagnostic should serialize");
        assert!(json.contains("\"code\":\"R1001\""));
    }

    #[test]
    fn test_glitch_fallback_no_longer_guesses_codes_from_messages() {
        let error = BasicError {
            message: "ParserMissmatch: legacy text should not drive modern codes".to_string(),
        };
        let diagnostic = Diagnostic::from_glitch(&error, Severity::Error, None);

        assert_eq!(diagnostic.code, DiagnosticCode::unknown());
    }

    #[test]
    fn test_coded_report_helpers_preserve_legacy_location_and_help_views() {
        let mut report = DiagnosticReport::new();
        report.add_coded_error(
            "R2000",
            "explicit code",
            Some(DiagnosticLocation {
                file: Some("pkg/main.fol".to_string()),
                line: 9,
                column: 4,
                length: Some(2),
            }),
        );

        let diagnostic = report
            .diagnostics
            .first()
            .expect("report should contain a coded diagnostic");

        assert_eq!(diagnostic.code.as_str(), "R2000");
        assert_eq!(
            diagnostic.legacy_location(),
            Some(&DiagnosticLocation {
                file: Some("pkg/main.fol".to_string()),
                line: 9,
                column: 4,
                length: Some(2),
            })
        );
        assert_eq!(diagnostic.legacy_help(), None);
    }

    #[test]
    fn test_rich_diagnostic_json_roundtrip_keeps_structured_fields() {
        let diagnostic = Diagnostic::error("R3001", "rich serialization")
            .with_primary_label_message(
                DiagnosticLocation {
                    file: Some("pkg/main.fol".to_string()),
                    line: 5,
                    column: 7,
                    length: Some(4),
                },
                "offending expression",
            )
            .with_secondary_label(
                DiagnosticLocation {
                    file: Some("pkg/lib.fol".to_string()),
                    line: 2,
                    column: 1,
                    length: Some(3),
                },
                "related declaration",
            )
            .with_note("extra note")
            .with_help("first help")
            .with_suggestion(DiagnosticSuggestion {
                message: "replace with `value`".to_string(),
                replacement: Some("value".to_string()),
                location: Some(DiagnosticLocation {
                    file: Some("pkg/main.fol".to_string()),
                    line: 5,
                    column: 7,
                    length: Some(4),
                }),
            });

        let json = serde_json::to_string_pretty(&diagnostic)
            .expect("rich diagnostics should serialize to JSON");

        assert!(json.contains("\"labels\""));
        assert!(json.contains("\"notes\""));
        assert!(json.contains("\"helps\""));
        assert!(json.contains("\"suggestions\""));
        assert!(json.contains("\"location\""));
        assert!(json.contains("\"help\""));

        let decoded: Diagnostic =
            serde_json::from_str(&json).expect("rich diagnostics should deserialize");

        assert_eq!(decoded.code.as_str(), "R3001");
        assert_eq!(decoded.labels.len(), 2);
        assert_eq!(decoded.notes, vec!["extra note".to_string()]);
        assert_eq!(decoded.helps, vec!["first help".to_string()]);
        assert_eq!(decoded.suggestions.len(), 1);
        assert_eq!(decoded.legacy_help(), Some("first help"));
        assert_eq!(
            decoded.primary_label().and_then(|label| label.message.as_deref()),
            Some("offending expression")
        );
    }
}
