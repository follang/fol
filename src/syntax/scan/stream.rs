#![allow(dead_code)]

use std::fmt;
use std::iter;
use colored::Colorize;
use crate::syntax::point;
use crate::syntax::token;
use crate::syntax::scan::source;
use crate::syntax::scan::element;

use crate::syntax::scan::element::Element;
use crate::syntax::error::*;

// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Elements {
    src: Box<dyn Iterator<Item = source::Source>>,
    elm: Box<dyn Iterator<Item = Element>>,
    tri: (Element, Element, Element),
}

impl Elements {
    pub fn elements(self) -> Box<dyn Iterator<Item = Element>> {
        self.elm
    }
    pub fn sources(self) -> Box<dyn Iterator<Item = source::Source>> {
        self.src
    }
}

impl Elements {
    pub fn init(path: &'static str) -> Self {
        let mut src = Box::new(source::sources(&path));
        let elm = Box::new(element::elements2(src.next().unwrap()));
        Elements {
            src, elm, tri: (
                Element::default(), 
                Element::default(), 
                Element::default()
            )
        }
    }

    pub fn bump(&mut self) -> Opt<(Element, Element, Element)> {
        println!("{}", self.tri.1);
        match self.elm.next() {
            Some(v) => {
                self.tri = (self.tri.1.to_owned(), self.tri.2.to_owned(), v);
                Some(self.tri.clone())
            },
            None => {
                if let Some(v) = self.src.next() {
                    self.elm = Box::new(element::elements2(v));
                    if let Some(u) = self.elm.next() {
                        self.tri = (self.tri.1.to_owned(), self.tri.2.to_owned(), u);
                        return Some(self.tri.clone())
                    };
                };
                None
            }
        }
    }
}

impl fmt::Display for Elements {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tri.1)
    }
}
