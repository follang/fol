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


    // for e in source::sources("./etc/var/var.fol") {
    //     let mut text = text::Text::init(e);
    //     while let Some(win) = text.bump3() {
    //         println!("{}", win[0]);
    //     }
    // }

    // for e in source::sources("./etc") {
    //     for d in element::elements2(e) {
    //     for l in text::chars(e) {
    //         println!("{}", l);
    //     }
    // }

    // let mut s = vector::Elements::init("./etc/var");
    // while !s.list().is_empty() {
    //     println!("{}", s);
    //     s.bump()
    // }


    let mut e = stream::Elements::init("./etc/var");
    loop {
        match e.bump() {
            Some(val) => {
                // println!("{}", val.len());
                println!("{}", val[0]);
            }
            None => {
                // println!("{}", e);
                break;
            }
        }
    }
}
