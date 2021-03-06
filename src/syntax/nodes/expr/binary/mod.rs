use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node};

#[derive(Clone)]
pub enum NodeExprBinary {
    leaf(Node),
    node(Node, Node, Node),
}

impl NodeTrait for NodeExprBinary {}

impl fmt::Display for NodeExprBinary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
