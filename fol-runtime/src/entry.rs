//! Entrypoint-facing helpers for generated programs.

use crate::{abi::FolRecover, builtins::FolEchoFormat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolProcessOutcome {
    exit_code: i32,
    message: Option<String>,
}

impl FolProcessOutcome {
    pub fn new(exit_code: i32, message: Option<String>) -> Self {
        Self { exit_code, message }
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }

    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

pub fn outcome_from_recoverable<T, E: FolEchoFormat>(value: FolRecover<T, E>) -> FolProcessOutcome {
    match value {
        FolRecover::Ok(_) => FolProcessOutcome::new(0, None),
        FolRecover::Err(error) => FolProcessOutcome::new(1, Some(error.fol_echo_format())),
    }
}

pub fn module_name() -> &'static str {
    "entry"
}

#[cfg(test)]
mod tests {
    use super::{outcome_from_recoverable, FolProcessOutcome};
    use crate::{abi::FolRecover, strings::FolStr};

    #[test]
    fn recoverable_entry_results_map_to_minimal_process_outcomes() {
        let success = outcome_from_recoverable(FolRecover::<i64, FolStr>::ok(7));
        let failure =
            outcome_from_recoverable(FolRecover::<i64, FolStr>::err(FolStr::from("bad-input")));

        assert_eq!(success, FolProcessOutcome::new(0, None));
        assert!(success.is_success());
        assert_eq!(success.message(), None);

        assert_eq!(failure, FolProcessOutcome::new(1, Some("bad-input".to_string())));
        assert!(failure.is_failure());
        assert_eq!(failure.message(), Some("bad-input"));
    }
}
