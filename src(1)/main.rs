#![allow(unused_imports)]
#![allow(unused_variables)]

mod error;
mod parsing;
mod scanning;
#[macro_use]
extern crate getset;
extern crate colored;
// use crate::scanning::token;
// use crate::scanning::parts;
use crate::scanning::reader;
use crate::scanning::scanner;
use crate::scanning::stream;
use crate::error::flaw;
use crate::parsing::ast::*;
use crate::parsing::lexer;
use crate::parsing::parser;

fn main() {
    // for mut e in reader::iteratize("./etc") {
    //     for s in scanner::vectorize(&mut e) {
    //         println!("{}", s);
    //         // s.log("--");
    //     }
    // }

    // let mut s = stream::STREAM::init("./etc");
    // while !s.list().is_empty() {
    //     println!("{}", s);
    //     s.bump()
    // }

    // let mut error = flaw::FLAW::init();
    // let mut s = lexer::init("./etc", &mut error);
    // while s.not_empty() {
    //     println!("{}", s);
    //     s.bump()
    // }
        //
    let path = "./etc";
    let mut error = flaw::FLAW::init();
    let mut tokens = lexer::init(path, &mut error);
    let mut forest = parser::new();
    forest.init(&mut tokens, &mut error);
    for tree in forest.trees {
        if let tree_type::stat(stat_type::Ali(_)) = tree.get() {
            // println!("{}\t\t{}", tree.clone().loc(), tree.clone().get());
        }
    }
    error.show();
}
