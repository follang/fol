//! Editor tooling foundations for the FOL language.
//!
//! `fol-editor` will host both the Tree-sitter-facing editor syntax layer and
//! the compiler-backed language-server layer.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Editor;

pub const CRATE_NAME: &str = "fol-editor";

impl Editor {
    pub fn new() -> Self {
        Self
    }
}

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

#[cfg(test)]
mod tests {
    use super::{crate_name, Editor, CRATE_NAME};

    #[test]
    fn crate_name_matches_editor_identity() {
        assert_eq!(crate_name(), CRATE_NAME);
    }

    #[test]
    fn public_editor_shell_is_constructible() {
        assert_eq!(Editor::new(), Editor);
    }
}
