#![allow(dead_code)]

use std::fmt;
use colored::Colorize;
use crate::syntax::point;
use crate::syntax::token;
use crate::syntax::scan::source;
use crate::syntax::scan::element;

use crate::syntax::scan::element::Element;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stream {
    vec: Vec<Element>,
    prev: Element,
    curr: Element,
    bracs: Vec<(point::Location, token::KEYWORD)>,
}

impl Stream {
    pub fn list(&self) -> &Vec<Element> {
        &self.vec
    }
    pub fn curr(&self) -> &Element {
        &self.curr
    }
    pub fn prev(&self) -> &Element {
        &self.prev
    }

    pub fn bracs(&mut self) -> &mut Vec<(point::Location, token::KEYWORD)> {
        &mut self.bracs
    }
}

impl Stream {
    pub fn init(path: &str) -> Self {
        let mut vec: Vec<Element> = Vec::new();
        let bracs: Vec<(point::Location, token::KEYWORD)> = Vec::new();
        for src in source::sources(path) {
            vec.extend(element::elements(&src))
        }
        let prev = Element::zero(&vec.last().unwrap().loc().name());
        let curr = vec.get(0).unwrap_or(&Element::zero(" ")).to_owned();
        Stream {
            vec,
            prev,
            curr,
            bracs,
        }
    }

    pub fn bump(&mut self) {
        if !self.vec.is_empty() {
            self.prev = self.curr.to_owned();
            self.vec = self.vec[1..].to_vec();
            self.curr = self.vec.get(0).unwrap_or(&Element::zero(" ")).to_owned();
            // let curr = vec.remove(0);
        }
    }
    pub fn nth(&self, num: usize) -> Element {
        self.vec.get(num).unwrap_or(&Element::zero(" ")).to_owned()
    }
    pub fn next(&self) -> Element {
        self.nth(1)
    }
    pub fn peek(&self) -> Element {
        if self.next().key().is_space() {
            self.nth(2)
        } else {
            self.next()
        }
    }
    pub fn seek(&self) -> Element {
        if self.nth(2).key().is_space() {
            self.nth(3)
        } else {
            self.nth(2)
        }
    }

    pub fn after(&self) -> token::KEYWORD {
        let mut i = 1;
        while self.nth(i).key().is_symbol() {
            i += 1;
        }
        self.nth(i).key().clone()
    }

    pub fn to_endline(&mut self) {
        let deep = self.curr().loc().deep();
        loop {
            if (self.curr().key().is_terminal() && self.curr().loc().deep() <= deep)
                || (self.curr().key().is_eof())
            {
                break;
            }
            self.bump()
        }
        if self.curr().key().is_terminal() {
            self.bump()
        }
    }
    pub fn to_endsym(&mut self) {
        while !self.curr().key().is_void() {
            self.bump()
        }
    }

    pub fn log(&self, msg: &str) {
        println!(
            " {} [{:>2} {:>2}] \t prev:{} \t curr:{} \t next:{}",
            msg,
            self.curr().loc().row(),
            self.curr().loc().col(),
            self.prev().key(),
            self.curr().key(),
            self.next().key()
        )
    }
    pub fn log2(&self, msg: &str) {
        println!(
            " {} [{:>2} {:>2}] \t \t {:<30} {:>20}",
            msg,
            self.curr().loc().row(),
            self.curr().loc().col(),
            self.curr().key(),
            self.curr().con()
        )
    }
}

impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.curr())
    }
}

pub fn zero() -> Element {
    Element::zero(" ")
}
