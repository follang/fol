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
extern crate terminal_size;

use std::fs::File;
use crate::types::*;
use crate::syntax::point;
use crate::syntax::index;
use crate::syntax::lexer::stage1;
use crate::syntax::lexer::text;
use crate::syntax::lexer::*;
use crate::syntax::token::*;
use crate::syntax::nodes::*;
use crate::syntax::parse::*;

fn main() -> Vod {
    let path = "./test/main/var2".to_string();
    let sources = index::sources::Sources::init(path);
    for e in sources {
        // let mut el = Vec::new();
        // let mut er = Vec::new();
        let mut elems = stage2::Elements::init(&e);
        // while let Some(c) = elems.bump() {
        //     match c {
        //         Ok(e) => { println!("{}", e); },
        //         Err(e) => { println!("{}", e); }
        //     }
        // }
        let mut parser = Parser::default();
        parser.init(&mut elems, &e);
    }
    Ok(())
}
