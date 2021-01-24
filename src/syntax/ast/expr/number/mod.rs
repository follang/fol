use crate::syntax::ast::Tree;

pub enum NumberExpr {
    int(isize),
    int_8(i8),
}

impl Tree for NumberExpr {}
