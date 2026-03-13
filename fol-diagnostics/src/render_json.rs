use crate::DiagnosticReport;

pub fn render_report(report: &DiagnosticReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use crate::{
        Diagnostic, DiagnosticLocation, DiagnosticReport, DiagnosticSuggestion, Severity,
    };
    use serde_json::Value;

    #[test]
    fn render_report_serializes_diagnostic_reports() {
        let mut report = DiagnosticReport::new();
        report.add_diagnostic(
            Diagnostic::new(Severity::Error, "R5000", "json renderer").with_primary_label(
                DiagnosticLocation {
                    file: Some("pkg/main.fol".to_string()),
                    line: 1,
                    column: 1,
                    length: Some(3),
                },
            ),
        );

        let rendered = super::render_report(&report);

        assert!(rendered.contains("\"diagnostics\""));
        assert!(rendered.contains("\"message\": \"json renderer\""));
        assert!(rendered.contains("\"file\": \"pkg/main.fol\""));
    }

    #[test]
    fn render_report_keeps_structured_diagnostic_fields_stable() {
        let mut report = DiagnosticReport::new();
        report.add_diagnostic(
            Diagnostic::new(Severity::Warning, "R5001", "structured json")
                .with_primary_label(DiagnosticLocation {
                    file: Some("pkg/main.fol".to_string()),
                    line: 4,
                    column: 2,
                    length: Some(5),
                })
                .with_secondary_label(
                    DiagnosticLocation {
                        file: Some("pkg/lib.fol".to_string()),
                        line: 2,
                        column: 1,
                        length: Some(3),
                    },
                    "related declaration",
                )
                .with_note("shadowed here")
                .with_help("rename the local binding")
                .with_suggestion(DiagnosticSuggestion {
                    message: "replace the local name".to_string(),
                    replacement: Some("other_name".to_string()),
                    location: Some(DiagnosticLocation {
                        file: Some("pkg/main.fol".to_string()),
                        line: 4,
                        column: 2,
                        length: Some(5),
                    }),
                }),
        );

        let rendered = super::render_report(&report);
        let json: Value = serde_json::from_str(&rendered).expect("JSON renderer should stay valid");
        let diagnostic = &json["diagnostics"][0];

        assert_eq!(json["error_count"], 0);
        assert_eq!(json["warning_count"], 1);
        assert_eq!(diagnostic["code"], "R5001");
        assert!(diagnostic["location"].is_object());
        assert!(diagnostic["help"].is_string());
        assert!(diagnostic["labels"].is_array());
        assert!(diagnostic["notes"].is_array());
        assert!(diagnostic["helps"].is_array());
        assert!(diagnostic["suggestions"].is_array());
        assert_eq!(diagnostic["labels"].as_array().map(|items| items.len()), Some(2));
        assert_eq!(diagnostic["notes"][0], "shadowed here");
        assert_eq!(diagnostic["helps"][0], "rename the local binding");
        assert_eq!(diagnostic["suggestions"][0]["replacement"], "other_name");
    }

    #[test]
    fn render_report_serializes_warning_and_info_severities() {
        let mut report = DiagnosticReport::new();
        report.add_diagnostic(Diagnostic::warning("W5002", "json warning"));
        report.add_diagnostic(Diagnostic::info("I5003", "json info"));

        let rendered = super::render_report(&report);
        let json: Value = serde_json::from_str(&rendered).expect("JSON renderer should stay valid");

        assert_eq!(json["warning_count"], 1);
        assert_eq!(json["diagnostics"][0]["severity"], "Warning");
        assert_eq!(json["diagnostics"][1]["severity"], "Info");
    }
}
