#![allow(dead_code)]
#![allow(unused_macros)]

use std::fmt;
// use crate::scan::scanner;
// use crate::scan::reader;
// use crate::scan::locate;
use crate::scan::token;
use crate::scan::stream;
use crate::error::err;

use crate::scan::scanner::SCAN;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BAG {
    vec: Vec<SCAN>,
    prev: SCAN,
    curr: SCAN,
    brac: Vec<token::SYMBOL>,
}

impl BAG {
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

pub fn init(path: &str, e: &mut err::FLAW) -> BAG {
    let mut stream = stream::STREAM::init(path);
    let mut vec: Vec<SCAN> = Vec::new();
    while !stream.list().is_empty() {
        vec.push(stream.analyze(e).to_owned());
    }
    let curr = vec.get(0).unwrap_or(&stream::zero()).to_owned();
    let prev = curr.to_owned();
    BAG { vec, prev, curr, brac: Vec::new() }
}


#[macro_export]
macro_rules! expect(($e:expr, $p:expr) => (
    match $e {
        $p => { true },
        _ => { false }
    }
));

impl BAG {
    pub fn not_empty(&self) -> bool {
        !self.list().is_empty()
    }
    pub fn bump(&mut self) {
        if self.not_empty(){
            self.prev = self.curr.to_owned();
            self.vec = self.vec[1..].to_vec();
            self.curr = self.vec.get(0).unwrap_or(&stream::zero()).to_owned();
        }
    }
    pub fn jump(&mut self, t: u8) {
        for i in 0..t {
            self.bump()
        }
    }
    pub fn eat(&mut self) {
        if self.curr().key().is_void(){
            self.bump()
        }
    }

    pub fn toend(&mut self) {
        let deep = self.curr().loc().deep();
        loop {
            if (self.is_terminal() && self.curr().loc().deep() <= deep) || (self.curr().key().is_eof()) { break }
            self.bump()
        }
        self.bump();
        self.eat();
    }

    pub fn report(&mut self, s: String, e: &mut err::FLAW) {
        e.report(err::flaw_type::parser, &s, self.curr().loc().clone());
        self.toend();
    }

    pub fn next(&self) -> SCAN {
        self.vec.get(1).unwrap_or(&stream::zero()).to_owned()
    }
    pub fn peek(&self, num: usize) -> SCAN {
        self.vec.get(num).unwrap_or(&stream::zero()).to_owned()
    }

    pub fn is_terminal(&self) -> bool {
        self.curr().key().is_terminal()
    }
}

use crate::scan::token::*;
use crate::scan::token::KEYWORD::*;
impl stream::STREAM {
    pub fn analyze(&mut self, e: &mut err::FLAW) -> SCAN {
        let mut result = self.curr().clone();
        if (self.prev().key().is_void() || self.prev().key().is_bracket()) &&
            self.curr().key().is_symbol() && (self.next().key().is_symbol() || self.next().key().is_void()) {
            if self.after_symbol().is_void() || self.after_symbol().is_bracket() {
                while self.next().key().is_symbol(){
                    result.combine(&self.next());
                    self.bump()
                }
            } else { return result }
            match result.con().as_str() {
                "..." => { result.set_key(operator(OPERATOR::ddd_)) }
                ".." => { result.set_key(operator(OPERATOR::dd_)) }
                "=>" => { result.set_key(operator(OPERATOR::flow_)) }
                "->" => { result.set_key(operator(OPERATOR::flow2_)) }
                "+" => { result.set_key(operator(OPERATOR::add_)) }
                "-" => { result.set_key(operator(OPERATOR::subtract_)) }
                "*" => { result.set_key(operator(OPERATOR::multiply_)) }
                "/" => { result.set_key(operator(OPERATOR::divide_)) }
                "<" => { result.set_key(operator(OPERATOR::less_)) }
                ">" => { result.set_key(operator(OPERATOR::greater_)) }
                "==" => { result.set_key(operator(OPERATOR::equal_)) }
                ">=" => { result.set_key(operator(OPERATOR::greatereq_)) }
                "<=" => { result.set_key(operator(OPERATOR::lesseq_)) }
                "+=" => { result.set_key(operator(OPERATOR::addeq_)) }
                "-=" => { result.set_key(operator(OPERATOR::subtracteq_)) }
                "*=" => { result.set_key(operator(OPERATOR::multiplyeq_)) }
                "/=" => { result.set_key(operator(OPERATOR::divideeq_)) }
                "<<" => { result.set_key(operator(OPERATOR::shiftleft_)) }
                ">>" => { result.set_key(operator(OPERATOR::shiftright_)) }
                _ => {}
            }
        } else if self.curr().key().is_symbol() && self.next().key().is_assign() {
            match result.con().as_str() {
                "~" => { result.set_key(option(OPTION::mut_)) },
                "!" => { result.set_key(option(OPTION::sta_)) },
                "+" => { result.set_key(option(OPTION::exp_)) },
                "-" => { result.set_key(option(OPTION::hid_)) },
                "@" => { result.set_key(option(OPTION::hep_)) },
                _ => { result.set_key(ident) },
            }
        } else if self.curr().key().is_eol() {
            if self.prev().key().is_nonterm() || self.next().key().is_dot() {
                result.set_key(void(VOID::endline_(false)))
            }
        }
        self.bump();
        result
    }

}

impl fmt::Display for BAG {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.curr())
    }
}
