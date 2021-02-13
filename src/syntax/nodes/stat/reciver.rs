use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes, StatTrait};

#[derive(Clone)]
pub struct NodeStatReciver(String);

impl Default for NodeStatReciver {
    fn default() -> Self {
        Self { 0: String::new() }
    }
}

impl NodeTrait for NodeStatReciver {}
impl StatTrait for NodeStatReciver {}

impl fmt::Display for NodeStatReciver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl NodeStatReciver {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}
