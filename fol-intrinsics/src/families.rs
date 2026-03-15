use crate::IntrinsicEntry;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ComparisonOperandContract {
    EqualityScalar,
    OrderedScalar,
}

impl ComparisonOperandContract {
    pub const fn expected_operands(self) -> &'static str {
        match self {
            Self::EqualityScalar => "two comparable scalar operands",
            Self::OrderedScalar => "two ordered scalar operands",
        }
    }
}

pub const fn comparison_operand_contract(
    entry: &IntrinsicEntry,
) -> Option<ComparisonOperandContract> {
    match entry.id.index() {
        0 | 1 => Some(ComparisonOperandContract::EqualityScalar),
        2 | 3 | 4 | 5 => Some(ComparisonOperandContract::OrderedScalar),
        _ => None,
    }
}
