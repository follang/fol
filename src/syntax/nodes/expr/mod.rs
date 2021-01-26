use dyn_clone::DynClone;

pub mod container;
pub mod letter;
pub mod number;
pub mod binary;
pub use crate::syntax::nodes::Node;
pub use crate::syntax::nodes::expr::{
    letter::LetterExpr,
    container::ContainerExpr,
    binary::BinaryExpr,
    number::NumberExpr };

pub trait Expr: Node {}
dyn_clone::clone_trait_object!(Expr);
