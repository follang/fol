use crate::syntax::ast::Ast;
use crate::syntax::ast::expr::number::NumberExpr;

#[derive(Clone, Debug)]
pub enum BinaryExpr {
    leaf(NumberExpr),
    node(Box<BinaryExpr>, NumberExpr, Box<BinaryExpr>),
}

impl Ast for BinaryExpr {}
