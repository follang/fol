//! Shared intrinsic registry foundations for the FOL compiler.

mod model;
mod catalog;
mod registry;
mod select;
mod validate;

pub const CRATE_NAME: &str = "fol-intrinsics";

pub use model::{IntrinsicAvailability, IntrinsicCategory, IntrinsicId, IntrinsicStatus, IntrinsicSurface};
pub use catalog::{
    all_intrinsics, intrinsic_by_alias, intrinsic_by_canonical_name, intrinsic_registry,
    intrinsics_for_surface, is_reserved_intrinsic_name_for_surface, reserved_intrinsic_for_surface,
};
pub use registry::{IntrinsicArity, IntrinsicEntry, IntrinsicLoweringMode};
pub use select::{select_intrinsic, IntrinsicSelectionError, IntrinsicSelectionErrorKind};
pub use validate::{validate_intrinsic_registry, RegistryValidationError, RegistryValidationErrorKind};

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

    #[test]
    fn intrinsic_entries_capture_registry_metadata() {
        const ENTRY: IntrinsicEntry = IntrinsicEntry::new(
            IntrinsicId::new(0),
            "eq",
            &["equal"],
            IntrinsicCategory::Comparison,
            IntrinsicSurface::DotRootCall,
            IntrinsicAvailability::V1,
            IntrinsicStatus::Implemented,
            IntrinsicArity::Exactly(2),
            IntrinsicLoweringMode::GeneralIr,
            "compare two values for equality",
        );

        assert_eq!(ENTRY.name, "eq");
        assert_eq!(ENTRY.aliases, &["equal"]);
        assert_eq!(ENTRY.arity, IntrinsicArity::Exactly(2));
        assert_eq!(ENTRY.lowering_mode, IntrinsicLoweringMode::GeneralIr);
    }

    #[test]
    fn canonical_registry_contains_expected_first_batch_and_deferred_entries() {
        let names: Vec<_> = intrinsic_registry().iter().map(|entry| entry.name).collect();

        assert!(names.contains(&"eq"));
        assert!(names.contains(&"not"));
        assert!(names.contains(&"len"));
        assert!(names.contains(&"echo"));
        assert!(names.contains(&"de_alloc"));
        assert!(names.contains(&"pointer_value"));
    }

    #[test]
    fn lookup_apis_find_intrinsics_by_name_alias_and_surface() {
        let eq = intrinsic_by_canonical_name("eq").expect("eq should exist");
        let nq = intrinsic_by_alias("ne").expect("ne alias should exist");
        let dot_calls = intrinsics_for_surface(IntrinsicSurface::DotRootCall);

        assert_eq!(eq.name, "eq");
        assert_eq!(nq.name, "nq");
        assert!(dot_calls.iter().any(|entry| entry.name == "len"));
    }

    #[test]
    fn parser_facing_helpers_identify_reserved_names_by_surface() {
        let len = reserved_intrinsic_for_surface(IntrinsicSurface::DotRootCall, "len")
            .expect("len should be reserved for dot-root intrinsics");

        assert_eq!(len.name, "len");
        assert!(is_reserved_intrinsic_name_for_surface(
            IntrinsicSurface::KeywordCall,
            "panic"
        ));
        assert!(!is_reserved_intrinsic_name_for_surface(
            IntrinsicSurface::DotRootCall,
            "user_helper"
        ));
    }

    #[test]
    fn selection_api_distinguishes_unknown_names_from_surface_mismatches() {
        let eq = select_intrinsic(IntrinsicSurface::DotRootCall, "eq")
            .expect("eq should select on the dot-root surface");
        let wrong_surface = select_intrinsic(IntrinsicSurface::DotRootCall, "panic")
            .expect_err("panic should be keyword-only");
        let unknown = select_intrinsic(IntrinsicSurface::DotRootCall, "user_helper")
            .expect_err("unknown helpers should stay unknown");

        assert_eq!(eq.name, "eq");
        assert_eq!(wrong_surface.kind, IntrinsicSelectionErrorKind::WrongSurface);
        assert_eq!(wrong_surface.name, "panic");
        assert_eq!(unknown.kind, IntrinsicSelectionErrorKind::UnknownName);
    }

    #[test]
    fn registry_validation_accepts_the_canonical_registry() {
        assert!(validate_intrinsic_registry(intrinsic_registry()).is_ok());
    }
}
