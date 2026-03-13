use crate::DiagnosticReport;

pub fn render_report(report: &DiagnosticReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use crate::{Diagnostic, DiagnosticLocation, DiagnosticReport, Severity};

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
}
