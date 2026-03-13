// FOL Diagnostics - Error formatting and output
use colored::Colorize;
use fol_types::Glitch;
use serde::{Deserialize, Serialize};

/// Output format for diagnostics
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Location information for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticLocation {
    pub file: Option<String>,
    pub line: usize,
    pub column: usize,
    pub length: Option<usize>,
}

/// Severity levels for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A single diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub location: Option<DiagnosticLocation>,
    pub help: Option<String>,
}

/// Collection of diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub diagnostics: Vec<Diagnostic>,
    pub error_count: usize,
    pub warning_count: usize,
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

    pub fn add_error(&mut self, error: &dyn Glitch, location: Option<DiagnosticLocation>) {
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

        // Summary
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
    pub fn from_glitch(
        error: &dyn Glitch,
        severity: Severity,
        location: Option<DiagnosticLocation>,
    ) -> Self {
        let error_msg = error.to_string();
        let code = extract_error_code(&error_msg);

        Self {
            severity,
            code,
            message: error_msg,
            location,
            help: None,
        }
    }

    fn to_human_readable(&self) -> String {
        let mut output = String::new();

        // Error prefix with severity
        let prefix = match self.severity {
            Severity::Error => "error".red().bold(),
            Severity::Warning => "warning".yellow().bold(),
            Severity::Info => "info".blue().bold(),
        };

        output.push_str(&format!("{}: {}", prefix, self.message));

        // Location information
        if let Some(loc) = &self.location {
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

        // Help text
        if let Some(help) = &self.help {
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
        "E0000".to_string() // Unknown error
    }
}

/// Helper trait to convert locations to diagnostic locations
pub trait ToDiagnosticLocation {
    fn to_diagnostic_location(&self, file: Option<String>) -> DiagnosticLocation;
}

impl DiagnosticLocation {
    /// Create from fol-lexer point::Location (if available)
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
}
