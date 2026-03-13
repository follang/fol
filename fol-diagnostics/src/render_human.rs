use colored::Colorize;

use crate::{Diagnostic, DiagnosticReport, Severity};

pub fn render_report(report: &DiagnosticReport) -> String {
    let mut output = String::new();

    for diagnostic in &report.diagnostics {
        output.push_str(&render_diagnostic(diagnostic));
        output.push('\n');
    }

    if report.error_count > 0 || report.warning_count > 0 {
        output.push('\n');
        if report.error_count > 0 {
            let label = if report.error_count == 1 { "" } else { "s" };
            output.push_str(&format!(
                "{} found {} error{}",
                "error:".red().bold(),
                report.error_count,
                label
            ));
        }
        if report.warning_count > 0 {
            if report.error_count > 0 {
                output.push_str(", ");
            }
            let label = if report.warning_count == 1 { "" } else { "s" };
            output.push_str(&format!(
                "{} {} warning{}",
                "warning:".yellow().bold(),
                report.warning_count,
                label
            ));
        }
        output.push('\n');
    }

    output
}

pub fn render_diagnostic(diagnostic: &Diagnostic) -> String {
    let mut output = String::new();

    let prefix = match diagnostic.severity {
        Severity::Error => "error".red().bold(),
        Severity::Warning => "warning".yellow().bold(),
        Severity::Info => "info".blue().bold(),
    };

    output.push_str(&format!("{}: {}", prefix, diagnostic.message));

    if let Some(loc) = diagnostic.primary_location() {
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

    if let Some(help) = diagnostic.first_help() {
        output.push('\n');
        output.push_str(&format!("  {} {}", "help:".green().bold(), help));
    }

    output
}

#[cfg(test)]
mod tests {
    use crate::{Diagnostic, DiagnosticLocation, Severity};

    #[test]
    fn render_diagnostic_uses_primary_location_and_first_help() {
        let diagnostic = Diagnostic::new(Severity::Error, "R4000", "renderer split")
            .with_primary_label(DiagnosticLocation {
                file: Some("pkg/main.fol".to_string()),
                line: 3,
                column: 2,
                length: Some(5),
            })
            .with_help("use this renderer");

        let rendered = super::render_diagnostic(&diagnostic);

        assert!(rendered.contains("renderer split"));
        assert!(rendered.contains("--> pkg/main.fol:3:2"));
        assert!(rendered.contains("help: use this renderer"));
    }
}
