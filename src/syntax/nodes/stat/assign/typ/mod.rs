use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes, StatTrait};

#[derive(Clone)]
pub struct NodeStatAssTyp {
    options: Option<Nodes>,
    ident: Option<Node>,
    contracts: Option<Nodes>,
    data: Option<Node>,
    body: Option<Node>,
}

impl Default for NodeStatAssTyp {
    fn default() -> Self {
        Self { options: None, ident: None, contracts: None, data: None, body: None }
    }
}

impl NodeTrait for NodeStatAssTyp {}
impl StatTrait for NodeStatAssTyp {}

impl fmt::Display for NodeStatAssTyp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => e.to_string(), None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let contr = match self.contracts { Some(ref e) => e.to_string(), None => String::new()  };
        let data = match self.data { Some(ref e) => ": ".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}[{}]{}({}){};", "typ", opts, ident, contr, data)
    }
}

impl NodeStatAssTyp {
    pub fn new(options: Option<Nodes>, ident: Option<Node>, data: Option<Node>, contracts: Option<Nodes>, body: Option<Node>) -> Self {
        Self{ options, ident, data, contracts, body }
    }
    pub fn set_options(&mut self, options: Option<Nodes>) {
        self.options = options;
    }
    pub fn set_ident(&mut self, ident: Option<Node>) {
        self.ident = ident;
    }
    pub fn set_contracts(&mut self, contracts: Option<Nodes>) {
        self.contracts = contracts;
    }
    pub fn set_datatype(&mut self, dt: Option<Node>) {
        self.data = dt;
    }
}

impl From<NodeStatAssTyp> for Node {
    fn from(el: NodeStatAssTyp) -> Self {
        Self::new(Box::new(el)) 
    }
}
