use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes, StatTrait};

#[derive(Clone)]
pub struct NodeStatAssFun {
    options: Option<Nodes>,
    reciver: Option<Node>,
    ident: Option<Node>,
    parameters: Option<Nodes>,
    data: Option<Node>,
    body: Option<Node>,
}

impl Default for NodeStatAssFun {
    fn default() -> Self {
        Self { 
            options: None,
            reciver: None,
            ident: None,
            parameters: None,
            data: None,
            body: None }
    }
}

impl NodeTrait for NodeStatAssFun {}
impl StatTrait for NodeStatAssFun {}

impl fmt::Display for NodeStatAssFun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => e.to_string(), None => String::new()  };
        let reciver = match self.reciver { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let parameters = match self.parameters { Some(ref e) => e.to_string(), None => String::new()  };
        let data = match self.data { Some(ref e) => ": ".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}[{}] ({}){}({}){};", "fun", opts, reciver, ident, parameters, data)
    }
}

impl NodeStatAssFun {
    pub fn new(
            options: Option<Nodes>,
            reciver: Option<Node>,
            ident: Option<Node>,
            data: Option<Node>,
            parameters: Option<Nodes>,
            body: Option<Node> ) -> Self {
        Self{ options, reciver, ident, data, parameters, body }
    }
    pub fn set_options(&mut self, options: Option<Nodes>) {
        self.options = options;
    }
    pub fn set_reciver(&mut self, reciver: Option<Node>) {
        self.reciver = reciver;
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
