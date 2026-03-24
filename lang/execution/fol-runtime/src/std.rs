//! Marker surface for the hosted runtime tier.

pub const HAS_HEAP: bool = true;
pub const HAS_OS: bool = true;

pub fn module_name() -> &'static str {
    "std"
}

pub fn tier_name() -> &'static str {
    "std"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_tier_marks_heap_and_os() {
        assert_eq!(module_name(), "std");
        assert_eq!(tier_name(), "std");
        assert!(HAS_HEAP);
        assert!(HAS_OS);
    }
}
