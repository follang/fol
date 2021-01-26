use std::fmt;
use crate::syntax::nodes::{Node, Expr};

#[derive(Clone)]
pub struct ContainerExpr {
    uniform: bool,
    elements: Box<dyn Node>,
}

impl Node for ContainerExpr {}
impl Expr for ContainerExpr {}


impl fmt::Display for ContainerExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
