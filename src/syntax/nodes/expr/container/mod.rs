use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeExprContainer {
    uniform: bool,
    elements: Vec<Node>,
}

impl NodeTrait for NodeExprContainer {}


impl fmt::Display for NodeExprContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
