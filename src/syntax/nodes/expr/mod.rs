use dyn_clone::DynClone;
use crate::types::*;

pub mod container;
pub mod letter;
pub mod number;
pub mod binary;
pub use crate::syntax::nodes::{NodeTrait, Node};
pub use crate::syntax::nodes::expr::{
    letter::NodeExprLetter,
    container::NodeExprContainer,
    binary::NodeExprBinary,
    number::NodeExprNumber };

pub trait ExprTrait: NodeTrait {}
dyn_clone::clone_trait_object!(ExprTrait);
impl NodeTrait for Box<dyn ExprTrait> {}
pub type Expr = ID<Box<dyn ExprTrait>>;
impl From<Expr> for Node {
    fn from(expr: Expr) -> Self {
        Self {
            key: expr.key().clone(), 
            loc: expr.loc().clone(), 
            node: Box::new(expr.node().clone())
        }
    }
}
