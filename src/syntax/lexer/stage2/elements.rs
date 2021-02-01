use std::fmt;
use crate::types::*;
use crate::syntax::token::*;
use crate::syntax::lexer::stage1;
use crate::syntax::lexer::stage2::Element;
use crate::syntax::index::*;


pub struct Elements {
    elem: Box<dyn Iterator<Item = Con<Element>>>,
    win: Win<Element>,
    _in_count: u8,
}


impl Elements {
    pub fn init(file: &source::Source) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut elem = Box::new(elements(file));
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
    pub fn jump(&mut self, loops: isize, elem: bool) {
        for _ in 0..loops+1 {
            if elem && self.curr(false).key().is_void() {
                self.bump();
            }
            self.bump();
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
    let mut stg = Box::new(stage1::Elements::init(file));
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




impl Elements {
    pub fn until_term(&mut self, term: bool) {
        loop{ 
            self.bump();
            if self.curr(false).key().is_terminal() || self.curr(false).key().is_eof() {
                if term { self.bump(); }
                break
            }
        }
    }
    pub fn until_char(&mut self, el: &str) {
        loop{ 
            if self.curr(false).con() == &el.to_string() || self.curr(false).key().is_eof() {
                break
            }
            self.bump();
        }
    }
    pub fn until_bracket(&mut self) {
        let deep = self.curr(false).loc().deep() - 1;
        loop{
            if (self.curr(false).key().is_close_bracket() && self.curr(false).loc().deep() == deep) 
                || self.curr(false).key().is_eof() {
                break
            }
            self.bump();
        }
        self.bump();
    }
    pub fn debug(&self) {
        println!("{}\t{}", self.curr(false).loc(), self.curr(false).key());
    }
}


impl Elements {
    pub fn expect(&self, keyword: KEYWORD, ignore: bool) -> Vod {
        if self.curr(ignore).key() == keyword {
            return Ok(())
        };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: keyword,
        }))
    }
    pub fn expect_many(&self, keywords: Vec<KEYWORD>, ignore: bool) -> Vod {
        if let Some(e) = keywords.iter().find(|&x| x == &self.curr(ignore).key()) {
            return Ok(())
        }
        Err( catch!( Typo::ParserManyUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            keys: keywords,
        }))
    }
    pub fn expect_option(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore).key(), KEYWORD::option(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: KEYWORD::option(OPTION::ANY),
        }))
    }
    pub fn expect_assign(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore).key(), KEYWORD::assign(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: KEYWORD::assign(ASSIGN::ANY), 
        }))
    }
    pub fn expect_types(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore).key(), KEYWORD::types(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: KEYWORD::types(TYPE::ANY), 
        }))
    }
    pub fn expect_form(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore).key(), KEYWORD::form(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: KEYWORD::form(FORM::ANY), 
        }))
    }
    pub fn expect_literal(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore).key(), KEYWORD::literal(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: KEYWORD::literal(LITERAL::ANY), 
        }))
    }
    pub fn expect_buildin(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore).key(), KEYWORD::buildin(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: KEYWORD::buildin(BUILDIN::ANY), 
        }))
    }
    pub fn expect_symbol(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore).key(), KEYWORD::symbol(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: KEYWORD::symbol(SYMBOL::ANY), 
        }))
    }
    pub fn expect_operator(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore).key(), KEYWORD::operator(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: KEYWORD::operator(OPERATOR::ANY), 
        }))
    }
    pub fn expect_void(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore).key(), KEYWORD::void(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore).loc().clone()), 
            key1: self.curr(ignore).key(), 
            key2: KEYWORD::void(VOID::ANY), 
        }))
    }
}
