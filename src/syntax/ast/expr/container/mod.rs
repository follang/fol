use crate::syntax::ast::{Node, Tree};

pub struct ContainerExpr {
    uniform: bool,
    ulements: Node,
}

impl Tree for ContainerExpr {}
