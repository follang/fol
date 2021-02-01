use std::fmt;
use crate::syntax::nodes::{NodeTrait, ExprTrait, Node};

#[derive(Clone)]
pub struct NodeExprContainer {
    uniform: bool,
    elements: Vec<Node>,
}

impl NodeTrait for NodeExprContainer {}
impl ExprTrait for NodeExprContainer {}


impl fmt::Display for NodeExprContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
