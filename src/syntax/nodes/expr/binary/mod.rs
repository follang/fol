use std::fmt;
use crate::syntax::nodes::{NodeTrait, ExprTrait, NumberExpr};

#[derive(Clone)]
pub enum BinaryExpr {
    leaf(NumberExpr),
    node(Box<dyn ExprTrait>, NumberExpr, Box<dyn ExprTrait>),
}

impl NodeTrait for BinaryExpr {}
impl ExprTrait for BinaryExpr {}

impl fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
