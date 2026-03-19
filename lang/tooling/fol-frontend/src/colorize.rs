use colored::Colorize;

/// Apply ANSI colors to plain-text diagnostic output from `render_human`.
///
/// The renderer in `fol-diagnostics` produces structured plain text with
/// known prefix markers. This function scans each line and applies colors
/// to those markers so the library crate stays color-free.
pub fn colorize_diagnostics(plain: &str) -> String {
    let mut output = String::with_capacity(plain.len() + 256);
    for line in plain.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("error[") || trimmed.starts_with("error:") {
            output.push_str(&colorize_prefix(line, "error", |s| {
                s.red().bold().to_string()
            }));
        } else if trimmed.starts_with("warning[") || trimmed.starts_with("warning:") {
            output.push_str(&colorize_prefix(line, "warning", |s| {
                s.yellow().bold().to_string()
            }));
        } else if trimmed.starts_with("info[") || trimmed.starts_with("info:") {
            output.push_str(&colorize_prefix(line, "info", |s| {
                s.blue().bold().to_string()
            }));
        } else if trimmed.starts_with("-->") {
            output.push_str(&line.replace("-->", &"-->".blue().bold().to_string()));
        } else if trimmed.starts_with("note:") {
            output.push_str(&line.replacen("note:", &"note:".blue().bold().to_string(), 1));
        } else if trimmed.starts_with("help:") {
            output.push_str(&line.replacen("help:", &"help:".green().bold().to_string(), 1));
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }
    // Remove the trailing newline we added if the original didn't end with one
    if !plain.ends_with('\n') && output.ends_with('\n') {
        output.pop();
    }
    output
}

fn colorize_prefix(line: &str, keyword: &str, style: impl Fn(&str) -> String) -> String {
    if let Some(pos) = line.find(keyword) {
        let (before, rest) = line.split_at(pos);
        // Find the end of the prefix part (keyword + optional [CODE]:)
        let prefix_end = if rest.starts_with(&format!("{keyword}[")) {
            // error[CODE]: or warning[CODE]:
            rest.find("]: ").map(|i| i + 2).unwrap_or(keyword.len())
        } else {
            // error: or warning:
            keyword.len() + 1 // include the colon
        };
        let (prefix, remainder) = rest.split_at(prefix_end);
        format!("{}{}{}", before, style(prefix), remainder)
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorize_adds_ansi_to_error_prefix() {
        let plain = "error[P1001]: unexpected token\n";
        let colored = colorize_diagnostics(plain);
        assert!(colored.contains("\x1b["));
        assert!(colored.contains("unexpected token"));
    }

    #[test]
    fn colorize_preserves_plain_lines() {
        let plain = "  1 | var x: int = 42;\n";
        let colored = colorize_diagnostics(plain);
        assert_eq!(colored, plain);
    }

    #[test]
    fn colorize_handles_help_and_note() {
        let plain = "  note: related\n  help: try this\n";
        let colored = colorize_diagnostics(plain);
        assert!(colored.contains("\x1b["));
        assert!(colored.contains("related"));
        assert!(colored.contains("try this"));
    }
}
