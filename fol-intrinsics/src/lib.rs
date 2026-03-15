//! Shared intrinsic registry foundations for the FOL compiler.

mod model;

pub const CRATE_NAME: &str = "fol-intrinsics";

pub use model::{IntrinsicAvailability, IntrinsicCategory, IntrinsicId, IntrinsicStatus, IntrinsicSurface};

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_matches_expected_foundation_identity() {
        assert_eq!(crate_name(), "fol-intrinsics");
    }

    #[test]
    fn public_model_types_cover_basic_intrinsic_dimensions() {
        assert_eq!(IntrinsicId::new(3).index(), 3);
        assert_eq!(IntrinsicCategory::Comparison.as_str(), "comparison");
        assert_eq!(IntrinsicSurface::DotRootCall.as_str(), "dot-root-call");
        assert_eq!(IntrinsicAvailability::V1.as_str(), "V1");
        assert_eq!(IntrinsicStatus::Implemented.as_str(), "implemented");
    }
}
