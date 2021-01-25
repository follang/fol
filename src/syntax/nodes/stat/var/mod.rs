use std::fmt;
use crate::syntax::nodes::{Node, NodeTrait, ExprTrait};

#[derive(Clone)]
pub struct VarStat{
    options: Option<Node>,
    multi: Option<(usize, String)>,
    ident: Node,
    retype: Option<Node>,
    body: Option<Node>,
}

impl NodeTrait for VarStat {}
impl ExprTrait for VarStat {}

impl fmt::Display for VarStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
