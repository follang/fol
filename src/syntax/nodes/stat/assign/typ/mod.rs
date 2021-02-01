use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes, StatTrait};

#[derive(Clone)]
pub struct NodeStatAssTyp {
    options: Nodes,
    ident: Node,
    body: Option<Node>,
}

impl NodeTrait for NodeStatAssTyp {}
impl StatTrait for NodeStatAssTyp {}

impl fmt::Display for NodeStatAssTyp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
