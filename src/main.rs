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
    let mut vec = Vec::new();
    let elem = Elements::init(path);
    for c in elem {
        vec.push(c);
    }
    let newvec = vec.clone();
    for c in newvec {
        println!("{}", c);
    }

}
