use std::fmt;
use crate::syntax::nodes::{NodeTrait};


#[derive(Clone)]
pub enum NodeExprLetter {
    string_normal,
    string_raw,
    string_formated,
    char_normal(char),
    char_binary(u8),
}

impl NodeTrait for NodeExprLetter {}


impl fmt::Display for NodeExprLetter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
