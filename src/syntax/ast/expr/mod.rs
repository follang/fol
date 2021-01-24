pub mod container;
pub mod letter;
pub mod number;
pub mod binary;
pub use crate::syntax::ast::expr::{
    letter::LetterExpr,
    container::ContainerExpr,
    binary::BinaryExpr,
    number::NumberExpr };


#[derive(Clone, Debug)]
pub enum Expr {
    Illegal,
    Comment,
    Number,
    Letter(LetterExpr),
    Container(ContainerExpr),
    Binary(BinaryExpr),
}

