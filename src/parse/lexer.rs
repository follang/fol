#![allow(dead_code)]

use std::fmt;
// use crate::scan::scanner;
// use crate::scan::reader;
// use crate::scan::token;
// use crate::scan::locate;
use crate::scan::stream;

use crate::scan::scanner::SCAN;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LEXEME {
    vec: Vec<SCAN>,
    prev: SCAN,
    curr: SCAN,
}

impl LEXEME {
    pub fn list(&self) -> &Vec<SCAN> {
        &self.vec
    }
    pub fn curr(&self) -> &SCAN {
        &self.curr
    }
    pub fn prev(&self) -> &SCAN {
        &self.prev
    }
}

impl LEXEME {
    pub fn init(path: &str ) -> Self {
        let mut stream = stream::STREAM::init(path);
        let prev = stream.prev().to_owned();
        let curr = stream.curr().to_owned();
        let mut vec: Vec<SCAN> = Vec::new();
        while !stream.list().is_empty() {
            vec.push(stream.analyze().to_owned());
            stream.bump();
        }
        LEXEME { vec, prev, curr }
    }

    pub fn analyze(&mut self, s: &mut stream::STREAM) -> SCAN {
        s.curr().to_owned()
    }

    pub fn bump(&mut self) {
        if !self.vec.is_empty(){
            self.prev = self.curr.to_owned();
            self.vec = self.vec[1..].to_vec();
            self.curr = self.vec.get(0).unwrap_or(&stream::zero()).to_owned();
        }
    }
    pub fn next(&self) -> SCAN {
        self.vec.get(1).unwrap_or(&stream::zero()).to_owned()
    }
    pub fn nth(&self, num: usize) -> SCAN {
        self.vec.get(num).unwrap_or(&stream::zero()).to_owned()
    }
}

impl stream::STREAM {
    pub fn analyze(&mut self) -> &SCAN {
        self.curr()
    }
}

impl fmt::Display for LEXEME {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.curr())
    }
}

