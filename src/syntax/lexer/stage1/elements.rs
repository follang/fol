use std::fmt;
use crate::syntax::lexer::text;

use crate::types::*;
use crate::syntax::token::{help::*, KEYWORD, KEYWORD::*};
use crate::syntax::lexer::stage1::Element;
use crate::syntax::index::*;

pub struct Elements {
    elem: Box<dyn Iterator<Item = Con<Element>>>,
    win: Win<Con<Element>>,
    _in_count: usize,
}


impl Elements {
    pub fn init(file: &source::Source) -> Self {
        let enderr: Con<Element> = Err(Box::new(Flaw::EndError{ msg: None }));
        let initerr: Con<Element> = Err(Box::new(Flaw::InitError{ msg: None }));
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut elem = Box::new(elements(file));
        for _ in 0..SLIDER { prev.push(initerr.clone()) }
        for _ in 0..SLIDER { next.push(elem.next().unwrap_or(enderr.clone())) }
        Self {
            elem,
            win: (prev, initerr, next),
            _in_count: SLIDER
        }
    }
    pub fn curr(&self) -> Con<Element> {
        self.win.1.clone()
    }
    pub fn next_vec(&self) -> Vec<Con<Element>> {
        self.win.2.clone()
    }
    pub fn peek(&self, index: usize) -> Con<Element> { 
        let u = if index > SLIDER { 0 } else { index };
        self.next_vec()[u].clone() 
    }
    pub fn prev_vec(&self) -> Vec<Con<Element>> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, index: usize) -> Con<Element> { 
        let u = if index > SLIDER { 0 } else { index };
        self.prev_vec()[u].clone()
    }
    pub fn bump(&mut self) -> Option<Con<Element>> {
        match self.elem.next() {
            Some(v) => {
                // TODO: Handle better .ok()
                self.win.0.remove(0).ok(); self.win.0.push(self.win.1.clone());
                self.win.1 = self.win.2[0].clone();
                // TODO: Handle better .ok()
                self.win.2.remove(0).ok(); self.win.2.push(v);
                return Some(self.win.1.clone());
            },
            None => {
                if self._in_count > 0 {
                    let enderr: Con<Element> = Err(Box::new(Flaw::EndError{ msg: None }));
                    // TODO: Handle better .ok()
                    self.win.0.remove(0).ok(); self.win.0.push(self.win.1.clone());
                    self.win.1 = self.win.2[0].clone();
                    // TODO: Handle better .ok()
                    self.win.2.remove(0).ok(); self.win.2.push(enderr);
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
pub fn elements(file: &source::Source) -> impl Iterator<Item = Con<Element>>  {
    let mut txt = Box::new(text::Text::init(file));
    // *sins = *txt.sins();
    std::iter::from_fn(move || {
        if let Some(v) = txt.bump() {
            match v {
                Ok(i) => {
                    let mut loc = i.1.clone(); loc.set_len(1);
                    let mut result = Element::init(illegal, loc, String::new());
                    if let Err(err) = result.analyze(&mut txt) {
                        return Some(Err(err));
                    }
                    return Some(Ok(result));
                },
                Err(e) => { return Some(Err(e)); }
            }
        }
        None
    })
}
