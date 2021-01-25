use std::fmt;
use crate::syntax::nodes::{Node, NodeTrait, ExprTrait};

#[derive(Clone)]
pub struct TypStat {
    options: Option<Node>,
    multi: Option<(usize, String)>,
    ident: Node,
    generics: Option<Vec<(Node, Node)>>,
    contract: Option<Vec<Node>>,
    retype: Option<Node>,
    body: Option<Node>,
}

impl NodeTrait for TypStat {}
impl ExprTrait for TypStat {}

impl fmt::Display for TypStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
