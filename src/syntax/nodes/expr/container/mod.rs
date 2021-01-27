use std::fmt;
use crate::syntax::nodes::{NodeTrait, ExprTrait, Node};

#[derive(Clone)]
pub struct ContainerExpr {
    uniform: bool,
    elements: Vec<Node>,
}

impl NodeTrait for ContainerExpr {}
impl ExprTrait for ContainerExpr {}


impl fmt::Display for ContainerExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
