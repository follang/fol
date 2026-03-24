//! Marker surface for the heap-enabled, OS-free runtime tier.

pub const HAS_HEAP: bool = true;
pub const HAS_OS: bool = false;

pub fn module_name() -> &'static str {
    "alloc"
}

pub fn tier_name() -> &'static str {
    "alloc"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_tier_marks_heap_without_os() {
        assert_eq!(module_name(), "alloc");
        assert_eq!(tier_name(), "alloc");
        assert!(HAS_HEAP);
        assert!(!HAS_OS);
    }
}
