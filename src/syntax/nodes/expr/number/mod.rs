use std::fmt;
use crate::syntax::nodes::{Node, Expr};

#[derive(Clone)]
pub enum NumberExpr {
    int(isize),
    int_8(i8),
}

impl Node for NumberExpr {}
impl Expr for NumberExpr {}

impl fmt::Display for NumberExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
