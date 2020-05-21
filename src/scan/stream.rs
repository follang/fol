#![allow(dead_code)]

use std::fmt;
use crate::scan::scanner;
use crate::scan::reader;
use crate::scan::token;
use crate::scan::locate;
use crate::error::flaw;

use crate::scan::scanner::SCAN;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct STREAM {
    vec: Vec<SCAN>,
    prev: SCAN,
    curr: SCAN,
    bracs: Vec<(locate::LOCATION, token::KEYWORD)>
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

    pub fn bracs(&mut self) -> &mut Vec<(locate::LOCATION, token::KEYWORD)> {
        &mut self.bracs
    }
}

impl STREAM {
    pub fn init(path: &str) -> Self {
        let mut vec: Vec<SCAN> = Vec::new();
        let bracs: Vec<(locate::LOCATION, token::KEYWORD)> = Vec::new();
        for mut e in reader::iteratize(path) {
            vec.append(&mut scanner::vectorize(&mut e))
        }
        let prev = SCAN::zero(&vec.last().unwrap().loc().name());
        let curr = vec.get(0).unwrap_or(&zero()).to_owned();
        STREAM { vec, prev, curr, bracs }
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

    pub fn to_endline(&mut self) {
        let deep = self.curr().loc().deep();
        loop {
            if (self.curr().key().is_terminal() && self.curr().loc().deep() <= deep) || (self.curr().key().is_eof()) { break }
            self.bump()
        }
        if self.curr().key().is_terminal() { self.bump() }
    }
    pub fn to_endsym(&mut self) {
        while !self.curr().key().is_void() {
            self.bump()
        }
    }

    pub fn report(&mut self, s: String, l: locate::LOCATION, e: &mut flaw::FLAW, t: flaw::flaw_type) {
        e.report(t, &s, l);
        self.to_endsym();
    }

    pub fn unexpect_report(&mut self, k: String, e: &mut flaw::FLAW) {
        let s = String::from("expected:") + &k + " but recieved:" + &self.curr().key().to_string();
        self.report(s, self.curr().loc().clone(), e, flaw::flaw_type::lexer(flaw::lexer::lexer_bracket_unmatch));
    }

    pub fn log(&self, msg: &str) {
        println!(" {} [{:>2} {:>2}] \t prev:{} \t curr:{} \t next:{}",
            msg,
            self.curr().loc().row(),
            self.curr().loc().col(),
            self.prev().key(),
            self.curr().key(),
            self.next().key())
    }
    pub fn log2(&self, msg: &str) {
        println!(" {} [{:>2} {:>2}] \t \t {:<30} {:>20}",
            msg,
            self.curr().loc().row(),
            self.curr().loc().col(),
            self.curr().key(),
            self.curr().con())
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
