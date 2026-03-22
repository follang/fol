//! Minimal ANSI styling helpers — replaces the `colored` crate.
//!
//! Colors are automatically disabled when stdout is not a terminal.
//! Call [`set_enabled`] to override.

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

static ENABLED: AtomicBool = AtomicBool::new(false);
static INIT: Once = Once::new();

fn init() {
    INIT.call_once(|| {
        ENABLED.store(std::io::stdout().is_terminal(), Ordering::Relaxed);
    });
}

pub fn set_enabled(on: bool) {
    init();
    ENABLED.store(on, Ordering::Relaxed);
}

fn enabled() -> bool {
    init();
    ENABLED.load(Ordering::Relaxed)
}

fn wrap2(s: &str, a: &str, b: &str) -> String {
    if enabled() {
        format!("\x1b[{a};{b}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn cyan(s: &str) -> String {
    if enabled() {
        format!("\x1b[36m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn bold_red(s: &str) -> String {
    wrap2(s, "1", "31")
}

pub fn bold_green(s: &str) -> String {
    wrap2(s, "1", "32")
}

pub fn bold_yellow(s: &str) -> String {
    wrap2(s, "1", "33")
}

pub fn bold_blue(s: &str) -> String {
    wrap2(s, "1", "34")
}

pub fn bold_cyan(s: &str) -> String {
    wrap2(s, "1", "36")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_helpers_emit_escape_sequences_when_enabled() {
        set_enabled(true);
        assert_eq!(cyan("hi"), "\x1b[36mhi\x1b[0m");
        assert_eq!(bold_red("err"), "\x1b[1;31merr\x1b[0m");
    }

    #[test]
    fn ansi_helpers_pass_through_when_disabled() {
        set_enabled(false);
        assert_eq!(cyan("hi"), "hi");
        assert_eq!(bold_red("err"), "err");
        set_enabled(true);
    }
}
