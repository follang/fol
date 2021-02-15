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

use crate::syntax::index::{ Input, Source };
use crate::syntax::lexer;
use crate::syntax::parse;

fn main() {
    let path = "./test/main".to_string();
    let mut elems = lexer::Elements::init(&Input::Source(path, false));
    let parser = parse::Parser::init(&mut elems);
    
    // for e in index::Source::init(&path, false).iter() {
    //     let input = index::Input::Source(e.clone());
    //     let mut elems = lexer::Elements::init(&input);
    //     // while let Some(c) = elems.bump() {
    //     //     match c {
    //     //         Ok(e) => { println!("{}", e); },
    //     //         Err(e) => { println!("\n{}", e); }
    //     //     }
    //     // }
    //     let parser = parse::Parser::init(&mut elems);
    // }
}
