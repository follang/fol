use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};

#[derive(Clone)]
pub struct NodeStatDatatypes {
    string: String,
    form: Option<Nodes>,
    bound: Option<Node>,
}

impl Default for NodeStatDatatypes {
    fn default() -> Self {
        Self { string: String::new(), form: None, bound: None }
    }
}

impl NodeTrait for NodeStatDatatypes {}

impl fmt::Display for NodeStatDatatypes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let form = match self.form { Some(ref e) => "[".to_string() + &e.to_string() + "]", None => String::new()  };
        let bound = match self.bound { Some(ref e) => "[".to_string() + &e.to_string() + "]", None => String::new()  };
        write!(f, "{}{}{}", self.string, form, bound )
    }
}

impl NodeStatDatatypes {
    pub fn new(
            string: String,
            form: Option<Nodes>,
            bound: Option<Node>,
        ) -> Self { Self{ string, form, bound } }

    pub fn set_string(&mut self, string: String) {
        self.string = string;
    }
    pub fn set_form(&mut self, form: Option<Nodes>) {
        self.form = form;
    }
    pub fn set_bound(&mut self, bound: Option<Node>) {
        self.bound = bound;
    }
}

impl From<NodeStatDatatypes> for Node {
    fn from(el: NodeStatDatatypes) -> Self {
        Self::new(Box::new(el)) 
    }
}
