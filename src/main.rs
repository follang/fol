#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
mod types;
mod syntax;

#[macro_use]
extern crate regex;
extern crate colored;
extern crate dyn_clone;

use crate::syntax::point;
use crate::syntax::lexer::{Element, Elements};

fn main() {
    let path = "./test/main/var/var.fol".to_string();
    // let path = "./test/main".to_string();

    // let elem = source::Sources::init(path);
    // let elem = text2::Text::init(path);
    // let elem = stage1::Elements::init(path);
    let mut el = Vec::new();
    let mut er = Vec::new();
    let mut elem = Elements::init(path);
    while let Some(c) = elem.bump() {
        match c {
            Ok(o) => { el.push(o); },
            Err(e) => { er.push(e); }
        }
    }
    if er.len() == 0 {
        for c in el {
            println!("{}", c);
        }
    } else {
        for c in er {
            println!("{}", c);
        }
    }

}
