//! Hosted runtime tier surface, including console-facing formatting hooks.

use crate::{
    abi::FolRecover,
    alloc::FolStr,
    containers::{FolArray, FolMap, FolSeq, FolSet, FolVec},
    core::RuntimeTier,
    shell::{FolError, FolOption},
};

pub use crate::{crate_name, CRATE_NAME};

pub const HAS_HEAP: bool = true;
pub const HAS_OS: bool = true;
pub const TIER: RuntimeTier = RuntimeTier::new("std", HAS_HEAP, HAS_OS);

pub trait FolEchoFormat {
    fn fol_echo_format(&self) -> String;
}

fn join_echo<I>(items: I) -> String
where
    I: IntoIterator<Item = String>,
{
    items.into_iter().collect::<Vec<_>>().join(", ")
}

pub fn render_echo<T: FolEchoFormat + ?Sized>(value: &T) -> String {
    value.fol_echo_format()
}

pub fn echo<T: FolEchoFormat>(value: T) -> T {
    println!("{}", value.fol_echo_format());
    value
}

pub const FOL_EXIT_SUCCESS: i32 = 0;
pub const FOL_EXIT_FAILURE: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolProcessOutcome {
    exit_code: i32,
    message: Option<String>,
}

impl FolProcessOutcome {
    pub fn new(exit_code: i32, message: Option<String>) -> Self {
        Self { exit_code, message }
    }

