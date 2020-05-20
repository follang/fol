#![allow(dead_code)]

use std::fmt;
use crate::scan::scanner;
use crate::scan::reader;
use crate::scan::token;

use crate::scan::scanner::SCAN;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct STREAM {
    vec: Vec<SCAN>,
    prev: SCAN,
    curr: SCAN,
}

impl STREAM {
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

impl STREAM {
    pub fn init(path: &str) -> Self {
        let mut vec: Vec<SCAN> = Vec::new();
        for mut e in reader::iteratize(path) {
            vec.append(&mut scanner::vectorize(&mut e))
        }
        let prev = SCAN::zero(&vec.last().unwrap().loc().name());
        let curr = vec.get(0).unwrap_or(&zero()).to_owned();
        STREAM { vec, prev, curr }
    }

    pub fn bump(&mut self) {
        if !self.vec.is_empty(){
            self.prev = self.curr.to_owned();
            self.vec = self.vec[1..].to_vec();
            self.curr = self.vec.get(0).unwrap_or(&zero()).to_owned();
            // let curr = vec.remove(0);
        }
    }
    pub fn next(&self) -> SCAN {
        self.vec.get(1).unwrap_or(&zero()).to_owned()
    }
    pub fn nth(&self, num: usize) -> SCAN {
        self.vec.get(num).unwrap_or(&zero()).to_owned()
    }

    pub fn after_symbol(&self) -> token::KEYWORD {
        let mut i = 1;
        while self.nth(i).key().is_symbol() { i += 1; }
        self.nth(i).key().clone()
    }

    pub fn after(&self, key: token::KEYWORD) -> token::KEYWORD {
        let mut i = 1;
        while matches!( self.nth(i).key(), key) { i += 1; }
        self.nth(i).key().clone()
    }
}

impl fmt::Display for STREAM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.curr())
    }
}

pub fn zero() -> SCAN {
    SCAN::zero(" ")
}
