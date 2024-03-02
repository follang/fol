#[macro_use]
mod types;
mod syntax;
mod helper;

extern crate regex;
extern crate colored;
extern crate dyn_clone;
extern crate terminal_size;

use types::border_up;

use crate::syntax::index;
use crate::syntax::lexer;
use crate::syntax::parse;

fn main() {
    // let input = index::Input::String("let one: int = .5\ntyp 5".to_string(), index::StringType::UserInput);
    let input = index::Input::Path("./test/main/main.fol".to_string(), index::SourceType::File);
    let mut elems = lexer::stage3::Elements::init(&input);
    // for el in elems {
    //      match el {
    //         Ok(o) => { println!("{}", o); },
    //         Err(e) => { println!("{}", e);}
    //     }
    // }
    let parser = parse::Parser::init(&mut elems);
    for e in parser.nodes() { 
        println!("{}\n", e) 
    }

    // for e in parser.errors().iter().enumerate() { 
    //     let bup = border_up("-", " FLAW: #".to_string() + &e.0.to_string() + " ");
    //     println!("{}{}", bup, e.1)
    // }
}

#[test]
fn it_works() {
    assert_eq!("0", "0")
}
