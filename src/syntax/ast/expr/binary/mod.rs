use crate::syntax::ast::Tree;
use crate::syntax::ast::expr::number::NumberExpr;

pub enum BinaryExpr {
    leaf(NumberExpr),
    node(Box<BinaryExpr>, NumberExpr, Box<BinaryExpr>),
}

impl Tree for BinaryExpr {}
