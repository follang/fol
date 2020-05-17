#![allow(unused_imports)]
#![allow(unused_variables)]

mod scan;
// use crate::scan::token;
// use crate::scan::parts;
use crate::scan::reader;
use crate::scan::scanner;
use crate::scan::stream;
mod node;
use crate::node::lexer;
use crate::node::parser;
use crate::node::ast;
mod error;
use crate::error::err;

fn main() {
    // for mut e in reader::iteratize("./etc") {
        // for s in scanner::vectorize(&mut e) {
            // println!("{}", s);
        // }
    // }

    // let mut s = stream::STREAM::init("./etc");
    // while !s.list().is_empty() {
        // println!("{}", s);
        // s.bump()
    // }

    // let mut error = err::ERROR::init();
    // let path = "./etc";
    // let mut s = lexer::init(path, &mut error);
    // while s.not_empty() {
        // println!("{}", s);
        // if s.curr().key().is_eol(){
        // }
        // s.bump()
    // }

    let path = "./etc";
    let mut error = err::ERROR::init();
    let mut tokens = lexer::init(path, &mut error);
    let mut forest = parser::new();
    forest.init(&mut tokens, &mut error);
    for tree in forest.trees {
        println!("{}\t{}", tree.loc(), tree.node());
    }
    error.show();
}
