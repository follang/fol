use crate::syntax::nodes::Tree;
use crate::syntax::nodes::expr::number::NumberExpr;

pub enum BinaryExpr {
    leaf(NumberExpr),
    node(Box<BinaryExpr>, NumberExpr, Box<BinaryExpr>),
}

impl Tree for BinaryExpr {}
