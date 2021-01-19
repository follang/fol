#![allow(dead_code)]

use std::fmt;
use std::iter;
use colored::Colorize;
use crate::syntax::point;
use crate::syntax::token;
use crate::syntax::scan::source;
use crate::syntax::scan::stage1;

use crate::syntax::error::*;

// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
//
const SLIDER: usize = 9;
pub struct Elements {
    src: Box<dyn Iterator<Item = source::Source>>,
    elm: Box<dyn Iterator<Item = stage1::Element>>,
    win: Vec<stage1::Element>,
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
        let mut win = Vec::with_capacity(SLIDER);
        let mut src = Box::new(source::sources(&path));
        let mut elm = Box::new(stage1::elements(src.next().unwrap()));
        for _ in 0..SLIDER { win.push(elm.next().unwrap()) }
        Elements { src, elm, win }
    }

    pub fn bump(&mut self) -> Opt<Vec<stage1::Element>> {
        match self.elm.next() {
            Some(v) => {
                self.win.remove(0);
                self.win.push(v);
                return Some(self.win.clone())
            },
            None => {
                if let Some(v) = self.src.next() {
                    self.elm = Box::new(stage1::elements(v));
                    if let Some(u) = self.elm.next() {
                        self.win.remove(0);
                        self.win.push(u);
                        return Some(self.win.clone())
                    };
                };
                None
            }
        }
    }
}

impl fmt::Display for Elements {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.win[0])
    }
}
