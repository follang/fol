#![allow(dead_code)]
use std::fmt;
use crate::syntax::lexer::text;

use crate::types::{Con, Win, SLIDER};
use crate::syntax::token::{help::*, KEYWORD, KEYWORD::*};
use crate::syntax::lexer::stage1::Element;

pub struct Elements {
    elem: Box<dyn Iterator<Item = Con<Element>>>,
    win: Win<Element>,
    _in_count: usize,
}


impl Elements {
    pub fn init(dir: String) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut elem = Box::new(elements(dir));
        for _ in 0..SLIDER { prev.push(Element::default()) }
        for _ in 0..SLIDER { next.push(elem.next().unwrap_or(Ok(Element::default())).unwrap()) }
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
    pub fn peek(&self, index: usize) -> Element { 
        let u = if index > SLIDER { 0 } else { index };
        self.next_vec()[u].clone() 
    }
    pub fn prev_vec(&self) -> Vec<Element> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, index: usize) -> Element { 
        let u = if index > SLIDER { 0 } else { index };
        self.prev_vec()[u].clone()
    }
    pub fn bump(&mut self) -> Option<Con<Element>> {
        match self.elem.next() {
            Some(v) => {
                self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                self.win.1 = self.win.2[0].clone();
                //TODO: fix this unwrap
                self.win.2.remove(0); self.win.2.push(v.unwrap());
                return Some(Ok(self.win.1.clone()))
            },
            None => {
                if self._in_count > 0 {
                    self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                    self.win.1 = self.win.2[0].clone();
                    self.win.2.remove(0); self.win.2.push(Element::default());
                    self._in_count -= 1;
                    return Some(Ok(self.win.1.clone()))
                } else { return None }
            }
        }
    }
}

impl Iterator for Elements {
    type Item = Element;
    fn next(&mut self) -> Option<Element> {
        loop {
            match self.bump() {
                Some(v) => {
                    match v {
                        Ok(i) => { return Some(i) },
                        Err(_) => continue
                    }
                },
                None => return None
            }
        }
    }
}

/// Creates a iterator that produces tokens from the input string.
pub fn elements(dir: String) -> impl Iterator<Item = Con<Element>>  {
    let mut txt = Box::new(text::Text::init(dir));
    // *sins = *txt.sins();
    std::iter::from_fn(move || {
        if let Some(_) = txt.bump() {
            let mut loc = txt.curr().1.clone(); loc.set_len(1);
            let mut result = Element::init(illegal, loc, String::new());
            if txt.curr().0 == '/' && (txt.peek(0).0 == '/' || txt.peek(0).0 == '*') {
                result.comment(&mut txt);
            } else if is_eol(&txt.curr().0) {
                result.endline(&mut txt, false);
            } else if is_space(&txt.curr().0) {
                result.space(&mut txt);
            } else if txt.curr().0 == '"' || txt.curr().0 == '\'' || txt.curr().0 == '`' {
                result.encap(&mut txt);
            } else if is_digit(&txt.curr().0) {
                result.digit(&mut txt);
            } else if is_symbol(&txt.curr().0) {
                result.symbol(&mut txt);
            } else if is_alpha(&txt.curr().0) {
                result.alpha(&mut txt);
            }
            return Some(Ok(result));
        }
        None
    })
}
