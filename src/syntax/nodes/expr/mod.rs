use dyn_clone::DynClone;

pub mod container;
pub mod letter;
pub mod number;
pub mod binary;
pub use crate::syntax::nodes::{NodeTrait, id};
pub use crate::syntax::nodes::expr::{
    letter::LetterExpr,
    container::ContainerExpr,
    binary::BinaryExpr,
    number::NumberExpr };

pub trait ExprTrait: NodeTrait {}
dyn_clone::clone_trait_object!(ExprTrait);

pub type Expr = id<Box<dyn ExprTrait + 'static>>;
