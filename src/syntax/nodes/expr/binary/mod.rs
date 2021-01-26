use std::fmt;
use crate::syntax::nodes::{Node, Expr, NumberExpr};

#[derive(Clone)]
pub enum BinaryExpr {
    leaf(NumberExpr),
    node(Box<dyn Expr>, NumberExpr, Box<dyn Expr>),
}

impl Node for BinaryExpr {}
impl Expr for BinaryExpr {}

impl fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
