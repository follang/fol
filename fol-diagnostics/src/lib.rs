// FOL Diagnostics - Error formatting and output
use colored::Colorize;

mod model;

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

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Output the report in the specified format
    pub fn output(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => self.to_json(),
            OutputFormat::Human => self.to_human_readable(),
        }
    }

    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    fn to_human_readable(&self) -> String {
        let mut output = String::new();

        for diagnostic in &self.diagnostics {
            output.push_str(&diagnostic.to_human_readable());
            output.push('\n');
        }

        if self.error_count > 0 || self.warning_count > 0 {
            output.push('\n');
            if self.error_count > 0 {
                let label = if self.error_count == 1 { "" } else { "s" };
                output.push_str(&format!(
                    "{} found {} error{}",
                    "error:".red().bold(),
                    self.error_count,
                    label
                ));
            }
            if self.warning_count > 0 {
                if self.error_count > 0 {
                    output.push_str(", ");
                }
                let label = if self.warning_count == 1 { "" } else { "s" };
                output.push_str(&format!(
                    "{} {} warning{}",
                    "warning:".yellow().bold(),
                    self.warning_count,
                    label
                ));
            }
            output.push('\n');
        }

        output
    }
}

impl Default for DiagnosticReport {
    fn default() -> Self {
        Self::new()
    }
}

impl Diagnostic {
    fn to_human_readable(&self) -> String {
        let mut output = String::new();

        let prefix = match self.severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warning".yellow().bold(),
            Severity::Info => "info".blue().bold(),
        };

        output.push_str(&format!("{}: {}", prefix, self.message));

        if let Some(loc) = self.primary_location() {
            output.push('\n');
            if let Some(file) = &loc.file {
                output.push_str(&format!(
                    "  {} {}:{}:{}",
                    "-->".blue().bold(),
                    file,
                    loc.line,
                    loc.column
                ));
            } else {
                output.push_str(&format!(
                    "  {} line {}:{}",
                    "-->".blue().bold(),
                    loc.line,
                    loc.column
                ));
            }
        }

        if let Some(help) = self.first_help() {
            output.push('\n');
            output.push_str(&format!("  {} {}", "help:".green().bold(), help));
        }

        output
    }
}

/// Extract error code from error message for categorization
fn extract_error_code(message: &str) -> String {
    if message.contains("LexerSpaceAdd") {
        "E0001".to_string()
    } else if message.contains("ParserMissmatch") {
        "E0002".to_string()
    } else if message.contains("ReadingBadContent") {
        "E0003".to_string()
    } else if message.contains("GettingNoEntry") {
        "E0004".to_string()
    } else if message.contains("GettingWrongPath") {
        "E0005".to_string()
    } else {
        "E0000".to_string()
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
}
