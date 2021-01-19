#![allow(dead_code)]

use std::fmt;
use crate::syntax::point;
use crate::syntax::scan::source;
use crate::syntax::scan::text;

use crate::syntax::token::KEYWORD::*;
use crate::syntax::token::*;

// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
// pub struct Element {
//     key: KEYWORD,
//     loc: point::Location,
//     con: String,
// }



// impl std::default::Default for Element {
//     fn default() -> Self {
//         Self {
//             key: KEYWORD::illegal,
//             loc: point::Location::default(),
//             con: String::new(),
//         }
//     }
// }

// impl Element {
//     pub fn empty(key: KEYWORD, loc: point::Location, con: String) -> Self {
//         Element { key, loc, con }
//     }
//     pub fn key(&self) -> &KEYWORD {
//         &self.key
//     }
//     pub fn loc(&self) -> &point::Location {
//         &self.loc
//     }
//     pub fn con(&self) -> &String {
//         &self.con
//     }
//     pub fn set_key(&mut self, k: KEYWORD) {
//         self.key = k;
//     }
// }

// /// Creates a iterator that produces tokens from the input string.
// pub fn elements2(src: source::Source) -> impl Iterator<Item = Element> {
//     let mut loc = point::Location::init((src.path(true), src.path(false)), &src.module());
//     let mut code = text::Text::init(src);
//     std::iter::from_fn(move || {
//         if let Some(v) = code.bump(&mut loc) {
//             return Some(Element::default());
//         }
//         None
//     })
// }
