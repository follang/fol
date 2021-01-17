#![allow(unused_imports)]
#![allow(unused_variables)]

mod syntax;
#[macro_use]
extern crate regex;
extern crate colored;

use crate::syntax::error::*;
use crate::syntax::scan::*;

fn main() {
    // match source::from_dir("/home/bresilla/data/code/proj/folang/fol/etc") {
    //     Ok(files) => {
    //         for f in files.iter(){
    //             println!("{}", f);    
    //         }
    //     },
    //     Err(e) => println!("{}", e)
    // }


    for e in source::sources("/home/bresilla/data/code/proj/folang/fol/etc/var/var.fol") {
    // for e in source::sources("/home/bresilla/data/code/proj/folang/fol/etc") {
    // for e in source::sources("./etc/var/var.fol") {
    // for e in reader::iteratize("./etc") {
        // println!("{}", e);
        for s in element::elements(&e) {
            println!("{}", s);
            // s.log("--");
        }
    }


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
}