    pub fn success() -> Self {
        Self::new(FOL_EXIT_SUCCESS, None)
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self::new(FOL_EXIT_FAILURE, Some(message.into()))
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn is_success(&self) -> bool {
        self.exit_code == FOL_EXIT_SUCCESS
    }

    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

pub fn failure_outcome_from_error<E: FolEchoFormat>(error: E) -> FolProcessOutcome {
    FolProcessOutcome::failure(error.fol_echo_format())
}

pub fn printable_outcome_message(outcome: &FolProcessOutcome) -> Option<&str> {
    outcome.message()
}

pub fn outcome_from_recoverable<T, E: FolEchoFormat>(value: FolRecover<T, E>) -> FolProcessOutcome {
    match value {
        FolRecover::Ok(_) => FolProcessOutcome::success(),
        FolRecover::Err(error) => failure_outcome_from_error(error),
    }
}

impl FolEchoFormat for i64 {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl FolEchoFormat for f64 {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl FolEchoFormat for bool {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl FolEchoFormat for char {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl FolEchoFormat for FolStr {
    fn fol_echo_format(&self) -> String {
        self.to_string()
    }
}

impl<T: FolEchoFormat, const N: usize> FolEchoFormat for FolArray<T, N> {
    fn fol_echo_format(&self) -> String {
        format!("arr[{}]", join_echo(self.iter().map(render_echo)))
    }
}

impl<T: FolEchoFormat> FolEchoFormat for FolVec<T> {
    fn fol_echo_format(&self) -> String {
        format!("vec[{}]", join_echo(self.as_slice().iter().map(render_echo)))
    }
}

impl<T: FolEchoFormat> FolEchoFormat for FolSeq<T> {
    fn fol_echo_format(&self) -> String {
        format!("seq[{}]", join_echo(self.as_slice().iter().map(render_echo)))
    }
}

impl<T: FolEchoFormat + Ord> FolEchoFormat for FolSet<T> {
    fn fol_echo_format(&self) -> String {
        format!("set{{{}}}", join_echo(self.as_set().iter().map(render_echo)))
    }
}

impl<K: FolEchoFormat + Ord, V: FolEchoFormat> FolEchoFormat for FolMap<K, V> {
    fn fol_echo_format(&self) -> String {
        format!(
            "map{{{}}}",
            join_echo(self.as_map().iter().map(|(key, value)| format!(
                "{}: {}",
                render_echo(key),
                render_echo(value)
            )))
        )
    }
}

impl<T: FolEchoFormat> FolEchoFormat for FolOption<T> {
    fn fol_echo_format(&self) -> String {
        match self {
            FolOption::Some(value) => format!("some({})", render_echo(value)),
            FolOption::Nil => "nil".to_string(),
        }
    }
}

impl<T: FolEchoFormat> FolEchoFormat for FolError<T> {
    fn fol_echo_format(&self) -> String {
        format!("err({})", render_echo(self.as_ref()))
    }
}

pub fn module_name() -> &'static str {
    "std"
}

pub fn tier_name() -> &'static str {
    TIER.name
}

pub fn base_core_tier() -> RuntimeTier {
    crate::core::capabilities()
}

pub fn base_alloc_tier() -> RuntimeTier {
    crate::alloc::capabilities()
}

pub fn capabilities() -> RuntimeTier {
    TIER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DemoEcho(&'static str);

    impl FolEchoFormat for DemoEcho {
        fn fol_echo_format(&self) -> String {
            format!("demo({})", self.0)
        }
    }

    #[test]
    fn std_tier_marks_heap_and_os() {
        assert_eq!(module_name(), "std");
        assert_eq!(tier_name(), "std");
        assert!(HAS_HEAP);
        assert!(HAS_OS);
        assert_eq!(capabilities(), TIER);
    }

    #[test]
    fn std_tier_builds_on_core_and_alloc_tiers() {
        assert_eq!(base_core_tier(), crate::core::TIER);
        assert_eq!(base_alloc_tier(), crate::alloc::TIER);
        assert!(base_alloc_tier().has_heap);
        assert!(capabilities().has_heap);
        assert!(capabilities().has_os);
    }

    #[test]
    fn runtime_echo_trait_and_helpers_freeze_backend_hook_boundary() {
        let value = DemoEcho("trace");

        assert_eq!(render_echo(&value), "demo(trace)");
        assert_eq!(echo(value.clone()), value);
    }

    #[test]
    fn runtime_echo_formats_builtin_scalars_and_strings() {
        let text = FolStr::from("Ada");

        assert_eq!(render_echo(&7i64), "7");
        assert_eq!(render_echo(&3.5f64), "3.5");
        assert_eq!(render_echo(&true), "true");
        assert_eq!(render_echo(&'x'), "x");
        assert_eq!(render_echo(&text), "Ada");
        assert_eq!(echo(text.clone()), text);
    }

    #[test]
    fn runtime_echo_formats_current_v1_container_families() {
        let array: FolArray<i64, 3> = [1, 2, 3];
        let vector = FolVec::from_items(vec![1, 2, 3]);
        let sequence = FolSeq::from_items(vec![1, 2, 3]);
        let set = FolSet::from_items(vec![3, 1, 2]);
        let map = FolMap::from_pairs(vec![(FolStr::from("lin"), 2), (FolStr::from("ada"), 1)]);

        assert_eq!(render_echo(&array), "arr[1, 2, 3]");
        assert_eq!(render_echo(&vector), "vec[1, 2, 3]");
        assert_eq!(render_echo(&sequence), "seq[1, 2, 3]");
        assert_eq!(render_echo(&set), "set{1, 2, 3}");
        assert_eq!(render_echo(&map), "map{ada: 1, lin: 2}");
    }

    #[test]
    fn runtime_echo_formats_current_v1_shell_families() {
        let some = FolOption::some(FolStr::from("Ada"));
        let nil = FolOption::<FolStr>::nil();
        let error = FolError::new(FolStr::from("broken"));

        assert_eq!(render_echo(&some), "some(Ada)");
        assert_eq!(render_echo(&nil), "nil");
        assert_eq!(render_echo(&error), "err(broken)");
    }

    #[test]
    fn runtime_echo_formats_nested_v1_values_stably() {
        let nested_seq =
            FolSeq::from_items(vec![FolOption::some(FolStr::from("Ada")), FolOption::nil()]);
        let nested_map = FolMap::from_pairs(vec![
            (
                FolStr::from("left"),
                FolError::new(FolSeq::from_items(vec![1i64, 2, 3])),
            ),
            (
                FolStr::from("right"),
                FolError::new(FolSeq::from_items(vec![4i64, 5])),
            ),
        ]);

        assert_eq!(render_echo(&nested_seq), "seq[some(Ada), nil]");
        assert_eq!(
            render_echo(&nested_map),
            "map{left: err(seq[1, 2, 3]), right: err(seq[4, 5])}"
        );
    }

    #[test]
    fn runtime_echo_formats_nested_container_values_stably() {
        let nested_seq = FolSeq::from_items(vec![
            FolSeq::from_items(vec![1i64, 2]),
            FolSeq::from_items(vec![3i64]),
        ]);
        let nested_map = FolMap::from_pairs(vec![
            (FolStr::from("left"), FolSet::from_items(vec![3i64, 1, 2])),
            (FolStr::from("right"), FolSet::from_items(vec![5i64, 4])),
        ]);

        assert_eq!(render_echo(&nested_seq), "seq[seq[1, 2], seq[3]]");
        assert_eq!(
            render_echo(&nested_map),
            "map{left: set{1, 2, 3}, right: set{4, 5}}"
        );
    }

    #[test]
    fn recoverable_entry_results_map_to_minimal_process_outcomes() {
        let success = outcome_from_recoverable(FolRecover::<i64, FolStr>::ok(7));
        let failure =
            outcome_from_recoverable(FolRecover::<i64, FolStr>::err(FolStr::from("bad-input")));

        assert_eq!(success, FolProcessOutcome::success());
        assert!(success.is_success());
        assert_eq!(success.message(), None);

        assert_eq!(failure, FolProcessOutcome::failure("bad-input"));
        assert!(failure.is_failure());
        assert_eq!(failure.message(), Some("bad-input"));
    }

    #[test]
    fn failure_helpers_keep_printable_messages_stable() {
        let failure = failure_outcome_from_error(FolStr::from("broken"));

        assert_eq!(failure, FolProcessOutcome::failure("broken"));
        assert_eq!(printable_outcome_message(&failure), Some("broken"));
        assert_eq!(
            printable_outcome_message(&FolProcessOutcome::success()),
            None
        );
    }

    #[test]
    fn exit_code_constants_freeze_minimal_v1_process_policy() {
        assert_eq!(FOL_EXIT_SUCCESS, 0);
        assert_eq!(FOL_EXIT_FAILURE, 1);
        assert_eq!(FolProcessOutcome::success().exit_code(), FOL_EXIT_SUCCESS);
        assert_eq!(
            FolProcessOutcome::failure("broken").exit_code(),
            FOL_EXIT_FAILURE
        );
    }

    #[test]
    fn top_level_success_and_failure_messages_stay_backend_ready() {
        let success = outcome_from_recoverable(FolRecover::<i64, FolStr>::ok(9));
        let failure =
            outcome_from_recoverable(FolRecover::<i64, FolStr>::err(FolStr::from("fatal")));

        assert!(success.is_success());
        assert_eq!(success.exit_code(), FOL_EXIT_SUCCESS);
        assert_eq!(printable_outcome_message(&success), None);

        assert!(failure.is_failure());
        assert_eq!(failure.exit_code(), FOL_EXIT_FAILURE);
        assert_eq!(printable_outcome_message(&failure), Some("fatal"));
    }
}
