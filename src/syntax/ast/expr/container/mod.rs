use crate::syntax::ast::{Tree, Ast};

#[derive(Clone, Debug)]
pub struct ContainerExpr {
    uniform: bool,
    ulements: Box<Tree>,
}

impl Ast for ContainerExpr {}
