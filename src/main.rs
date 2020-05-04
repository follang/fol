#![allow(unused_imports)]
#![allow(unused_variables)]

mod scan;
// use crate::scan::token;
// use crate::scan::parts;
use crate::scan::reader;
use crate::scan::scanner;
use crate::scan::stream;
mod parse;
use crate::parse::lexer;

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

    let mut s = lexer::LEXEME::init("./etc");
    while !s.list().is_empty() {
        println!("{}", s);
        s.bump()
    }


}
