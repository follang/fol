use std::fmt;
use crate::syntax::nodes::{NodeTrait, Expr, ExprTrait, NumberExpr};

#[derive(Clone)]
pub enum BinaryExpr {
    leaf(NumberExpr),
    node(Expr, NumberExpr, Expr),
}

impl NodeTrait for BinaryExpr {}
impl ExprTrait for BinaryExpr {}

impl fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
