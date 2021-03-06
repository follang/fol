use crate::types::*;
use crate::syntax::token::*;
use crate::syntax::lexer::stage2;
use crate::syntax::lexer::stage3::Element;
use crate::syntax::index;


pub struct Elements {
    elem: Box<dyn Iterator<Item = Con<Element>>>,
    win: Win<Con<Element>>,
    _in_count: usize,
}


impl Elements {
    pub fn default(&self) -> Element { Element::default() }
    pub fn init(file: &index::Input) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut elem = Box::new(elements(file));
        for _ in 0..SLIDER { prev.push(Ok(Element::default())) }
        for _ in 0..SLIDER { next.push(elem.next().unwrap_or(Ok(Element::default()))) }
        let newborn = Self {
            elem,
            win: (prev, Ok(Element::default()), next),
            _in_count: SLIDER,
        };
        newborn
    }
    pub fn set_key(&mut self, key: KEYWORD) -> Vod {
        if self.win.1.clone()?.key().is_space() || self.win.2[0].is_ok() { self.bump(); }
        if let Ok(a) = &mut self.win.1 {
            a.set_key(key);
        }
        Ok(())
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
            if !self.curr(false)?.key().is_eof(){
                if let Err(e) = self.bump().unwrap() {
                    self.until_term(true)?;
                    return Err(e);
                };
            }
        }
        Ok(())
    }
    pub fn eat(&mut self) {
        if let Ok(e) =  self.curr(false) {
            if matches!(e.key(), KEYWORD::Void(_)) && !e.key().is_terminal() { self.bump(); };
        } 
    }
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

    pub fn debug(&self, bol: bool, ln: usize) -> Vod {
        if self.curr(bol)?.loc().row() == ln || ln == 0 {
            println!("{}\t{}\t{}", self.curr(bol)?.loc(), self.curr(bol)?.key(), self.curr(bol)?.con());
        }
        Ok(())
    }
    pub fn window(&self, bol: bool, ln: usize) -> Vod {
        if self.curr(bol)?.loc().row() == ln || ln == 0 {
            println!("----\nseek: {}\ncurr: {}\npeek: {}", self.seek(0, bol)?, self.curr(bol)?, self.peek(0, bol)?);
        }
        Ok(())
    }
}

impl Iterator for Elements {
    type Item = Con<Element>;
    fn next(&mut self) -> Option<Self::Item> {
        self.bump()
    }
}


/// Creates a iterator that produces tokens from the input string.
pub fn elements(file: &index::Input) -> impl Iterator<Item = Con<Element>>  {
    let mut stg = Box::new(stage2::Elements::init(file));
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
