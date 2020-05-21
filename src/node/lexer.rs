#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

use std::fmt;
// use crate::scan::scanner;
// use crate::scan::reader;
use crate::scan::locate;
use crate::scan::token;
use crate::scan::stream;
use crate::error::flaw;

use crate::getset;

use crate::scan::scanner::SCAN;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, GetSet)]
pub struct BAG {
    PAST: Vec<SCAN>,
    NEXT: Vec<SCAN>,
    curr: SCAN,
}

impl BAG {
    pub fn bump(&mut self) {
        if self.not_empty(){
            self.PAST.push(self.curr.to_owned());
            self.NEXT = self.NEXT[1..].to_vec();
            self.curr = self.NEXT.get(0).unwrap_or(&stream::zero()).to_owned();
        }
    }
    pub fn jump(&mut self, t: u8) {
        for i in 0..t { self.bump() }
    }

    //current token
    pub fn curr(&self) -> &SCAN {
        &self.curr
    }
    //current token ignoring space
    pub fn look(&self) -> SCAN {
        if self.curr().key().is_space() { self.next() } else { self.curr().clone() }
    }

    //next th token
    pub fn nth(&self, num: usize) -> SCAN {
        self.NEXT.get(num).unwrap_or(&stream::zero()).to_owned()
    }
    //next token
    pub fn next(&self) -> SCAN {
        self.nth(1)
    }
    //next token ignoring space
    pub fn peek(&self) -> SCAN {
        if self.next().key().is_space() { self.nth(2) } else { self.next() }
    }

    //past th token
    pub fn pth(&self, num: usize) -> SCAN {
        let len = if self.PAST.len() > num { self.PAST.len() - num } else { 0 };
        self.PAST.get(len).unwrap_or(&stream::zero()).to_owned()
    }
    //past token
    pub fn prev(&self) -> SCAN {
        self.pth(1)
    }
    //past token ignoring space
    pub fn past(&self) -> SCAN {
        if self.prev().key().is_space() { self.pth(2) } else { self.prev() }
    }

}

pub fn init(path: &str, e: &mut flaw::FLAW) -> BAG {
    let mut stream = stream::STREAM::init(path);
    let mut NEXT: Vec<SCAN> = Vec::new();
    let PAST: Vec<SCAN> = Vec::new();
    while !stream.list().is_empty() {
        let last = NEXT.last().cloned().unwrap_or(stream::zero()).to_owned();
        NEXT.push(stream.analyze(e, &last).to_owned());
    }
    let curr = NEXT.get(0).unwrap_or(&stream::zero()).to_owned();
    BAG { NEXT, PAST, curr}
}

impl BAG {
    pub fn not_empty(&self) -> bool {
        !self.get_NEXT().is_empty()
    }

    pub fn eat_space(&mut self, e: &mut flaw::FLAW) {
        if self.curr().key().is_space() {
            self.bump()
        }
    }
    pub fn eat_termin(&mut self, e: &mut flaw::FLAW) {
        while self.curr().key().is_terminal() || self.curr().key().is_space() {
            self.bump()
        }
    }

    pub fn to_end(&mut self, e: &mut flaw::FLAW) {
        let deep = self.curr().loc().deep();
        loop {
            if (self.curr().key().is_terminal() && self.curr().loc().deep() <= deep) || (self.curr().key().is_eof()) { break }
            self.bump()
        }
        if self.curr().key().is_terminal() { self.bump() }
    }

    pub fn report(&mut self, s: String, l: locate::LOCATION, e: &mut flaw::FLAW, t: flaw::flaw_type) {
        e.report(t, &s, l);
        self.to_end(e);
    }

