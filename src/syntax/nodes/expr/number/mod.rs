use std::fmt;
use crate::syntax::nodes::{NodeTrait, ExprTrait};

#[derive(Clone)]
pub enum NumberExpr {
    int(isize),
    int_8(i8),
}

impl NodeTrait for NumberExpr {}
impl ExprTrait for NumberExpr {}

impl fmt::Display for NumberExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberExpr::int(i) => write!(f, "{}", i),
            NumberExpr::int_8(i) => write!(f, "{}", i)
        }
    }
}
