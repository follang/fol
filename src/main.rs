#[macro_use]
mod types;
mod syntax;
mod helper;

extern crate regex;
extern crate colored;
extern crate dyn_clone;
extern crate terminal_size;

use crate::syntax::index::Input;
use crate::syntax::lexer;
use crate::syntax::parse;

fn main() {
    let mut elems = lexer::Elements::init(&Input::Path("./test/main/main.fol".to_string(), true));
    // while let Some(c) = elems.bump() {
    //     match c {
    //         Ok(e) => { println!("{}", e); },
    //         Err(e) => { println!("\n{}", e); }
    //     }
    // }
    let _parser = parse::Parser::init(&mut elems);
    // for e in Source::init(&path, false).iter() {
    //     let mut elems = lexer::Elements::init(&Input::SourceAlt(e.clone());
    //     let parser = parse::Parser::init(&mut elems);
    // }
}

#[test]
fn it_works() {
    assert_eq!("0", "0")
}
