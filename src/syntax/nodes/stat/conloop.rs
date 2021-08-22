use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};


#[derive(Clone)]
pub struct NodeStatLoop{
    condition: Option<Nodes>,
    body: Option<Nodes>,
}

impl Default for NodeStatLoop {
    fn default() -> Self {
        Self {
            condition: None,
            body: None
        }
    }
}

impl NodeTrait for NodeStatLoop {}

impl fmt::Display for NodeStatLoop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let condition = match self.condition { Some(ref e) => "(".to_string() + &e.to_string() + ")", None => "()".to_string() };
        let body = match self.body { Some(ref e) => "{\n".to_string() + &e.to_string() + " }", None => String::new()  };
        write!(f, "{}{}{}", "loop", condition, body)
    }
}

impl NodeStatLoop {
    pub fn set_condition(&mut self, condition: Option<Nodes>) {
        self.condition = condition;
    }
    pub fn set_body(&mut self, body: Option<Nodes>) {
        self.body = body;
    }
}

impl From<NodeStatLoop> for Node {
    fn from(el: NodeStatLoop) -> Self {
        Self::new(Box::new(el)) 
    }
}
