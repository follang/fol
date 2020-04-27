#![allow(unused_imports)]
#![allow(unused_variables)]

mod lex;
use std::str;
use crate::lex::token;
use crate::lex::parts;
use crate::lex::reader;
use crate::lex::lexer::*;

fn main() {
    let reader = reader::READER::init("./dirk.mod");
    for e in reader.iter(){
        println!("-----------------------------------------------------------------");
        println!("{}", e.path());
        let parter = parts::Part::new(str::from_utf8(&e.data()).unwrap());


    }
}
