#![allow(dead_code)]

use std::fmt;
use std::iter;
use colored::Colorize;
use crate::syntax::point;
use crate::syntax::token;
use crate::syntax::scan::stage1::{Element, elements};
use crate::syntax::scan::stage1;

use crate::syntax::error::*;

const SLIDER: usize = 9;
pub struct Stream<'a> {
    elm: &'a mut stage1::Elements<'a>,
    win: (Vec<Element>, Element, Vec<Element>),
    _in_count: usize,
}

impl<'a> Stream<'a> {
    pub fn init(mut txt: &'static stage1::Elements<'a>) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut elm = Box::new(stage1::Elements::init(&mut txt));
        for _ in 0..SLIDER { prev.push(Element::default()) }
        for _ in 0..SLIDER { next.push(elm.next().unwrap()) }
        Self {
            elm,
            win: ( prev, Element::default(), next ),
            _in_count: SLIDER }
    }
    pub fn curr(&self) -> Element {
        self.win.1.clone()
    }
    pub fn next_vec(&self) -> Vec<Element> {
        self.win.2.clone()
    }
    pub fn next(&self) -> Element { 
        self.next_vec()[0].clone() 
    }
    pub fn prev_vec(&self) -> Vec<Element> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn prev(&self) -> Element { 
        self.prev_vec()[0].clone() 
    }
    pub fn bump(&mut self) -> Opt<Element> {
        match self.elm.next() {
            Some(v) => {
                self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                self.win.1 = self.win.2[0].clone();
                self.win.2.remove(0); self.win.2.push(v);
                return Some(self.win.1.clone())
            },
            None => {
                if self._in_count > 1 {
                    self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                    self.win.1 = self.win.2[0].clone();
                    self.win.2.remove(0); self.win.2.push(Element::default());
                    self._in_count -= 1;
                    return Some(self.win.1.clone())
                } else { return None }
            }
        }
    }
}

impl fmt::Display for Stream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.win.1)
    }
}
