use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes, StatTrait};

#[derive(Clone)]
pub struct VarStat{
    options: Option<Nodes>,
    ident: Option<Node>,
    data: Option<Node>,
    body: Option<Node>,
}

impl Default for VarStat {
    fn default() -> Self {
        Self { options: None, ident: None, data: None, body: None }
    }
}

impl NodeTrait for VarStat {}
impl StatTrait for VarStat {}

impl fmt::Display for VarStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let opts = match self.options { Some(ref e) => e.to_string(), None => String::new()  };
        let ident = match self.ident { Some(ref e) => " ".to_string() + &e.to_string(), None => String::new()  };
        let data = match self.data { Some(ref e) => " :".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}{}{}{}{}{};", "var","[",opts,"]", ident, data)
    }
}

impl VarStat {
    pub fn new(options: Option<Nodes>, ident: Option<Node>, data: Option<Node>, body: Option<Node>) -> Self {
        Self{ options, ident, data, body }
    }
}
