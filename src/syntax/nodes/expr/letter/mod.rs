use std::fmt;
use crate::syntax::nodes::{Node, Expr};


#[derive(Clone)]
pub enum LetterExpr {
    string_normal,
    string_raw,
    string_formated,
    char_normal(char),
    char_binary(u8),
}

impl Node for LetterExpr {}
impl Expr for LetterExpr {}


impl fmt::Display for LetterExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
