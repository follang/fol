#![allow(unused_imports)]
#![allow(unused_variables)]

mod syntax;
#[macro_use]
extern crate regex;
extern crate colored;

use crate::syntax::point;
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
    //     while let Some(win) = text.bump2(&mut point::Location::default()) {
    //         println!("{}", win);
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


    let mut e = stream::Elements::init("./etc");
    while let Some(val) = e.bump() {
        println!("{}", val)
    }
}
