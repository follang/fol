#[macro_use]
mod types;
mod syntax;
mod helper;

extern crate regex;
extern crate colored;
extern crate dyn_clone;
extern crate terminal_size;

use types::border_up;

use crate::syntax::index::Input;
use crate::syntax::lexer;
use crate::syntax::parse;

fn main() {
    let mut elems = lexer::stage3::Elements::init(&Input::Path("./test/main/main.fol".to_string(), true));
    let parser = parse::Parser::init(&mut elems);
    for e in parser.nodes() { println!("{}", e) }
    for e in parser.errors().iter().enumerate() { 
        let bup = border_up("-", " FLAW: #".to_string() + &e.0.to_string() + " ");
        println!("{}{}", bup, e.1)
    }
}

#[test]
fn it_works() {
    assert_eq!("0", "0")
}
