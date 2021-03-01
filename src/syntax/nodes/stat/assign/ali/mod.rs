use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeStatAssAli {
    options: Option<Nodes>,
    ident: Option<Node>,
    data: Option<Node>,
    body: Option<Node>,
}

impl Default for NodeStatAssAli {
    fn default() -> Self {
        Self { options: None, ident: None, data: None, body: None }
    }
}

impl NodeTrait for NodeStatAssAli {}

impl fmt::Display for NodeStatAssAli {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => "[".to_string() + &e.to_string() + "]", None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let data = match self.data { Some(ref e) => ": ".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}{}{}{}", "ali", opts, ident, data)
    }
}

impl NodeStatAssAli {
    pub fn new(options: Option<Nodes>, ident: Option<Node>, data: Option<Node>, body: Option<Node>) -> Self {
        Self{ options, ident, data, body }
    }
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

impl From<NodeStatAssAli> for Node {
    fn from(el: NodeStatAssAli) -> Self {
        Self::new(Box::new(el)) 
    }
}
