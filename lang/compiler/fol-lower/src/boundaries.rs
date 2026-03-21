#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnsupportedLoweringSurface {
    TypeMatchingWhenOf,
    EntryVariantConstruction,
}

impl UnsupportedLoweringSurface {
    pub fn label(self) -> &'static str {
        match self {
            Self::TypeMatchingWhenOf => "when-of-branches",
            Self::EntryVariantConstruction => "entry-variant-construction",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::TypeMatchingWhenOf => {
                "typed type-matching when/of branches still stop at the lowering boundary"
            }
            Self::EntryVariantConstruction => {
                "typed entry variant construction through bare variant access still stops at the lowering boundary"
            }
        }
    }
}

const V1_BOUNDARIES: &[UnsupportedLoweringSurface] = &[
    UnsupportedLoweringSurface::TypeMatchingWhenOf,
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
                UnsupportedLoweringSurface::TypeMatchingWhenOf,
                UnsupportedLoweringSurface::EntryVariantConstruction,
            ]
        );
        assert_eq!(
            inventory
                .iter()
                .map(|surface| surface.label())
                .collect::<Vec<_>>(),
            vec!["when-of-branches", "entry-variant-construction",]
        );
    }
}
