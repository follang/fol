use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes, StatTrait};

#[derive(Clone)]
pub struct NodeStatIdent(Option<Node>);

impl Default for NodeStatIdent {
    fn default() -> Self {
        Self { 0: None }
    }
}

impl NodeTrait for NodeStatIdent {}
impl StatTrait for NodeStatIdent {}

impl fmt::Display for NodeStatIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data = match self.0 { Some(ref e) => " :".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}", data)
    }
}

impl NodeStatIdent {
    pub fn new(node: Node) -> Self {
        Self(Some(node))
    }
}
