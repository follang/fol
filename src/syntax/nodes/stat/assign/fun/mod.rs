use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeStatAssFun {
    options: Option<Nodes>,
    recivers: Option<Nodes>,
    ident: Option<Node>,
    parameters: Option<Nodes>,
    data: Option<Node>,
    body: Option<Node>,
}

impl Default for NodeStatAssFun {
    fn default() -> Self {
        Self { 
            options: None,
            recivers: None,
            ident: None,
            parameters: None,
            data: None,
            body: None }
    }
}

impl NodeTrait for NodeStatAssFun {}

impl fmt::Display for NodeStatAssFun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => e.to_string(), None => String::new()  };
        let recivers = match self.recivers { Some(ref e) => ", ".to_string() + &e.to_string(), None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let parameters = match self.parameters { Some(ref e) => e.print(), None => String::new()  };
        let data = match self.data { Some(ref e) => ": ".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}[{}{}]{}({}){};", "fun", opts, recivers, ident, parameters, data)
    }
}

impl NodeStatAssFun {
    pub fn new(
            options: Option<Nodes>,
            recivers: Option<Nodes>,
            ident: Option<Node>,
            parameters: Option<Nodes>,
            data: Option<Node>,
            body: Option<Node> ) -> Self {
        Self{ options, recivers, ident, data, parameters, body }
    }
    pub fn set_options(&mut self, options: Option<Nodes>) {
        self.options = options;
    }
    pub fn set_recivers(&mut self, recivers: Option<Nodes>) {
        self.recivers = recivers;
    }
    pub fn set_ident(&mut self, ident: Option<Node>) {
        self.ident = ident;
    }
    pub fn set_parameters(&mut self, parameters: Option<Nodes>) {
        self.parameters = parameters;
    }
    pub fn set_datatype(&mut self, dt: Option<Node>) {
        self.data = dt;
    }
}

impl From<NodeStatAssFun> for Node {
    fn from(el: NodeStatAssFun) -> Self {
        Self::new(Box::new(el)) 
    }
}
