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
    // let reader = reader::READER::init("/home/bresilla/data/code/PROJECTS/folang/fol_in_rust/dirk.mod");
    // for e in red.iter(){
        // // println!("-----------------------------------------------------------------");
        // println!("{}/{}", e.name(), e.file());
        // // for d in e.data().chars(){
            // // print!("{}", d)
        // // }
        // // let parter = parts::Part::new(str::from_utf8(&e.data()).unwrap());


    // }
    // println!("-----------------------------------------------------------------");

    // for e in reader::readerize("./dirk.mod") {
        // println!("{}/{}", e.name(), e.file());
    // }
    // println!("-----------------------------------------------------------------");
    for e in lexer::tokenize("./dirk.mod") {
        println!("{}", e)
    }
    println!("-----------------------------------------------------------------");
    // for e in lexer::read_dir("./dirk.mod") {
        // println!("{}", e)
    // }
}
