use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeStatDecL {
    string: String,
    options: Option<Nodes>,
    generics: Option<Nodes>,
    ident: Option<Node>,
    parameters: Option<Nodes>,
    data: Option<Node>,
    body: Option<Nodes>,
}

impl Default for NodeStatDecL {
    fn default() -> Self {
        Self { 
            string: String::new(),
            options: None,
            generics: None,
            ident: None,
            parameters: None,
            data: None,
            body: None
        }
    }
}

impl NodeTrait for NodeStatDecL {}

impl fmt::Display for NodeStatDecL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => "[".to_string() + &e.to_string() + "]", None => String::new()  };
        let gen = match self.generics { Some(ref e) => "[".to_string() + &e.to_string() + "]", None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let contr = match self.parameters { Some(ref e) => e.to_string(), None => String::new()  };
        let data = match self.data { Some(ref e) => ": ".to_string() + &e.to_string(), None => String::new()  };
        let body = match self.body { Some(ref e) => "\n".to_string() + &e.print_newline(), None => String::new()  };
        write!(f, "{}{}{}{}({}){}{}", self.string, opts, gen, ident, contr, data, body)
    }
}

impl NodeStatDecL {
    pub fn set_string(&mut self, string: String) { self.string = string }
    pub fn set_options(&mut self, options: Option<Nodes>) {
        self.options = options;
    }
    pub fn set_ident(&mut self, ident: Option<Node>) {
        self.ident = ident;
    }
    pub fn set_generics(&mut self, generics: Option<Nodes>) {
        self.generics = generics;
    }
    pub fn set_parameters(&mut self, parameters: Option<Nodes>) {
        self.parameters = parameters;
    }
    pub fn set_datatype(&mut self, dt: Option<Node>) {
        self.data = dt;
    }
    pub fn set_body(&mut self, body: Option<Nodes>) {
        self.body = body;
    }
}

impl From<NodeStatDecL> for Node {
    fn from(el: NodeStatDecL) -> Self {
        Self::new(Box::new(el)) 
    }
}
