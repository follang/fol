#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnsupportedLoweringSurface {
    UnaryOperators,
    BinaryOperators,
    TypeMatchingWhenOf,
    IterationLoops,
    ProcedureStyleFreeCalls,
    ProcedureStyleMethodCalls,
    EntryVariantConstruction,
}

impl UnsupportedLoweringSurface {
    pub fn label(self) -> &'static str {
        match self {
            Self::UnaryOperators => "unary-operators",
            Self::BinaryOperators => "binary-operators",
            Self::TypeMatchingWhenOf => "when-of-branches",
            Self::IterationLoops => "iteration-loops",
            Self::ProcedureStyleFreeCalls => "procedure-style-free-calls",
            Self::ProcedureStyleMethodCalls => "procedure-style-method-calls",
            Self::EntryVariantConstruction => "entry-variant-construction",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::UnaryOperators => {
                "typed unary operators other than postfix unwrap still stop at the lowering boundary"
            }
            Self::BinaryOperators => {
                "typed binary operators still stop at the lowering boundary"
            }
            Self::TypeMatchingWhenOf => {
                "typed type-matching when/of branches still stop at the lowering boundary"
            }
            Self::IterationLoops => {
                "typed iteration loops still stop at the lowering boundary"
            }
            Self::ProcedureStyleFreeCalls => {
                "typed procedure-style free calls without value results still stop at the lowering boundary"
            }
            Self::ProcedureStyleMethodCalls => {
                "typed procedure-style method calls without value results still stop at the lowering boundary"
            }
            Self::EntryVariantConstruction => {
                "typed entry variant construction through bare variant access still stops at the lowering boundary"
            }
        }
    }
}

const V1_BOUNDARIES: &[UnsupportedLoweringSurface] = &[
    UnsupportedLoweringSurface::UnaryOperators,
    UnsupportedLoweringSurface::BinaryOperators,
    UnsupportedLoweringSurface::TypeMatchingWhenOf,
    UnsupportedLoweringSurface::IterationLoops,
    UnsupportedLoweringSurface::ProcedureStyleFreeCalls,
    UnsupportedLoweringSurface::ProcedureStyleMethodCalls,
    UnsupportedLoweringSurface::EntryVariantConstruction,
];

pub fn v1_lowering_boundaries() -> &'static [UnsupportedLoweringSurface] {
    V1_BOUNDARIES
}

#[cfg(test)]
mod tests {
    use super::{v1_lowering_boundaries, UnsupportedLoweringSurface};

    #[test]
    fn v1_lowering_boundary_inventory_lists_the_current_remaining_surfaces() {
        let inventory = v1_lowering_boundaries();

        assert_eq!(
            inventory,
            &[
                UnsupportedLoweringSurface::UnaryOperators,
                UnsupportedLoweringSurface::BinaryOperators,
                UnsupportedLoweringSurface::TypeMatchingWhenOf,
                UnsupportedLoweringSurface::IterationLoops,
                UnsupportedLoweringSurface::ProcedureStyleFreeCalls,
                UnsupportedLoweringSurface::ProcedureStyleMethodCalls,
                UnsupportedLoweringSurface::EntryVariantConstruction,
            ]
        );
        assert_eq!(
            inventory
                .iter()
                .map(|surface| surface.label())
                .collect::<Vec<_>>(),
            vec![
                "unary-operators",
                "binary-operators",
                "when-of-branches",
                "iteration-loops",
                "procedure-style-free-calls",
                "procedure-style-method-calls",
                "entry-variant-construction",
            ]
        );
    }
}
