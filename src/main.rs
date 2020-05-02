#![allow(unused_imports)]
#![allow(unused_variables)]

mod lex;
use std::str;
use crate::lex::token;
use crate::lex::parts;
use crate::lex::reader;
use crate::lex::lexer;

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

    for mut e in reader::readerize("./dirk.mod") {
        // println!("{}/{}", e.name(), e.file());
        // for p in lexer::iteratize(&mut e) {
            // println!("{}", p)
        // }
        for p in lexer::vectorize(&mut e).iter() {
            println!("{}", p)
        }
    }

    // let a = [1, 2, 3, 4, 5];
    // let mut b = a.iter().peekable();
    // println!("{}", b.peek().nth(0).unwrap());
    // println!("{}", b.nth(0).unwrap());
    // println!("{}", b.nth(0).unwrap());


}
