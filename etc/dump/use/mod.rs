use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeStatAssUse{
    string: String,
    options: Option<Nodes>,
    ident: Option<Node>,
    data: Option<Node>,
    body: Option<Node>,
}

impl Default for NodeStatAssUse {
    fn default() -> Self {
        Self {
            string: String::new(),
            options: None,
            ident: None,
            data: None,
            body: None 
        }
    }
}

impl NodeTrait for NodeStatAssUse {}

impl fmt::Display for NodeStatAssUse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => "[".to_string() + &e.to_string() + "]", None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let data = match self.data { Some(ref e) => ": ".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}{}{}{}", self.string, opts, ident, data)
    }
}

impl NodeStatAssUse {
    pub fn set_string(&mut self, string: String) { self.string = string }
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
