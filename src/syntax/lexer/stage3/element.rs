use std::fmt;
use colored::Colorize;
use crate::types::*;
use crate::syntax::index::*;
use crate::syntax::point;
use crate::syntax::lexer::stage2;
use crate::syntax::token::{
    literal::LITERAL,
    void::VOID,
    symbol::SYMBOL,
    operator::OPERATOR,
    buildin::BUILDIN,
    assign::ASSIGN,
    types::TYPE,
    option::OPTION,
    form::FORM };
use crate::syntax::token::{KEYWORD, KEYWORD::*};


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Element {
    key: KEYWORD,
    loc: point::Location,
    con: String,
}

impl std::default::Default for Element {
    fn default() -> Self {
        Self {
            key: KEYWORD::illegal,
            loc: point::Location::default(),
            con: String::new(),
        }
    }
}

impl From<stage2::Element> for Element {
    fn from(stg1: stage2::Element) -> Self {
        Self { key: stg1.key().clone(), loc: stg1.loc().clone(), con: stg1.con().clone() }
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let con = if self.key().is_literal()
            || self.key().is_comment()
            || self.key().is_orbit()
            || self.key().is_makro()
            || self.key().is_ident() { " ".to_string() + &self.con + " " } else { "".to_string() };
        write!(f, "{}\t{}{}", self.loc, self.key, con.black().on_red())
    }
}


impl Element {
    pub fn init(key: KEYWORD, loc: point::Location, con: String) -> Self { Self{ key, loc, con } }
    pub fn key(&self) -> KEYWORD { self.key.clone() }
    pub fn set_key(&mut self, k: KEYWORD) { self.key = k; }
    pub fn loc(&self) -> &point::Location { &self.loc }
    pub fn set_loc(&mut self, l: point::Location) { self.loc = l; }
    pub fn con(&self) -> &String { &self.con }
    pub fn set_con(&mut self, c: String) { self.con = c; }

    pub fn append(&mut self, other: &Element) {
        self.con.push_str(&other.con);
        self.loc.longer(&other.loc.len())
    }

    pub fn analyze(&mut self, el: &mut stage2::Elements, src: &source::Source) -> Vod {
        let _one = el.curr(false)?;
        Ok(())
    }
    pub fn bump(&mut self, el: &mut stage2::Elements) {
        el.bump();
    }
}
