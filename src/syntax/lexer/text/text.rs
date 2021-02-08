use std::fmt;
use std::str::Chars;
use crate::types::*;
use crate::syntax::point;
use crate::syntax::token::help::*;
use crate::syntax::index::*;
use crate::syntax::lexer::text::reader;

type Part<T> = (T, point::Location);

pub struct Text {
    chars: Box<dyn Iterator<Item = Con<Part<char>>>>,
    win: Win<Con<Part<char>>>,
    _in_count: usize,
}

impl Text {
    pub fn curr(&self) -> Con<Part<char>> {
        self.win.1.clone()
    }
    ///next vector
    pub fn next_vec(&self) -> Vec<Con<Part<char>>> {
        self.win.2.clone()
    }
    pub fn peek(&self, index: usize) -> Con<Part<char>> { 
        let u = if index > SLIDER { 0 } else { index };
        self.next_vec()[u].clone() 
    }
    ///prev vector
    pub fn prev_vec(&self) -> Vec<Con<Part<char>>> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, index: usize) -> Con<Part<char>> { 
        let u = if index > SLIDER { 0 } else { index };
        self.prev_vec()[u].clone() 
    }

    pub fn init(file: &source::Source) -> Self {
        let initerr: Con<Part<char>> = Err(Box::new(Flaw::InitError{ msg: None }));
        let enderr: Con<Part<char>> = Err(Box::new(Flaw::EndError{ msg: None }));
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut chars = Box::new(gen(file));
        for _ in 0..SLIDER { prev.push(initerr.clone()) }
        for _ in 0..SLIDER { next.push(chars.next().unwrap_or(enderr.clone())) }
        Self {
            chars,
            win: (prev, initerr, next),
            _in_count: SLIDER
        }
    }

    pub fn bump(&mut self) -> Option<Con<Part<char>>> {
        match self.chars.next() {
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
                    let enderr: Con<Part<char>> = Err(Box::new(Flaw::EndError{ msg: None }));
                    // TODO: Handle better .ok()
                    self.win.0.remove(0).ok(); self.win.0.push(self.win.1.clone());
                    self.win.1 = self.win.2[0].clone();
                    // TODO: Handle better .ok()
                    self.win.2.remove(0).ok(); self.win.2.push(enderr);
                    self._in_count -= 1;
                    return Some(self.win.1.clone());
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
        if let Ok(ok) = self.win.1.clone() {
            write!(f, "{} {}", self.win.1.clone().unwrap().1.show(), self.win.1.clone().unwrap().0)
        } else {
            write!(f, "ERROR")
        }
    }
}


pub fn gen(file: &source::Source) -> impl Iterator<Item = Con<Part<char>>> {
    let mut lines = get_lines(file);
    let mut chars = get_chars(lines.next().unwrap());
    let mut loc = point::Location::init(
        (file.path(true), file.path(false)), 
        &file.module()
    );
    loc.adjust(1,0);
    let mut last_eol = false;
    std::iter::from_fn(move || {
        match chars.next() {
            Some(i) => {
                loc.new_char();
                if is_open_bracket(&i) { loc.deepen() }
                else if is_close_bracket(&i) { loc.soften() }
                // if i == ' ' { loc.new_word() }
                return Some (Ok((i, loc.clone())))
            },
            None => {
                match lines.next() {
                    Some(j) => { 
                        loc.new_line();
                        loc.new_word();
                        chars = get_chars(j);
                        return Some(Ok((chars.next().unwrap_or('\n'), loc.clone())))
                    },
                    None => {
                        if !last_eol {
                            last_eol = true;
                            return Some(Ok(('\0', loc.clone())));
                        }
                        return None
                    }
                }
            }
        };
    })
}


fn get_lines(src: &source::Source) -> impl Iterator<Item = String> {
    let mut reader = reader::BufReader::open(src.path(true)).unwrap();
    let mut buffer = String::new();
    std::iter::from_fn(move || {
        if let Some(line) = reader.read_line(&mut buffer) {
            return Some(line.unwrap().clone());
        }
        None
    })
}


fn get_chars(src: String) -> impl Iterator<Item = char> {
    let mut chrs = src.clone();
    std::iter::from_fn(move || {
        if let Some(ch) =  chrs.chars().next() {
            chrs.remove(0);
            return Some(ch.clone()) 
        };
        None
    })
}
