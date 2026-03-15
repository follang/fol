use crate::{IntrinsicAvailability, IntrinsicCategory, IntrinsicId, IntrinsicStatus, IntrinsicSurface};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntrinsicArity {
    Exactly(u8),
    AtLeast(u8),
    Between { min: u8, max: u8 },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntrinsicLoweringMode {
    GeneralIr,
    DedicatedIr,
    RuntimeHook,
    Reject,
    Deferred,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IntrinsicEntry {
    pub id: IntrinsicId,
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub category: IntrinsicCategory,
    pub surface: IntrinsicSurface,
    pub availability: IntrinsicAvailability,
    pub status: IntrinsicStatus,
    pub arity: IntrinsicArity,
    pub lowering_mode: IntrinsicLoweringMode,
    pub doc_summary: &'static str,
}

impl IntrinsicEntry {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        id: IntrinsicId,
        name: &'static str,
        aliases: &'static [&'static str],
        category: IntrinsicCategory,
        surface: IntrinsicSurface,
        availability: IntrinsicAvailability,
        status: IntrinsicStatus,
        arity: IntrinsicArity,
        lowering_mode: IntrinsicLoweringMode,
        doc_summary: &'static str,
    ) -> Self {
        Self {
            id,
            name,
            aliases,
            category,
            surface,
            availability,
            status,
            arity,
            lowering_mode,
            doc_summary,
        }
    }
}
