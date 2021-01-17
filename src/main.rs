#![allow(unused_imports)]
#![allow(unused_variables)]

mod syntax;
#[macro_use]
extern crate regex;
extern crate colored;

use crate::syntax::error::*;
use crate::syntax::scan::*;

fn main() {
    match reader::from_dir("./etc") {
        Ok(files) => {
            for f in files.iter(){
                println!("{}", f);    
            }
        },
        Err(e) => println!("{}", e)
    }
    // match reader::from_dir("/home/bresilla/data/code/proj/folang/fol/etc") {
    //     Ok(files) => {
    //         for f in files.iter(){
    //             println!("{}", f);    
    //         }
    //     },
    //     Err(e) => println!("{}", e)
    // }
    // println!("{}", reader::READER::file("/home/bresilla/data/code/proj/folang/fol/etc/var/var.fol").unwrap());
    // println!("----");
    // for en in reader::READER::init("/home/bresilla/data/code/proj/folang/fol/etp").unwrap().iter() {
    // for en in reader::READER::from_dir("./etc").unwrap().iter() {
        // println!("{}", en);
    // }
    // println!("{}", reader::READER::from_dir("./etc/var/var.fol"));
    // for mut e in reader::iteratize("./etc") {
        // println!("{}", e);
        // for s in scanner::vectorize(&mut e) {
        //     println!("{}", s);
        //     // s.log("--");
        // }
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
}
