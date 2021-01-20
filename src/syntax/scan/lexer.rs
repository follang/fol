#![allow(dead_code)]

use std::fmt;
use crate::syntax::point;
use crate::syntax::scan::source;
use crate::syntax::scan::text;
use crate::syntax::scan::stage1;
use crate::types::*;
use crate::syntax::error::*;

use crate::syntax::token::KEYWORD::*;
use crate::syntax::token::*;


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

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}  {}", self.loc, self.key, self.con)
    }
}


impl Element {
    pub fn init(key: KEYWORD, loc: point::Location, con: String) -> Self { Self{ key, loc, con } }
    pub fn key(&self) -> &KEYWORD { &self.key }
    pub fn set_key(&mut self, k: KEYWORD) { self.key = k; }
    pub fn loc(&self) -> &point::Location { &self.loc }
    pub fn con(&self) -> &String { &self.con }
}

pub struct Elements {
    elem: Box<dyn Iterator<Item = Element>>,
    win: Win<Element>,
    _in_count: usize,
}


impl Elements {
    pub fn init(dir: String) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut elem = Box::new(elements(dir));
        for _ in 0..SLIDER { prev.push(Element::default()) }
        for _ in 0..SLIDER { next.push(elem.next().unwrap()) }
        Self {
            elem,
            win: (prev, Element::default(), next),
            _in_count: SLIDER
        }
    }
    pub fn curr(&self) -> Element {
        self.win.1.clone()
    }
    pub fn next_vec(&self) -> Vec<Element> {
        self.win.2.clone()
    }
    pub fn peek(&self, u: usize) -> Element { 
        if u > SLIDER { format!("{} is begger than SLIDER: {}", u, SLIDER); }
        self.next_vec()[0].clone() 
    }
    pub fn prev_vec(&self) -> Vec<Element> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, u: usize) -> Element { 
        if u > SLIDER { format!("{} is begger than SLIDER: {}", u, SLIDER); }
        self.prev_vec()[0].clone() 
    }
    pub fn bump(&mut self) -> Opt<Element> {
        match self.elem.next() {
            Some(v) => {
                self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                self.win.1 = self.win.2[0].clone();
                self.win.2.remove(0); self.win.2.push(v);
                return Some(self.win.1.clone())
            },
            None => {
                if self._in_count > 0 {
                    self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                    self.win.1 = self.win.2[0].clone();
                    self.win.2.remove(0); self.win.2.push(Element::default());
                    self._in_count -= 1;
                    return Some(self.win.1.clone())
                } else { return None }
            }
        }
    }
}

impl Iterator for Elements {
    type Item = Element;
    fn next(&mut self) -> Option<Element> {
        match self.bump() {
            Some(v) => Some(v),
            None => None
        }
    }
}

/// Creates a iterator that produces tokens from the input string.
pub fn elements(dir: String) -> impl Iterator<Item = Element>  {
    let mut txt = Box::new(text::Text::init(dir));
    std::iter::from_fn(move || {
        if let Some(v) = txt.bump() {
            let loc = v.1.1.clone();
            let mut result = Element::init(illegal, loc.clone(), String::new());
            result.loc.new_word();
            // if txt.curr().0 == '/' && (txt.peek(0).0 == '/' || txt.peek(0).0 == '*') {
            //     result.comment(&mut txt);
            // } else if is_eol(&txt.curr().0) {
            //     result.endline(&mut txt, false);
            // } else if is_space(&txt.curr().0) {
            //     result.space(&mut txt);
            // } else if txt.curr().0 == '"' || txt.curr().0 == '\'' || txt.curr().0 == '`' {
            //     result.encap(&mut txt);
            // } else if is_digit(&txt.curr().0) {
            //     result.digit(&mut txt);
            // } else if is_symbol(&txt.curr().0) {
            //     result.symbol(&mut txt);
            // } else if is_alpha(&txt.curr().0) {
            //     result.alpha(&mut txt);
            // }
            return Some(result);
        }
        None
    })
}
