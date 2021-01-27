use std::fmt;
use crate::syntax::nodes::{NodeTrait, ExprTrait, NumberExpr, Node};

#[derive(Clone)]
pub enum BinaryExpr {
    leaf(NumberExpr),
    node(Node, NumberExpr, Node),
}

impl NodeTrait for BinaryExpr {}
impl ExprTrait for BinaryExpr {}

impl fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
