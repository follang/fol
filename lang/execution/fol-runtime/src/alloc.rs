//! Marker surface for the heap-enabled, OS-free runtime tier.

use crate::core::{self, RuntimeTier};

pub const HAS_HEAP: bool = true;
pub const HAS_OS: bool = false;
pub const TIER: RuntimeTier = RuntimeTier::new("alloc", HAS_HEAP, HAS_OS);

pub fn module_name() -> &'static str {
    "alloc"
}

pub fn tier_name() -> &'static str {
    TIER.name
}

pub fn base_tier() -> RuntimeTier {
    core::capabilities()
}

pub fn capabilities() -> RuntimeTier {
    TIER
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
        assert_eq!(capabilities(), TIER);
    }

    #[test]
    fn alloc_tier_builds_on_core_tier() {
        assert_eq!(base_tier(), core::TIER);
        assert!(!base_tier().has_heap);
        assert!(capabilities().has_heap);
        assert_eq!(capabilities().has_os, base_tier().has_os);
    }
}
