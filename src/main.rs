// #![allow(unused_imports)]
#![allow(unused_variables)]

mod scan;
// use crate::scan::token;
// use crate::scan::parts;
// use crate::scan::reader;
// use crate::scan::scanner;
// use crate::scan::stream;
mod parse;
use crate::parse::lexer;

fn main() {
    // let red = reader::READER::init("./dirk.mod");
    // for e in red.iter(){
        // // println!("-----------------------------------------------------------------");
        // println!("{}/{}", e.name(), e.file());
        // for d in e.data().chars(){
            // print!("{}", d);
            // // print!("|{}", (d as u8))
        // }
    // }

    // println!("-----------------------------------------------------------------");
    // for e in lexer::tokenize("./dirk.mod") {
    // println!("{}", e)
    // }

    // for mut e in reader::iteratize("./etc") {
        // println!("{}/{}", e.name(), e.file());
        // for p in lexer::iteratize(&mut e) {
            // println!("{}", p)
        // }
        // for p in scanner::vectorize(&mut e).iter() {
            // println!("{}", p)
        // }
    // }

    // let a = [1, 2, 3, 4, 5];
    // let mut b = a.iter().peekable();
    // println!("{}", b.peek().nth(0).unwrap());
    // println!("{}", b.nth(0).unwrap());
    // println!("{}", b.nth(0).unwrap());
    // for p in lexer::vectorize("./etc") {
        // println!("{}", p)
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
