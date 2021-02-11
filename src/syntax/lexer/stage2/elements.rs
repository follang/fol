use std::fmt;
use crate::types::*;
use crate::syntax::token::*;
use crate::syntax::lexer::stage1;
use crate::syntax::lexer::stage2::Element;
use crate::syntax::index::*;


pub struct Elements {
    elem: Box<dyn Iterator<Item = Con<Element>>>,
    win: Win<Con<Element>>,
    _in_count: usize,
    _source: Source,
}


impl Elements {
    pub fn init(file: &source::Source) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut elem = Box::new(elements(file));
        for _ in 0..SLIDER { prev.push(Ok(Element::default())) }
        for _ in 0..SLIDER { next.push(elem.next().unwrap_or(Ok(Element::default()))) }
        let newborn = Self {
            elem,
            win: (prev, Ok(Element::default()), next),
            _in_count: SLIDER,
            _source: file.clone(),
        };
        // newborn.bump();
        newborn
    }
    pub fn curr(&self, ignore: bool) -> Con<Element> {
        if ignore && self.win.1.clone()?.key().is_space() { self.peek(0, false) } else { self.win.1.clone() }
    }
    pub fn next_vec(&self) -> Vec<Con<Element>> {
        self.win.2.clone()
    }
    pub fn peek(&self, index: usize, ignore: bool) -> Con<Element> { 
        let mut u = if index > SLIDER { SLIDER } else { index };
        if ignore && self.next_vec()[u].clone()?.key().is_space() && u < SLIDER { u += 1 };
        self.next_vec()[u].clone() 
    }
    pub fn prev_vec(&self) -> Vec<Con<Element>> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, index: usize, ignore: bool) -> Con<Element> { 
        let mut u = if index > SLIDER { SLIDER } else { index };
        if ignore && self.next_vec()[u].clone()?.key().is_space() && u < SLIDER { u += 1 };
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
    pub fn jump(&mut self, loops: isize, elem: bool) -> Vod {
        for _ in 0..loops+1 {
            if ( elem && self.curr(false)?.key().is_void() ) && !self.peek(0, true)?.key().is_eof() {
                if let Err(e) = self.bump().unwrap() {
                    self.until_term(true)?;
                    return Err(e);
                };
            }
            if let Err(e) = self.bump().unwrap() {
                self.until_term(true)?;
                return Err(e);
            };
        }
        Ok(())
    }
    pub fn eat(&mut self) {
        if let Ok(e) =  self.curr(false) {
            if matches!(e.key(), KEYWORD::void(_)) && !e.key().is_terminal() { self.bump(); };
        } 
    }
}

impl Iterator for Elements {
    type Item = Con<Element>;
    fn next(&mut self) -> Option<Self::Item> {
        self.bump()
    }
}


/// Creates a iterator that produces tokens from the input string.
pub fn elements(file: &source::Source) -> impl Iterator<Item = Con<Element>>  {
    let mut stg = Box::new(stage1::Elements::init(file));
    let src = file.clone();
    std::iter::from_fn(move || {
        if let Some(v) = stg.bump() {
            match v {
                Ok(el) => {
                    let mut result: Element = el.into();
                    if let Err(err) = result.analyze(&mut stg, &src) {
                        return Some(Err(err));
                    }
                    // println!("{}", result.clone());
                    return Some(Ok(result));
                },
                Err(e) => { return Some(Err(e)); }
            }
        }
        None
    })
}




impl Elements {
    pub fn until_term(&mut self, term: bool) -> Vod {
        while !self.curr(true)?.key().is_eof() {
            if self.curr(false)?.key().is_terminal() {
                if term { self.bump(); }
                break
            }
            self.bump();
        }
        Ok(())
    }
    pub fn until_char(&mut self, el: &str) -> Vod {
        loop{ 
            if self.curr(false)?.con() == &el.to_string() || self.curr(false)?.key().is_eof() {
                break
            }
            self.bump();
        }
        Ok(())
    }
    pub fn until_bracket(&mut self) -> Vod {
        let deep = self.curr(false)?.loc().deep() - 1;
        loop{
            if (self.curr(false)?.key().is_close_bracket() && self.curr(false)?.loc().deep() == deep) 
                || self.curr(false)?.key().is_eof() {
                break
            }
            self.bump();
        }
        self.bump();
        Ok(())
    }
    pub fn debug(&self) -> Vod {
        println!("{}\t{}\t{}", self.curr(false)?.loc(), self.curr(false)?.key(), self.curr(false)?.con());
        Ok(())
    }
}


impl Elements {
    pub fn expect(&self, keyword: KEYWORD, ignore: bool) -> Vod {
        if self.curr(ignore)?.key() == keyword {
            return Ok(())
        };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            key2: keyword,
            src: self._source.clone(),
        }))
    }
    pub fn expect_many(&self, keywords: Vec<KEYWORD>, ignore: bool) -> Vod {
        let currkey = &self.curr(ignore)?.key();
        if let Some(e) = keywords.iter().find(|&x| x == currkey) {
            return Ok(())
        }
        Err( catch!( Typo::ParserManyUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            keys: keywords,
            src: self._source.clone(),
        }))
    }
    pub fn expect_option(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore)?.key(), KEYWORD::option(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            key2: KEYWORD::option(OPTION::ANY),
            src: self._source.clone(),
        }))
    }
    pub fn expect_assign(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore)?.key(), KEYWORD::assign(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            key2: KEYWORD::assign(ASSIGN::ANY), 
            src: self._source.clone(),
        }))
    }
    pub fn expect_types(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore)?.key(), KEYWORD::types(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            key2: KEYWORD::types(TYPE::ANY), 
            src: self._source.clone(),
        }))
    }
    pub fn expect_form(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore)?.key(), KEYWORD::form(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            key2: KEYWORD::form(FORM::ANY), 
            src: self._source.clone(),
        }))
    }
    pub fn expect_literal(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore)?.key(), KEYWORD::literal(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            key2: KEYWORD::literal(LITERAL::ANY), 
            src: self._source.clone(),
        }))
    }
    pub fn expect_buildin(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore)?.key(), KEYWORD::buildin(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            key2: KEYWORD::buildin(BUILDIN::ANY), 
            src: self._source.clone(),
        }))
    }
    pub fn expect_symbol(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore)?.key(), KEYWORD::symbol(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            key2: KEYWORD::symbol(SYMBOL::ANY), 
            src: self._source.clone(),
        }))
    }
    pub fn expect_operator(&self, ignore: bool) -> Vod {
        if matches!(self.curr(ignore)?.key(), KEYWORD::operator(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(ignore)?.loc().clone()), 
            key1: self.curr(ignore)?.key(), 
            key2: KEYWORD::operator(OPERATOR::ANY), 
            src: self._source.clone(),
        }))
    }
    pub fn expect_void(&self) -> Vod {
        if matches!(self.curr(false)?.key(), KEYWORD::void(_)) { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(false)?.loc().clone()), 
            key1: self.curr(false)?.key(), 
            key2: KEYWORD::void(VOID::ANY), 
            src: self._source.clone(),
        }))
    }
    pub fn expect_terminal(&self) -> Vod {
        if self.curr(false)?.key().is_terminal() { return Ok(()) };
        Err( catch!( Typo::ParserUnexpected{ 
            loc: Some(self.curr(false)?.loc().clone()), 
            key1: self.curr(false)?.key(), 
            key2: KEYWORD::void(VOID::ANY), 
            src: self._source.clone(),
        }))
    }
}
