use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, StatTrait, Opts};

#[derive(Clone)]
pub struct VarStat{
    options: Vec<Opts>,
    ident: Node,
    data: Node,
    body: Option<Node>,
}

impl NodeTrait for VarStat {}
impl StatTrait for VarStat {}

impl fmt::Display for VarStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
