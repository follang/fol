use dyn_clone::DynClone;

pub mod container;
pub mod letter;
pub mod number;
pub mod binary;
pub use crate::syntax::nodes::{NodeTrait, Node, id};
pub use crate::syntax::nodes::expr::{
    letter::LetterExpr,
    container::ContainerExpr,
    binary::BinaryExpr,
    number::NumberExpr };

pub trait ExprTrait: NodeTrait {}
dyn_clone::clone_trait_object!(ExprTrait);
impl NodeTrait for Box<dyn ExprTrait> {}

pub type Expr = id<Box<dyn ExprTrait>>;
impl From<Expr> for Node {
    fn from(expr: Expr) -> Self {
        Self {
            loc: expr.get_loc().clone(), 
            node: Box::new(expr.get_node().clone())
        }
    }
}
