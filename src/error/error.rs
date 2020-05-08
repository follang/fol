#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


use std::fmt;
use crate::scan::locate;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ERROR {
    typ: TYPE,
    rec: bool,
    msg: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TYPE {
    lexer_,
    parser_,
}
impl ERROR {

    pub fn report(typ: TYPE, loc: locate::LOCATION, rec: bool, costum_msg: String) -> Self {
        let msg = String::new();
        ERROR { typ, rec, msg }
    }
}

impl fmt::Display for ERROR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.typ {
            TYPE::lexer_ => write!(f, "{}", self.msg),
            TYPE::parser_ => write!(f, "{}", self.msg)
        }
    }
}
