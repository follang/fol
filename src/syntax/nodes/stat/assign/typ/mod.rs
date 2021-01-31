use std::fmt;
use crate::syntax::nodes::{NodeTrait, Node, Nodes, StatTrait};

#[derive(Clone)]
pub struct TypStat {
    options: Nodes,
    ident: Node,
    body: Option<Node>,
}

impl NodeTrait for TypStat {}
impl StatTrait for TypStat {}

impl fmt::Display for TypStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
