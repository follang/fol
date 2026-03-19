use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LITERAL {
    CookedQuoted,
    RawQuoted,
    Float,
    Decimal,
    Hexadecimal,
    Octal,
    Binary,
}

impl fmt::Display for LITERAL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            LITERAL::CookedQuoted => "cooked-quoted",
            LITERAL::RawQuoted => "raw-quoted",
            LITERAL::Float => "float",
            LITERAL::Decimal => "decimal",
            LITERAL::Hexadecimal => "hexadecimal",
            LITERAL::Octal => "octal",
            LITERAL::Binary => "binary",
        };
        write!(f, "LITERAL:{}", t)
    }
}
