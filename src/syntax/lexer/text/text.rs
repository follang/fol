#![allow(dead_code)]

use std::fmt;
use std::str::Chars;
use crate::syntax::point;
use crate::syntax::lexer::source;
use crate::syntax::lexer::text::reader;
use crate::types::{Con, Win, SLIDER};

type Part<T> = (T, point::Location);

pub struct Text {
    chars: Box<dyn Iterator<Item = Con<Part<char>>>>,
    win: Win<Part<char>>,
    _in_count: usize,
}

impl Text {
    pub fn curr(&self) -> Part<char> {
        self.win.1.clone()
    }
    ///next vector
    pub fn next_vec(&self) -> Vec<Part<char>> {
        self.win.2.clone()
    }
    pub fn peek(&self, index: usize) -> Part<char> { 
        let u = if index > SLIDER { 0 } else { index };
        self.next_vec()[u].clone() 
    }
    ///prev vector
    pub fn prev_vec(&self) -> Vec<Part<char>> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, index: usize) -> Part<char> { 
        let u = if index > SLIDER { 0 } else { index };
        self.prev_vec()[u].clone() 
    }

    pub fn init(dir: String) -> Self {
        let def: Part<char> = ('\0', point::Location::default());
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut chars = Box::new(gen(dir));
        for _ in 0..SLIDER { prev.push(def.clone()) }
        for _ in 0..SLIDER { next.push(chars.next().unwrap_or(Ok(def.clone())).unwrap()) }
        Self {
            chars,
            win: (prev, def, next),
            _in_count: SLIDER
        }
    }

    pub fn bump(&mut self) -> Option<Con<Part<char>>> {
        match self.chars.next() {
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
                    self.win.2.remove(0); self.win.2.push(('\0', point::Location::default()));
                    self._in_count -= 1;
                    return Some(Ok(self.win.1.clone()));
                } else { return None }
            }
        }
    }
}

impl Iterator for Text {
    type Item = Part<char>;
    fn next(&mut self) -> Option<Part<char>> {
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


impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.win.1.1, self.win.1.0)
    }
}


pub fn gen(path: String) -> impl Iterator<Item = Con<Part<char>>> {
    let mut s = source::Sources::init(path);
    let mut l = lines(s.next().unwrap());
    let mut c = chars(l.next().unwrap());
    let mut loc = point::Location::init(
        (s.curr().path(true), s.curr().path(false)), 
        &s.curr().module()
    );
    loc.adjust(1,0);
    std::iter::from_fn(move || {
        loop {
            match c.next() {
                Some(i) => {
                    loc.new_char();
                    // if i == ' ' { loc.new_word() }
                    return Some (Ok((i, loc.clone())))
                },
                None => {
                    match l.next() {
                        Some(j) => { 
                            loc.new_line();
                            loc.new_word();
                            c = chars(j);
                            return Some(Ok((c.next().unwrap_or('\n'), loc.clone())))
                        },
                        None => {
                            match s.bump() {
                                Some(k) => {
                                    loc = point::Location::init(
                                        (s.curr().path(true), s.curr().path(false)), 
                                        &s.curr().module()
                                    );
                                    l = lines(k);
                                },
                                None => {
                                    return None
                                }
                            }
                        }
                    }
                }
            };
        }
    })
}


fn lines(src: source::Source) -> impl Iterator<Item = String> {
    let mut reader = reader::BufReader::open(src.path(true)).unwrap();
    let mut buffer = String::new();
    std::iter::from_fn(move || {
        if let Some(line) = reader.read_line(&mut buffer) {
            return Some(line.unwrap().clone());
        }
        None
    })
}


fn chars(src: String) -> impl Iterator<Item = char> {
    let mut chrs = src.clone();
    std::iter::from_fn(move || {
        if let Some(ch) =  chrs.chars().next() {
            chrs.remove(0);
            return Some(ch.clone()) 
        };
        None
    })
}
