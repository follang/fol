//! Runtime support foundations for executable FOL V1 programs.

pub mod abi;
pub mod builtins;
pub mod containers;
pub mod entry;
pub mod error;
pub mod prelude;
pub mod shell;
pub mod strings;
pub mod value;

pub const CRATE_NAME: &str = "fol-runtime";

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

pub use error::{RuntimeError, RuntimeErrorKind};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_matches_expected_runtime_identity() {
        assert_eq!(crate_name(), "fol-runtime");
    }

    #[test]
    fn public_runtime_module_shell_is_importable() {
        assert_eq!(abi::module_name(), "abi");
        assert_eq!(builtins::module_name(), "builtins");
        assert_eq!(containers::module_name(), "containers");
        assert_eq!(entry::module_name(), "entry");
        assert_eq!(error::module_name(), "error");
        assert_eq!(shell::module_name(), "shell");
        assert_eq!(strings::module_name(), "strings");
        assert_eq!(value::module_name(), "value");
        assert_eq!(prelude::crate_name(), "fol-runtime");
    }

    #[test]
    fn runtime_errors_can_be_constructed_with_stable_kinds() {
        let error = RuntimeError::new(
            RuntimeErrorKind::InvariantViolation,
            "runtime invariant failed",
        );

        assert_eq!(error.kind(), RuntimeErrorKind::InvariantViolation);
        assert_eq!(error.message(), "runtime invariant failed");
    }
}
