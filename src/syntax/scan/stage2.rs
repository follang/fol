#![allow(dead_code)]

use std::fmt;
use crate::syntax::point;
use crate::syntax::scan::source;
use crate::syntax::scan::text;
use crate::syntax::scan::stage1;

use crate::syntax::token::KEYWORD::*;
use crate::syntax::token::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Element {
    key: KEYWORD,
    loc: point::Location,
    con: String,
}

impl From<stage1::Element> for Element {
    fn from(el: stage1::Element) -> Self {
        Self { 
            key: el.key().clone(),
            loc: el.loc().clone(),
            con: el.con().clone(),
        }
    }
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

/// Creates a iterator that produces tokens from the input string.
pub fn elements<'a, I>(src: &mut Box<I>) -> impl Iterator<Item = Element> + '_
where I: Iterator<Item = stage1::Element> {
    std::iter::from_fn(move || {
        if let Some(v) = src.next() {
            return Some(v.into());
        }
        None
    })
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}  {}", self.loc, self.key, self.con)
    }
}

