use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeStatIdent(pub String);

impl Default for NodeStatIdent {
    fn default() -> Self {
        Self { 0: String::new() }
    }
}

impl NodeTrait for NodeStatIdent {}

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
