use crate::syntax::ast::Tree;

#[derive(Clone, Debug)]
pub struct ContainerExpr {
    uniform: bool,
    ulements: Box<Tree>,
}
