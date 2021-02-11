use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes, StatTrait};

#[derive(Clone)]
pub struct NodeStatAssVar{
    options: Option<Nodes>,
    ident: Option<Node>,
    data: Option<Node>,
    body: Option<Node>,
}

impl Default for NodeStatAssVar {
    fn default() -> Self {
        Self { options: None, ident: None, data: None, body: None }
    }
}

impl NodeTrait for NodeStatAssVar {}
impl StatTrait for NodeStatAssVar {}

impl fmt::Display for NodeStatAssVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => e.to_string(), None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let data = match self.data { Some(ref e) => " :".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}{}{}{}{}{};", "var","[",opts,"]", ident, data)
    }
}

impl NodeStatAssVar {
    pub fn new(options: Option<Nodes>, ident: Option<Node>, data: Option<Node>, body: Option<Node>) -> Self {
        Self{ options, ident, data, body }
    }
    pub fn set_options(&mut self, options: Option<Nodes>) {
        self.options = options;
    }
    pub fn set_ident(&mut self, ident: Option<Node>) {
        self.ident = ident;
    }
}
