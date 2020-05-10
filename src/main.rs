#![allow(unused_imports)]
#![allow(unused_variables)]

mod scan;
mod error;
// use crate::scan::token;
// use crate::scan::parts;
use crate::scan::reader;
use crate::scan::scanner;
use crate::scan::stream;
mod parse;
use crate::parse::lexer;
use crate::parse::parser;
use crate::parse::node;

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

    // let mut s = lexer::init("./etc");
    // while s.not_empty() {
        // println!("{}", s);
        // if s.curr().key().is_eol(){
        // }
        // s.bump()
    // }

    let mut s = lexer::init("./etc");
    let mut root = parser::init();
    root.parse_tree(&mut s);
    for e in root.el {
        println!("{}", e);
    }
}
