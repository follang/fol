#![allow(dead_code)]
#![allow(unused_macros)]

use std::fmt;
// use crate::scan::scanner;
// use crate::scan::reader;
use crate::scan::locate;
use crate::scan::token;
use crate::scan::stream;
use crate::error::err;

use crate::getset;

use crate::scan::scanner::SCAN;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, GetSet)]
pub struct BAG {
    vec: Vec<SCAN>,
    prev: SCAN,
    past: SCAN,
    curr: SCAN,
}

impl BAG {
    pub fn bump(&mut self) {
        if self.not_empty(){
            self.prev = self.curr.to_owned();
            if !self.curr.key().is_void() { self.past = self.curr.to_owned() };
            self.vec = self.vec[1..].to_vec();
            self.curr = self.vec.get(0).unwrap_or(&stream::zero()).to_owned();
        }
    }
    pub fn jump(&mut self, t: u8) {
        for i in 0..t { self.bump() }
    }

    //past token
    pub fn prev(&self) -> &SCAN {
        &self.prev
    }
    //past token ignoring space
    pub fn past(&self) -> &SCAN {
        &self.past
    }
    //current token
    pub fn curr(&self) -> &SCAN {
        &self.curr
    }
    //current token ignoring space
    pub fn look(&self) -> SCAN {
        if self.curr().key().is_space() { self.next() } else { self.curr().clone() }
    }
    //next token
    pub fn next(&self) -> SCAN {
        self.vec.get(1).unwrap_or(&stream::zero()).to_owned()
    }
    //next token ignoring space
    pub fn peek(&self) -> SCAN {
        if self.next().key().is_space() { self.nth(2) } else { self.next() }
    }
    //nth token
    pub fn nth(&self, num: usize) -> SCAN {
        self.vec.get(num).unwrap_or(&stream::zero()).to_owned()
    }

}

pub fn init(path: &str, e: &mut err::FLAW) -> BAG {
    let mut stream = stream::STREAM::init(path);
    let mut vec: Vec<SCAN> = Vec::new();
    while !stream.list().is_empty() {
        let last = vec.last().cloned().unwrap_or(stream::zero()).to_owned();
        vec.push(stream.analyze(e, &last).to_owned());
    }
    let curr = vec.get(0).unwrap_or(&stream::zero()).to_owned();
    let prev = SCAN::zero("");
    let past = SCAN::zero("");
    BAG { vec, prev, curr, past }
}

impl BAG {
    pub fn not_empty(&self) -> bool {
        !self.get_vec().is_empty()
    }

    pub fn eat_space(&mut self, e: &mut err::FLAW) {
        if self.curr().key().is_space() {
            self.bump()
        }
    }
    pub fn eat_termin(&mut self, e: &mut err::FLAW) {
        while self.curr().key().is_terminal() || self.curr().key().is_space() {
            self.bump()
        }
    }

    pub fn to_end(&mut self, e: &mut err::FLAW) {
        let deep = self.curr().loc().deep();
        loop {
            if (self.curr().key().is_terminal() && self.curr().loc().deep() <= deep) || (self.curr().key().is_eof()) { break }
            self.bump()
        }
        if self.curr().key().is_terminal() { self.bump() }
    }

    pub fn report(&mut self, s: String, l: locate::LOCATION, e: &mut err::FLAW, t: err::flaw_type) {
        e.report(t, &s, l);
        self.to_end(e);
    }

    pub fn unexpect_report(&mut self, k: String, e: &mut err::FLAW) {
        let s = String::from("expected: ") + &k + " but recieved: " + &self.curr().key().to_string();
        self.report(s, self.curr().loc().clone(), e, err::flaw_type::parser_unexpected);
    }
    pub fn missmatch_report(&mut self, k: String, e: &mut err::FLAW) {
        let s = String::from("expected: ") + &k + " but recieved: " + &self.curr().key().to_string();
        self.report(s, self.curr().loc().clone(), e, err::flaw_type::parser_missmatch);
    }
    pub fn separator_report(&mut self, k: String, e: &mut err::FLAW) {
        let s = String::from("space between: ") + &k + " and: " + &self.curr().key().to_string() + " nedds to be removed";
        self.report(s, self.prev().loc().clone(), e, err::flaw_type::parser_indentation);
    }

    pub fn match_bracket(&self, k: KEYWORD, d: isize) -> bool {
        if (matches!(self.curr().key(), k) && self.curr().loc().deep() == d) || self.curr().key().is_eof() { true } else { false }
    }

    pub fn log(&self, msg: &str) {
        println!(" {} [{:>2} {:>2}] \t past:{} \t prev:{} \t curr:{} \t next:{} \t peek:{}",
            msg,
            self.curr().loc().row(),
            self.curr().loc().col(),
            self.past().key(),
            self.prev().key(),
            self.curr().key(),
            self.next().key(),
            self.peek().key());
    }
}

use crate::scan::token::*;
use crate::scan::token::KEYWORD::*;
impl stream::STREAM {
    pub fn analyze(&mut self, e: &mut err::FLAW, p: &SCAN) -> SCAN {
        let mut result = self.curr().clone();

        if self.curr().key().is_eol() &&
            (self.prev().key().is_nonterm() || self.next().key().is_dot() || p.key().is_operator()) {
            result.set_key(void(VOID::space_))
        } else if self.curr().key().is_symbol()
            && (self.next().key().is_void() || self.next().key().is_symbol())
            && (self.prev().key().is_void() || self.prev().key().is_bracket())
        {
            if self.after_symbol().is_void() || self.after_symbol().is_bracket() {
                while self.next().key().is_symbol(){
                    result.combine(&self.next());
                    self.bump()
                }
            } else { self.bump(); return result }
            match result.con().as_str() {
                "=" => { result.set_key(operator(OPERATOR::assign_)) }
                ":=" => { result.set_key(operator(OPERATOR::assign2_)) }
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
                "!=" => { result.set_key(operator(OPERATOR::noteq_)) }
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
        } else if self.curr().key().is_symbol()
            && self.next().key().is_assign()
            && (self.prev().key().is_terminal() || self.prev().key().is_eof() || self.prev().key().is_void())
        {
            match result.con().as_str() {
                "~" => { result.set_key(option(OPTION::mut_)) },
                "!" => { result.set_key(option(OPTION::sta_)) },
                "+" => { result.set_key(option(OPTION::exp_)) },
                "-" => { result.set_key(option(OPTION::hid_)) },
                "@" => { result.set_key(option(OPTION::hep_)) },
                _ => {},
            }
        }
        if matches!(self.curr().key(), KEYWORD::ident(_)) {
            result.set_key(ident(self.curr().con().to_string()))
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
