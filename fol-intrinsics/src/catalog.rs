use crate::{
    IntrinsicArity, IntrinsicAvailability, IntrinsicCategory, IntrinsicEntry, IntrinsicId,
    IntrinsicLoweringMode, IntrinsicStatus, IntrinsicSurface,
};

const INTRINSICS: &[IntrinsicEntry] = &[
    IntrinsicEntry::new(
        IntrinsicId::new(0),
        "eq",
        &[],
        IntrinsicCategory::Comparison,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(2),
        IntrinsicLoweringMode::GeneralIr,
        "compare two values for equality",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(1),
        "nq",
        &["ne"],
        IntrinsicCategory::Comparison,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(2),
        IntrinsicLoweringMode::GeneralIr,
        "compare two values for inequality",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(2),
        "lt",
        &[],
        IntrinsicCategory::Comparison,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(2),
        IntrinsicLoweringMode::GeneralIr,
        "compare whether the left value is less than the right value",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(3),
        "gt",
        &[],
        IntrinsicCategory::Comparison,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(2),
        IntrinsicLoweringMode::GeneralIr,
        "compare whether the left value is greater than the right value",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(4),
        "ge",
        &[],
        IntrinsicCategory::Comparison,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(2),
        IntrinsicLoweringMode::GeneralIr,
        "compare whether the left value is greater than or equal to the right value",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(5),
        "le",
        &[],
        IntrinsicCategory::Comparison,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(2),
        IntrinsicLoweringMode::GeneralIr,
        "compare whether the left value is less than or equal to the right value",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(6),
        "not",
        &[],
        IntrinsicCategory::Boolean,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::GeneralIr,
        "negate a boolean value",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(7),
        "len",
        &[],
        IntrinsicCategory::Query,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::DedicatedIr,
        "query the length of a supported value",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(8),
        "echo",
        &["print"],
        IntrinsicCategory::Diagnostic,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::RuntimeHook,
        "emit a runtime-visible debug value and forward it unchanged",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(9),
        "cast",
        &[],
        IntrinsicCategory::Conversion,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::Exactly(2),
        IntrinsicLoweringMode::Reject,
        "perform an explicit conversion once the V1 conversion contract is frozen",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(10),
        "as",
        &[],
        IntrinsicCategory::Conversion,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::Exactly(2),
        IntrinsicLoweringMode::Reject,
        "perform an explicit conversion once the V1 conversion contract is frozen",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(11),
        "assert",
        &[],
        IntrinsicCategory::Diagnostic,
        IntrinsicSurface::KeywordCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::AtLeast(1),
        IntrinsicLoweringMode::Reject,
        "assert a condition once the V1 assert contract is frozen",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(12),
        "check",
        &[],
        IntrinsicCategory::Recoverable,
        IntrinsicSurface::KeywordCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::DedicatedIr,
        "inspect whether a recoverable routine call failed",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(13),
        "panic",
        &[],
        IntrinsicCategory::Diagnostic,
        IntrinsicSurface::KeywordCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Implemented,
        IntrinsicArity::AtLeast(0),
        IntrinsicLoweringMode::DedicatedIr,
        "abort control flow immediately",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(14),
        "cap",
        &[],
        IntrinsicCategory::Query,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::Deferred,
        "query container capacity after the shape query contract is frozen",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(15),
        "is_empty",
        &[],
        IntrinsicCategory::Query,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V1,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::Deferred,
        "query emptiness after the shape query contract is frozen",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(16),
        "de_alloc",
        &[],
        IntrinsicCategory::Memory,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V3,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::Reject,
        "explicit deallocation once ownership and lifetime semantics exist",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(17),
        "give_back",
        &[],
        IntrinsicCategory::Memory,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V3,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::Reject,
        "return ownership once ownership semantics exist",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(18),
        "address_of",
        &[],
        IntrinsicCategory::Pointer,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V3,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::Reject,
        "take the address of a value once pointer semantics exist",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(19),
        "pointer_value",
        &[],
        IntrinsicCategory::Pointer,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V3,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::Reject,
        "read a pointer target once pointer semantics exist",
    ),
    IntrinsicEntry::new(
        IntrinsicId::new(20),
        "borrow_from",
        &[],
        IntrinsicCategory::Pointer,
        IntrinsicSurface::DotRootCall,
        IntrinsicAvailability::V3,
        IntrinsicStatus::Unsupported,
        IntrinsicArity::Exactly(1),
        IntrinsicLoweringMode::Reject,
        "borrow a value once ownership and borrowing semantics exist",
    ),
];

pub const fn intrinsic_registry() -> &'static [IntrinsicEntry] {
    INTRINSICS
}

pub const fn all_intrinsics() -> &'static [IntrinsicEntry] {
    intrinsic_registry()
}

pub fn intrinsic_by_canonical_name(name: &str) -> Option<&'static IntrinsicEntry> {
    intrinsic_registry().iter().find(|entry| entry.name == name)
}

pub fn intrinsic_by_alias(alias: &str) -> Option<&'static IntrinsicEntry> {
    intrinsic_registry()
        .iter()
        .find(|entry| entry.aliases.contains(&alias))
}

pub fn intrinsics_for_surface(surface: IntrinsicSurface) -> Vec<&'static IntrinsicEntry> {
    intrinsic_registry()
        .iter()
        .filter(|entry| entry.surface == surface)
        .collect()
}

pub fn intrinsic_by_id(id: IntrinsicId) -> Option<&'static IntrinsicEntry> {
    intrinsic_registry().iter().find(|entry| entry.id == id)
}

pub fn reserved_intrinsic_for_surface(
    surface: IntrinsicSurface,
    name: &str,
) -> Option<&'static IntrinsicEntry> {
    intrinsic_registry().iter().find(|entry| {
        entry.surface == surface
            && (entry.name == name || entry.aliases.iter().any(|alias| *alias == name))
    })
}

pub fn is_reserved_intrinsic_name_for_surface(surface: IntrinsicSurface, name: &str) -> bool {
    reserved_intrinsic_for_surface(surface, name).is_some()
}

pub fn lowering_mode_for_intrinsic(id: IntrinsicId) -> Option<IntrinsicLoweringMode> {
    intrinsic_by_id(id).map(|entry| entry.lowering_mode)
}

pub fn intrinsics_for_lowering_mode(
    lowering_mode: IntrinsicLoweringMode,
) -> Vec<&'static IntrinsicEntry> {
    intrinsic_registry()
        .iter()
        .filter(|entry| entry.lowering_mode == lowering_mode)
        .collect()
}
