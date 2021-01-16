#![allow(unused_imports)]
#![allow(unused_variables)]

mod syntax;
#[macro_use]
extern crate regex;
extern crate colored;

use crate::syntax::scan::*;

fn main() {
    // for f in reader::file_list("./etc").iter(){
    //     println!("{}", f);    
    // }
    match reader::from_dir("./etp") {
        Ok(files) => {
            for f in files.iter(){
                println!("{}", f);    
            }
        },
        Err(e) => println!("{}", e)
    }
    // if let Ok(files) = reader::from_dir("./etc"){
    //     for f in files.iter(){
    //         println!("{}", f);    
    //     }
    // }
    // println!("{}", reader::READER::file("./etc/var/var.fol"));
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
