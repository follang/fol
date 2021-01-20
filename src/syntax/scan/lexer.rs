#![allow(dead_code)]

use std::fmt;
use crate::syntax::point;
use crate::syntax::scan::source;
// use crate::syntax::scan::text;
use crate::syntax::scan::text;

use crate::syntax::token::KEYWORD::*;
use crate::syntax::token::*;
use crate::syntax::error::*;


const SLIDER: usize = 9;


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

impl Element {
    pub fn empty(key: KEYWORD, loc: point::Location, con: String) -> Self {
        Element { key, loc, con }
    }
    pub fn key(&self) -> &KEYWORD {
        &self.key
    }
    pub fn loc(&self) -> &point::Location {
        &self.loc
    }
    pub fn con(&self) -> &String {
        &self.con
    }
    pub fn set_key(&mut self, k: KEYWORD) {
        self.key = k;
    }
}
