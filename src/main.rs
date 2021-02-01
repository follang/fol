#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
mod types;
mod syntax;
mod helper;

#[macro_use]
extern crate regex;
extern crate colored;
extern crate dyn_clone;

use std::fs::File;
use crate::types::*;
use crate::syntax::point;
use crate::syntax::index;
use crate::syntax::lexer::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::parse::*;

fn main() {
    let path = "./test/main/var".to_string();
    let sources = index::sources::Sources::init(path);
    for e in sources {
        let mut elems = Elements::init(&e);
        let parser = Parser::default().parse(&mut elems);
    }
}
