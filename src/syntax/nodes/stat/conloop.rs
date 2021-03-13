use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes};


#[derive(Clone)]
pub enum LoopKind {
    Enum,
    Cond,
    Infi
}

impl fmt::Display for LoopKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoopKind::Enum => { write!(f, "{}", "enum".to_string()) },
            LoopKind::Cond => { write!(f, "{}", "cond".to_string()) },
            LoopKind::Infi => { write!(f, "{}", "infi".to_string()) },
        }
    }
}
 

#[derive(Clone)]
pub struct NodeStatLoop{
    condition: Option<Nodes>,
    kind: Option<LoopKind>,
    body: Option<Nodes>,
}

impl Default for NodeStatLoop {
    fn default() -> Self {
        Self {
            condition: None,
            kind: None,
            body: None
        }
    }
}

impl NodeTrait for NodeStatLoop {}

impl fmt::Display for NodeStatLoop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let kind = match self.kind { Some(ref e) => "[".to_string() + &e.to_string() + "]", None => String::new()  };
        let condition = match self.condition { Some(ref e) => "(".to_string() + &e.to_string() + ")", None => String::new()  };
        let body = match self.body { Some(ref e) => " = ".to_string() + &e.to_string(), None => String::new()  };
        write!(f, "{}{}{}{}", "loop", kind, condition, body)
    }
}

impl NodeStatLoop {
    pub fn set_condition(&mut self, condition: Option<Nodes>) {
        self.condition = condition;
    }
    pub fn set_kind(&mut self, kind: Option<LoopKind>) {
        self.kind = kind;
    }
    pub fn set_body(&mut self, body: Option<Nodes>) {
        self.body = body;
    }
}

impl From<NodeStatLoop> for Node {
    fn from(el: NodeStatLoop) -> Self {
        Self::new(Box::new(el)) 
    }
}
