use std::fmt;
use crate::syntax::nodes::{NodeTrait, ExprTrait};

#[derive(Clone)]
pub enum LetterExpr {
    string_normal,
    string_raw,
    string_formated,
    char_normal(char),
    char_binary(u8),
}

impl NodeTrait for LetterExpr {}
impl ExprTrait for LetterExpr {}


impl fmt::Display for LetterExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
