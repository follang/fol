use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeStatAssTyp {
    string: String,
    options: Option<Nodes>,
    generics: Option<Nodes>,
    ident: Option<Node>,
    parameters: Option<Nodes>,
    data: Option<Node>,
    body: Option<Node>,
}

impl Default for NodeStatAssTyp {
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

impl NodeTrait for NodeStatAssTyp {}

impl fmt::Display for NodeStatAssTyp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let opts = match self.options { Some(ref e) => "[".to_string() + &e.to_string() + "]", None => String::new()  };
        let gen = match self.generics { Some(ref e) => "[".to_string() + &e.to_string() + "]", None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let contr = match self.parameters { Some(ref e) => e.to_string(), None => String::new()  };
        let data = match self.data { Some(ref e) => ": ".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}{}{}{}({}){}", self.string, opts, gen, ident, contr, data)
    }
}

impl NodeStatAssTyp {
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
}

impl From<NodeStatAssTyp> for Node {
    fn from(el: NodeStatAssTyp) -> Self {
        Self::new(Box::new(el)) 
    }
}
