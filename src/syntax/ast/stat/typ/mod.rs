use crate::syntax::ast::{Node, Tree};

pub struct TypStat {
    options: Option<Node>,
    multi: Option<(usize, String)>,
    ident: Node,
    generics: Option<Vec<(Node, Node)>>,
    contract: Option<Vec<Node>>,
    retype: Option<Node>,
    body: Option<Node>,
}

impl Tree for TypStat {}
