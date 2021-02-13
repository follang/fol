use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeStatAssUse{
    options: Option<Nodes>,
    ident: Option<Node>,
    data: Option<Node>,
    body: Option<Node>,
}

impl Default for NodeStatAssUse {
    fn default() -> Self {
        Self { options: None, ident: None, data: None, body: None }
    }
}

impl NodeTrait for NodeStatAssUse {}

impl fmt::Display for NodeStatAssUse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => e.to_string(), None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let data = match self.data { Some(ref e) => ": ".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}[{}]{}{};", "use", opts, ident, data)
    }
}

impl NodeStatAssUse {
    pub fn new(options: Option<Nodes>, ident: Option<Node>, data: Option<Node>, body: Option<Node>) -> Self {
        Self{ options, ident, data, body }
    }
    pub fn options(&self) -> &Option<Nodes> { &self.options }
    pub fn set_options(&mut self, options: Option<Nodes>) {
        self.options = options;
    }
    pub fn set_ident(&mut self, ident: Option<Node>) {
        self.ident = ident;
    }
    pub fn set_datatype(&mut self, dt: Option<Node>) {
        self.data = dt;
    }
}

impl From<NodeStatAssUse> for Node {
    fn from(el: NodeStatAssUse) -> Self {
        Self::new(Box::new(el)) 
    }
}
