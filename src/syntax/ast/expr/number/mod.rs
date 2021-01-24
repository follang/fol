use crate::syntax::ast::Ast;

#[derive(Clone, Debug)]
pub enum NumberExpr {
    int(isize),
    int_8(i8),
}

impl Ast for NumberExpr {}
