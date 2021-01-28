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
use crate::syntax::lexer::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::parse::*;

fn main() {
    let path = "./test/main/var".to_string();

    // let elems = source::Sources::init(path);
    // let elems = text::Text::init(path);
    // let elems = stage1::Elements::init(path);
    // let elems = stage2::Elements::init(path);
    // for e in elems {
        // println!("{}", e);
    // }
    // let mut el = Vec::new();
    // let mut er = Vec::new();
    let mut elems = Elements::init(path);
    let parser = Parser::default().parse(&mut elems);
    // for o in elems.filter(|x| x.key() == KEYWORD::comment) {
    //     println!("{}", o);
    // }
    // while let Some(c) = elems.bump() {
    //     match c {
    //         Ok(o) => { 
    //             el.push(o); 
    //         },
    //         Err(e) => { 
    //             er.push(e); 
    //         }
    //     }
    // }
    // if er.len() == 0 {
    //     for c in el {
    //         println!("{}", c);
    //     }
    // } else {
    //     for c in er {
    //         println!("{}", c);
    //     }
    // }
}
