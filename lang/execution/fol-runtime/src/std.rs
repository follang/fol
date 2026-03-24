//! Marker surface for the hosted runtime tier.

use crate::{alloc, core::RuntimeTier, core};

pub const HAS_HEAP: bool = true;
pub const HAS_OS: bool = true;
pub const TIER: RuntimeTier = RuntimeTier::new("std", HAS_HEAP, HAS_OS);

pub fn module_name() -> &'static str {
    "std"
}

pub fn tier_name() -> &'static str {
    TIER.name
}

pub fn base_core_tier() -> RuntimeTier {
    core::capabilities()
}

pub fn base_alloc_tier() -> RuntimeTier {
    alloc::capabilities()
}

pub fn capabilities() -> RuntimeTier {
    TIER
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
        assert_eq!(capabilities(), TIER);
    }

    #[test]
    fn std_tier_builds_on_core_and_alloc_tiers() {
        assert_eq!(base_core_tier(), core::TIER);
        assert_eq!(base_alloc_tier(), alloc::TIER);
        assert!(base_alloc_tier().has_heap);
        assert!(capabilities().has_heap);
        assert!(capabilities().has_os);
    }
}
