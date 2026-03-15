//! Runtime support foundations for executable FOL `V1` programs.
//!
//! `fol-runtime` is the support crate that future generated programs will link
//! against. It is not a front-end phase and it is not the backend itself.
//!
//! The intended compiler split is:
//!
//! - `fol-runtime` owns runtime data/layout/helper semantics
//! - `fol-backend` will later own code generation
//!
//! Current `V1` runtime scope:
//!
//! - builtin scalar support
//! - runtime strings
//! - runtime containers
//! - optional/error shells
//! - recoverable routine results
//! - backend-facing runtime hooks such as `.echo(...)`
//!
//! Explicitly out of scope for this milestone:
//!
//! - ownership / borrowing / pointers
//! - standards / generics
//! - concurrency runtime
//! - C ABI
//! - `core` / `std`

pub mod abi;
pub mod aggregate;
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
        assert_eq!(aggregate::module_name(), "aggregate");
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

    #[test]
    fn public_recoverable_abi_freezes_success_path_through_prelude() {
        let value = prelude::FolRecover::<prelude::FolInt, prelude::FolStr>::ok(7);

        assert!(!prelude::check_recoverable(&value));
        assert!(prelude::recoverable_succeeded(&value));
        assert_eq!(value.value_ref(), Some(&7));
        assert_eq!(Result::<prelude::FolInt, prelude::FolStr>::from(value), Ok(7));
    }

    #[test]
    fn public_recoverable_abi_freezes_failure_path_through_prelude() {
        let value = prelude::FolRecover::<prelude::FolInt, prelude::FolStr>::err(
            prelude::FolStr::from("bad-input"),
        );

        assert!(prelude::check_recoverable(&value));
        assert!(!prelude::recoverable_succeeded(&value));
        assert_eq!(value.error_ref().map(|error| error.as_str()), Some("bad-input"));
        assert_eq!(
            Result::<prelude::FolInt, prelude::FolStr>::from(value),
            Err(prelude::FolStr::from("bad-input"))
        );
    }

    #[test]
    fn public_shell_values_stay_distinct_from_recoverable_results() {
        let optional = prelude::FolOption::some(7);
        let error_shell = prelude::FolError::new(prelude::FolStr::from("broken"));
        let recoverable = prelude::FolRecover::<prelude::FolInt, prelude::FolStr>::err(
            prelude::FolStr::from("broken"),
        );

        assert_eq!(
            std::any::type_name::<prelude::FolOption<prelude::FolInt>>(),
            "fol_runtime::shell::FolOption<i64>"
        );
        assert_eq!(
            std::any::type_name::<prelude::FolError<prelude::FolStr>>(),
            "fol_runtime::shell::FolError<fol_runtime::strings::FolStr>"
        );
        assert_eq!(
            std::any::type_name::<prelude::FolRecover<prelude::FolInt, prelude::FolStr>>(),
            "fol_runtime::abi::FolRecover<i64, fol_runtime::strings::FolStr>"
        );

        assert_eq!(prelude::unwrap_optional_shell(optional), Ok(7));
        assert_eq!(
            prelude::unwrap_error_shell(error_shell),
            prelude::FolStr::from("broken")
        );
        assert!(prelude::check_recoverable(&recoverable));
    }
}
