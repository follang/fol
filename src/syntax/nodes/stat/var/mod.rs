use crate::syntax::nodes::{Node, Tree};

pub struct VarStat{
    options: Option<Node>,
    multi: Option<(usize, String)>,
    ident: Node,
    retype: Option<Node>,
    body: Option<Node>,
}

impl Tree for VarStat {}
