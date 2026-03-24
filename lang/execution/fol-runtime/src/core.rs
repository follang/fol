//! Marker surface for the no-heap, no-OS runtime tier.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeTier {
    pub name: &'static str,
    pub has_heap: bool,
    pub has_os: bool,
}

impl RuntimeTier {
    pub const fn new(name: &'static str, has_heap: bool, has_os: bool) -> Self {
        Self {
            name,
            has_heap,
            has_os,
        }
    }
}

pub const HAS_HEAP: bool = false;
pub const HAS_OS: bool = false;
pub const TIER: RuntimeTier = RuntimeTier::new("core", HAS_HEAP, HAS_OS);

pub fn module_name() -> &'static str {
    "core"
}

pub fn tier_name() -> &'static str {
    TIER.name
}

pub fn capabilities() -> RuntimeTier {
    TIER
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
        assert_eq!(capabilities(), TIER);
    }
}
