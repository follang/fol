use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeStatContracts(Vec<String>);

impl Default for NodeStatContracts {
    fn default() -> Self {
        Self { 0: Vec::new() }
    }
}

impl NodeTrait for NodeStatContracts {}

impl fmt::Display for NodeStatContracts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut comma_separated = String::new();
        for e in &self.0[0..self.0.len() - 1] {
                comma_separated.push_str(&e.to_string());
                comma_separated.push_str(", ");
            }
        comma_separated.push_str(&self.0[self.0.len() - 1].to_string());
        write!(f, "{}", comma_separated)
    }
}

impl NodeStatContracts {
    fn push(&mut self, strg: String) {
        self.0.push(strg);
    }
}
