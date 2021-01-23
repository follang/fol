pub mod container;
pub mod letter;
pub mod number;
pub mod binary;

// use crate::syntax::ast::expr::{letter, container, binary};


#[derive(Clone, Debug)]
pub enum Expr {
    Illegal,
    Comment,
    Number,
    Letter(letter::LetterExpr),
    Container(container::ContainerExpr),
    Binary(binary::BinaryExpr),
}

