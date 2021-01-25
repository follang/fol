#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
mod types;
mod syntax;

#[macro_use]
extern crate regex;
extern crate colored;

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
        // vec.push(c);
        match c {
            Ok(o) => {
                // println!("{}", o);
                el.push(o);
            },
            Err(e) => {
                // println!("{}", e);
                er.push(e);
            }
        }
    }
    // let newvec = er.clone();
    for c in el {
        println!("{}", c);
    }

}