    pub fn unexpect_report(&mut self, k: String, e: &mut flaw::FLAW) {
        let s = String::from("expected:") + &k + " but recieved:" + &self.curr().key().to_string();
        self.report(s, self.curr().loc().clone(), e, flaw::flaw_type::parser(flaw::parser::parser_unexpected));
    }
    pub fn missmatch_report(&mut self, k: String, e: &mut flaw::FLAW) {
        let s = String::from("expected:") + &k + " but recieved:" + &self.curr().key().to_string();
        self.report(s, self.curr().loc().clone(), e, flaw::flaw_type::parser(flaw::parser::parser_missmatch));
    }
    pub fn space_rem_report(&mut self, k: String, e: &mut flaw::FLAW) {
        let s = String::from("space between:") + &k + " and:" + &self.curr().key().to_string() + " needs to be removed";
        self.report(s, self.prev().loc().clone(), e, flaw::flaw_type::parser(flaw::parser::parser_space_rem));
    }
    pub fn space_add_report(&mut self, k: String, e: &mut flaw::FLAW) {
        let s = String::from("space between:") + &k + " and:" + &self.curr().key().to_string() + " needs to be added";
        self.report(s, self.prev().loc().clone(), e, flaw::flaw_type::parser(flaw::parser::parser_space_add));
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
    pub fn log2(&self, msg: &str) {
        println!(" {} [{:>2} {:>2}] \t prev:{} \t curr:{} \t next:{}",
            msg,
            self.curr().loc().row(),
            self.curr().loc().col(),
            self.prev().key(),
            self.curr().key(),
            self.next().key())
    }
}

use crate::scan::token::*;
use crate::scan::token::KEYWORD::*;
impl stream::STREAM {
    pub fn analyze(&mut self, e: &mut flaw::FLAW, prev: &SCAN) -> SCAN {
        self.curr().log(">>");
        let mut result = self.curr().clone();
        // EOL to SPACE
        if self.curr().key().is_eol() &&
            ( self.prev().key().is_nonterm() || self.next().key().is_dot() || prev.key().is_operator() )
        {
            result.set_key(void(VOID::space_))
        }

        // numbers
        else if matches!(self.curr().key(), KEYWORD::symbol(SYMBOL::dot_)) && self.next().key().is_number() {
            if self.prev().key().is_void() {
                result = self.make_number(e);
            }
        }
        else if (matches!(self.curr().key(), KEYWORD::symbol(SYMBOL::minus_))  && self.next().key().is_number()) || self.curr().key().is_number() {
            if !self.prev().key().is_void() && matches!(self.curr().key(), KEYWORD::symbol(SYMBOL::minus_)) {
                let key = self.prev().key().clone();
                self.report_space_add(key.to_string(), self.curr().loc().clone(), e);
            } else {
                result = self.make_number(e);
                // self.make_number(e, &mut result);
            }
        }

        // operators
        else if  self.curr().key().is_symbol()
            && ( self.next().key().is_void() || self.next().key().is_symbol() )
            && ( self.prev().key().is_void() || self.prev().key().is_bracket() )
        {
            result = self.make_operator(e);
        }

        // options
        else if self.curr().key().is_symbol()
            && self.next().key().is_assign()
            && (self.prev().key().is_terminal() || self.prev().key().is_eof() || self.prev().key().is_void())
        {
            result = self.make_syoption(e);
        }

        // set key content to indetifier
        else if matches!(self.curr().key(), KEYWORD::ident(_)) {
            result.set_key(ident(self.curr().con().to_string()))
        }

        // check bracket matching
        else if self.curr().key().is_bracket() {
            self.check_bracket_match(e);
        }

        result.log(">>");
        println!("-------------------------------------------------------------------------------------");
        self.bump();
        result
    }

