use std::fmt;
use crate::types::*;
use crate::syntax::token::*;
use crate::syntax::lexer::stage1;
use crate::syntax::lexer::stage2::Element;


pub struct Elements {
    elem: Box<dyn Iterator<Item = Con<Element>>>,
    win: Win<Element>,
    _in_count: u8,
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
            _in_count: SLIDER as u8
        }
    }
    pub fn curr(&self, ignore: bool) -> Element {
        if ignore && self.win.1.key().is_space() { self.peek(0, false) } else { self.win.1.clone() }
    }
    pub fn next_vec(&self) -> Vec<Element> {
        self.win.2.clone()
    }
    pub fn peek(&self, index: usize, ignore: bool) -> Element { 
        let mut u = if index > SLIDER { SLIDER } else { index };
        if ignore && self.next_vec()[u].key().is_space() && u < SLIDER { u += 1 };
        self.next_vec()[u].clone() 
    }
    pub fn prev_vec(&self) -> Vec<Element> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, index: usize, ignore: bool) -> Element { 
        let mut u = if index > SLIDER { SLIDER } else { index };
        if ignore && self.next_vec()[u].key().is_space() && u < SLIDER { u += 1 };
        self.prev_vec()[u].clone() 
    }
    pub fn expect(&self, keyword: KEYWORD, ignore: bool) -> Vod {
        if self.curr(ignore).key() == keyword {
            return Ok(())
        };
        Err( catch!( Typo::ParserManyUnexpected{ msg: None, loc: Some(self.curr(ignore).loc().clone()) } ))
    }

    pub fn bump(&mut self) -> Option<Con<Element>> {
        match self.elem.next() {
            Some(v) => {
                match v {
                    Ok(e) => {
                        self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                        self.win.1 = self.win.2[0].clone();
                        self.win.2.remove(0); self.win.2.push(e);
                        return Some(Ok(self.win.1.clone()));
                    },
                    Err(e) => {
                        return Some(Err(e));
                    }
                }
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
    let mut stg = Box::new(stage1::Elements::init(dir));
    std::iter::from_fn(move || {
        if let Some(v) = stg.bump() {
            match v {
                Ok(el) => {
                    let mut result: Element = el.into();
                    if let Err(err) = result.analyze(&mut stg) {
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

