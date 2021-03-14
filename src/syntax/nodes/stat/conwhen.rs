use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};


#[derive(Clone)]
pub struct NodeStatWhen{
    condition: Option<Nodes>,
    body: Option<Nodes>,
}

impl Default for NodeStatWhen {
    fn default() -> Self {
        Self {
            condition: None,
            body: None
        }
    }
}

impl NodeTrait for NodeStatWhen {}

impl fmt::Display for NodeStatWhen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let condition = match self.condition { Some(ref e) => "(".to_string() + &e.to_string() + ")", None => String::new()  };
        let body = match self.body { Some(ref e) => " = ".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}{}{}", "loop", condition, body)
    }
}

impl NodeStatWhen {
    pub fn set_condition(&mut self, condition: Option<Nodes>) {
        self.condition = condition;
    }
    pub fn set_body(&mut self, body: Option<Nodes>) {
        self.body = body;
    }
}

impl From<NodeStatWhen> for Node {
    fn from(el: NodeStatWhen) -> Self {
        Self::new(Box::new(el)) 
    }
}
