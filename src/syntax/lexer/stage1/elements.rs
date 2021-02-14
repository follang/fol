use std::fmt;
use crate::types::{Vod, Con, Win, SLIDER};
use crate::syntax::token::{help, KEYWORD, KEYWORD::*};
use crate::syntax::lexer::stage0;
use crate::syntax::lexer::stage1::Element;
use crate::syntax::index;

pub struct Elements {
    elem: Box<dyn Iterator<Item = Con<Element>>>,
    win: Win<Con<Element>>,
    _in_count: usize,
    _source: index::Source,
}


impl Elements {
    pub fn source(&self) -> index::Source { self._source.clone() }
    pub fn init(file: &index::Source) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut elem = Box::new(elements(file));
        for _ in 0..SLIDER { prev.push(Ok(Element::default())) }
        for _ in 0..SLIDER { next.push(elem.next().unwrap_or(Ok(Element::default()))) }
        Self {
            elem,
            win: (prev, Ok(Element::default()), next),
            _in_count: SLIDER,
            _source: file.clone(),
        }
    }
    pub fn curr(&self) -> Con<Element> {
        self.win.1.clone()
    }
    pub fn next_vec(&self) -> Vec<Con<Element>> {
        self.win.2.clone()
    }
    pub fn peek(&self, indx: usize) -> Con<Element> { 
        let u = if indx > SLIDER { 0 } else { indx };
        self.next_vec()[u].clone() 
    }
    pub fn prev_vec(&self) -> Vec<Con<Element>> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, indx: usize) -> Con<Element> { 
        let u = if indx > SLIDER { 0 } else { indx };
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
                    // TODO: Handle better .ok()
                    self.win.0.remove(0).ok(); self.win.0.push(self.win.1.clone());
                    self.win.1 = self.win.2[0].clone();
                    // TODO: Handle better .ok()
                    self.win.2.remove(0).ok(); self.win.2.push(Ok(Element::default()));
                    self._in_count -= 1;
                    return Some(self.win.1.clone())
                } else { return None }
            }
        }
    }
    pub fn debug(&self) -> Vod {
        println!("{}\t{}\t{}", self.curr()?.loc(), self.curr()?.key(), self.curr()?.con());
        Ok(())
    }
    pub fn echo(&self) { println!(">>>>>>>>>>>>>>>>>>>>>") }
}

impl Iterator for Elements {
    type Item = Con<Element>;
    fn next(&mut self) -> Option<Self::Item> {
        self.bump()
    }
}


/// Creates a iterator that produces tokens from the input string.
pub fn elements(file: &index::Source) -> impl Iterator<Item = Con<Element>>  {
    let mut txt = Box::new(stage0::Elements::init(file, false));
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
