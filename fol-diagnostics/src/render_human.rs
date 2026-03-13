use colored::Colorize;

use crate::{source, Diagnostic, DiagnosticReport, Severity};

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

        if let Some(source_line) = source::load_source_line(loc) {
            let line_number = loc.line.to_string();
            let gutter_width = line_number.len();
            output.push('\n');
            output.push_str(&format!("  {} |", " ".repeat(gutter_width)));
            output.push('\n');
            output.push_str(&format!("  {} | {}", line_number, source_line));
            output.push('\n');
            output.push_str(&format!(
                "  {} | {}",
                " ".repeat(gutter_width),
                source::primary_underline(loc)
            ));
            if let Some(message) = diagnostic
                .primary_label()
                .and_then(|label| label.message.as_deref())
            {
                output.push(' ');
                output.push_str(message);
            }
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
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn render_diagnostic_shows_primary_source_snippet_and_underline() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for test file naming")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("fol_diagnostics_render_{stamp}.fol"));
        std::fs::write(&path, "var answer: int = 42;\n")
            .expect("test source file should be writable");

        let diagnostic = Diagnostic::new(Severity::Error, "R4001", "snippet renderer")
            .with_primary_label_message(
                DiagnosticLocation {
                    file: Some(path.to_string_lossy().into_owned()),
                    line: 1,
                    column: 5,
                    length: Some(6),
                },
                "primary binding",
            );

        let rendered = super::render_diagnostic(&diagnostic);
        let _ = std::fs::remove_file(&path);

        assert!(rendered.contains("var answer: int = 42;"));
        assert!(rendered.contains("^^^^^^ primary binding"));
    }
}
