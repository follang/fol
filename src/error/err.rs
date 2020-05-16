#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


use std::fmt;
use crate::scan::locate;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct error {
    typ: TYPE,
    msg: String,
    loc: locate::LOCATION,
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ERROR {
    el: Vec<error>
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TYPE {
    lexer,
    parser,
}
impl error {
    pub fn new(typ: TYPE, msg: &str, loc: locate::LOCATION) -> Self {
        error { typ, msg: msg.to_string(), loc }
    }
}

impl ERROR {
    pub fn init() -> Self {
        ERROR{ el: Vec::new() }
    }

    pub fn list(&self) -> &Vec<error> {
        &self.el
    }

    pub fn report(&mut self, typ: TYPE, msg: &str, loc:locate::LOCATION) {
        let e = error::new(typ, msg, loc);
        &self.el.push(e);
    }

    pub fn show(&mut self) {
        for e in self.el.iter() {
            println!("{}", e)
        }
    }
}

impl fmt::Display for error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.typ {
            TYPE::lexer => write!(f, "{: <18} {}\n{}", "LEXER error at:", self.loc, self.msg),
            TYPE::parser => write!(f, "{: <18} {}\n{}", "PARSER error at:", self.loc, self.msg)
        }
    }
}
