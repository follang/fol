#![allow(dead_code)]

use std::fmt;
use std::iter;
use colored::Colorize;
use crate::syntax::point;
use crate::syntax::token;
use crate::syntax::scan::source;
use crate::syntax::scan::stage1;

use crate::syntax::error::*;

const SLIDER: usize = 9;
pub struct Elements {
    src: Box<dyn Iterator<Item = source::Source>>,
    elm: Box<dyn Iterator<Item = stage1::Element>>,
    win: (Vec<stage1::Element>, stage1::Element, Vec<stage1::Element>),
    _in_count: usize,
}

impl Elements {
    pub fn elements(self) -> Box<dyn Iterator<Item = stage1::Element>> {
        self.elm
    }
    pub fn sources(self) -> Box<dyn Iterator<Item = source::Source>> {
        self.src
    }
}

impl Elements {
    pub fn init(path: &'static str) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut src = Box::new(source::sources(&path));
        let mut elm = Box::new(stage1::elements(src.next().unwrap()));
        for _ in 0..SLIDER { prev.push(stage1::Element::default()) }
        for _ in 0..SLIDER { next.push(elm.next().unwrap()) }
        Elements {
            src,
            elm,
            win: ( prev, stage1::Element::default(), next ),
            _in_count: SLIDER }
    }
    pub fn bump(&mut self) -> Opt<stage1::Element> {
        match self.elm.next() {
            Some(v) => {
                self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                self.win.1 = self.win.2[0].clone();
                self.win.2.remove(0); self.win.2.push(v);
                return Some(self.win.1.clone())
            },
            None => {
                match self.src.next() {
                    Some(v) => {
                        self.elm = Box::new(stage1::elements(v));
                        if let Some(u) = self.elm.next() {
                            self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                            self.win.1 = self.win.2[0].clone();
                            self.win.2.remove(0); self.win.2.push(u);
                            return Some(self.win.1.clone())
                        }
                        None
                    }
                    None => {
                        if self._in_count > 1 {
                            self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                            self.win.1 = self.win.2[0].clone();
                            self.win.2.remove(0); self.win.2.push(stage1::Element::default());
                            self._in_count -= 1;
                            return Some(self.win.1.clone())
                        } else { return None }
                    }
                }
            }
        }
    }
}

impl fmt::Display for Elements {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.win.1)
    }
}