    pub fn make_operator(&mut self, e: &mut flaw::FLAW)  -> SCAN {
        let mut result = self.curr().clone();
            while self.next().key().is_symbol() && !self.next().key().is_bracket() {
                result.combine(&self.next());
                self.bump()
            }
        match result.con().as_str() {
            "=" => { result.set_key(operator(OPERATOR::assign_)) }
            "+" => { result.set_key(operator(OPERATOR::add_)) }
            "-" => { result.set_key(operator(OPERATOR::subtract_)) }
            "*" => { result.set_key(operator(OPERATOR::multiply_)) }
            "/" => { result.set_key(operator(OPERATOR::divide_)) }
            "<" => { result.set_key(operator(OPERATOR::less_)) }
            ">" => { result.set_key(operator(OPERATOR::greater_)) }
            ":=" => { result.set_key(operator(OPERATOR::assign2_)) }
            "..." => { result.set_key(operator(OPERATOR::ddd_)) }
            ".." => { result.set_key(operator(OPERATOR::dd_)) }
            "=>" => { result.set_key(operator(OPERATOR::flow_)) }
            "->" => { result.set_key(operator(OPERATOR::flow2_)) }
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
        result
    }
    pub fn make_number(&mut self, e: &mut flaw::FLAW) -> SCAN {
        let mut result = self.curr().clone();
        if self.curr().key().is_dot() && self.next().key().is_decimal() {
            result.set_key(literal(LITERAL::float_));
            result.combine(&self.next());
            self.bump();
            // println!("{} {} {} {}", self.next().key(), self.nth(2).key().is_void(), self.nth(3).key().is_ident(), self.nth(4).key());
            if self.next().key().is_dot() && self.nth(2).key().is_eol() && self.nth(3).key().is_ident() {
                return result
            } else if self.next().key().is_dot() && !self.nth(2).key().is_ident() {
                self.bump();
                self.report_primitive_acccess(" flt ".to_string(), self.next().loc().clone(), e);
            }
        } else if self.prev().key().is_continue() && self.curr().key().is_decimal() && self.next().key().is_dot() && !self.nth(2).key().is_ident() {
            result.set_key(literal(LITERAL::float_));
            result.combine(&self.next());
            self.bump();
            if self.next().key().is_number() {
                result.combine(&self.next());
                self.bump();
                if self.next().key().is_dot() && self.nth(2).key().is_number() {
                    self.bump();
                    self.report_primitive_acccess(" flt ".to_string(), self.next().loc().clone(), e);
                }
            } else if !self.next().key().is_void() {
                self.bump();
                self.report_space_add(self.prev().key().to_string(), self.next().loc().clone(), e);
            }
        }
        result
    }
    pub fn make_syoption(&mut self, e: &mut flaw::FLAW)  -> SCAN {
        let mut result = self.curr().clone();
        match result.con().as_str() {
            "~" => { result.set_key(option(OPTION::mut_)) },
            "!" => { result.set_key(option(OPTION::sta_)) },
            "+" => { result.set_key(option(OPTION::exp_)) },
            "-" => { result.set_key(option(OPTION::hid_)) },
            "@" => { result.set_key(option(OPTION::hep_)) },
            _ => {},
        }
        result
    }
    pub fn check_bracket_match(&mut self, e: &mut flaw::FLAW) {
        let loc = self.curr().loc().clone();
        let key = self.curr().key().clone();
        if self.curr().key().is_open_bracket() {
            self.bracs().push((loc, key))
        } else if self.curr().key().is_close_bracket() {
            if ( matches!(self.curr().key(), KEYWORD::symbol(SYMBOL::roundC_))
                && matches!(self.bracs().last().unwrap_or(&(loc.clone(), KEYWORD::illegal)).1, KEYWORD::symbol(SYMBOL::roundO_)) )
                || ( matches!(self.curr().key(), KEYWORD::symbol(SYMBOL::squarC_))
                && matches!(self.bracs().last().unwrap_or(&(loc.clone(), KEYWORD::illegal)).1, KEYWORD::symbol(SYMBOL::squarO_)) )
                || ( matches!(self.curr().key(), KEYWORD::symbol(SYMBOL::curlyC_))
                && matches!(self.bracs().last().unwrap_or(&(loc.clone(), KEYWORD::illegal)).1, KEYWORD::symbol(SYMBOL::curlyO_)) )
            {
                self.bracs().pop();
            } else {
                let key = match self.bracs().last().unwrap_or(&(loc.clone(), KEYWORD::illegal)).1 {
                    KEYWORD::symbol(SYMBOL::curlyO_) => { KEYWORD::symbol(SYMBOL::curlyC_) },
                    KEYWORD::symbol(SYMBOL::squarO_) => { KEYWORD::symbol(SYMBOL::squarC_) },
                    KEYWORD::symbol(SYMBOL::roundO_) => { KEYWORD::symbol(SYMBOL::roundC_) },
                    _ => { KEYWORD::illegal }
                };
                self.report_bracket(key.to_string(), self.curr().loc().clone(), e);
            }
        }
    }
}

impl fmt::Display for BAG {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.curr())
    }
}
