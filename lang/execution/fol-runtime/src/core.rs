//! Marker surface for the no-heap, no-OS runtime tier.

pub const HAS_HEAP: bool = false;
pub const HAS_OS: bool = false;

pub fn module_name() -> &'static str {
    "core"
}

pub fn tier_name() -> &'static str {
    "core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_tier_marks_no_heap_and_no_os() {
        assert_eq!(module_name(), "core");
        assert_eq!(tier_name(), "core");
        assert!(!HAS_HEAP);
        assert!(!HAS_OS);
    }
}
