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


    // for e in source::sources("/home/bresilla/data/code/proj/folang/fol/etc/var/var.fol") {
    // for e in source::sources("/home/bresilla/data/code/proj/folang/fol/etc") {
    // for e in source::sources("./etc/var/var.fol") {
    //     for d in element::elements2(e) {
    //         println!("{}", d);
    //     }
    for e in source::sources("./etc") {
        for l in text::chars(e) {
            println!("{}", l);
        }
        // for s in element::elements(&e) {
        //     println!("{}", s);
        //     // s.log("--");
        // }
    }


    // let mut s = vector::Elements::init("./etc/var");
    // while !s.list().is_empty() {
    //     println!("{}", s);
    //     s.bump()
    // }


    // let mut e = stream::Elements::init("./etc");
    // loop {
    //     match e.bump() {
    //         Some(val) => {
    //             // println!("{}", val.len());
    //             println!("{}", val[0]);
    //         }
    //         None => {
    //             // println!("{}", e);
    //             break;
    //         }
    //     }
    // }

    // let mut error = flaw::FLAW::init();
    // let mut s = lexer::init("./etc", &mut error);
    // while s.not_empty() {
    //     println!("{}", s);
    //     s.bump()
    // }
}
