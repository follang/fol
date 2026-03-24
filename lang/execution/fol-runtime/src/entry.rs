//! Compatibility-facing entrypoint module shell.
//!
//! Ownership for hosted process outcomes now lives in [`crate::std`]. This
//! module remains only as the legacy path until the old entry surface is
//! deleted in a later cleanup slice.

pub use crate::std::{
    failure_outcome_from_error, outcome_from_recoverable, printable_outcome_message,
    FolProcessOutcome, FOL_EXIT_FAILURE, FOL_EXIT_SUCCESS,
};

pub fn module_name() -> &'static str {
    "entry"
}

#[cfg(test)]
mod tests {
    use super::{
        failure_outcome_from_error, outcome_from_recoverable, printable_outcome_message,
        FolProcessOutcome, FOL_EXIT_FAILURE, FOL_EXIT_SUCCESS,
    };
    use crate::{abi::FolRecover, strings::FolStr};

    #[test]
    fn entry_module_reexports_std_owned_process_outcome_helpers() {
        let success = outcome_from_recoverable(FolRecover::<i64, FolStr>::ok(7));
        let failure = failure_outcome_from_error(FolStr::from("broken"));

        assert_eq!(module_name(), "entry");
        assert_eq!(success, FolProcessOutcome::success());
        assert_eq!(printable_outcome_message(&failure), Some("broken"));
        assert_eq!(FOL_EXIT_SUCCESS, 0);
        assert_eq!(FOL_EXIT_FAILURE, 1);
    }
}
