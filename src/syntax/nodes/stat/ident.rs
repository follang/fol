use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes, StatTrait};

#[derive(Clone)]
pub struct NodeStatIdent(String);

impl Default for NodeStatIdent {
    fn default() -> Self {
        Self { 0: String::new() }
    }
}

impl NodeTrait for NodeStatIdent {}
impl StatTrait for NodeStatIdent {}

impl fmt::Display for NodeStatIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NodeStatIdent {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}
