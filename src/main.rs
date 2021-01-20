#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
mod types;
mod syntax;

#[macro_use]
extern crate regex;
extern crate colored;

use crate::syntax::point;
use crate::syntax::error::*;
use crate::syntax::scan::*;

fn main() {
    // let path = "./etc/main/var/var.fol".to_string();
    let path = "./tst/main".to_string();
    // match source::from_dir("/home/bresilla/data/code/proj/folang/fol/etc") {
    //     Ok(files) => {
    //         for f in files.iter(){
    //             println!("{}", f);    
    //         }
    //     },
    //     Err(e) => println!("{}", e)
    // }

    // let mut srcs = source::Sources::init(path);
    // while let Some(src) = srcs.next() {
    //     println!("{}", src);
    // }
    // let chars = text2::Text::init(path);
    // for c in chars {
    //     println!("{}\t{}", c.1, c.0);
    // }
    // for e in  text2::gen(path) {
    //     println!("{}\t{}", e.1, e.0);
    // }

    // let elem = stage1::Elements::init(path);
    let elem = lexer::Elements::init(path);
    for c in elem {
        println!("{}", c);
    }

    // for e in source::sources("./etc/var/var.fol".to_string()) {
    //     let mut text = text::Text::init(e);
    //     while let Some(win) = text.bump(&mut point::Location::default()) {
    //         println!("{}", win);
    //     }
    // }

    // for e in source::sources("./etc") {
    //     for d in element::elements2(e) {
    //     for l in text::chars(e) {
    //         println!("{}", l);
    //     }
    // }

    // let mut s = vector::Elements::init(path);
    // while !s.list().is_empty() {
    //     println!("{}", s);
    //     s.bump()
    // }


    // let mut e = stream::Stream::init(&path);
    // while let Some(val) = e.bump() {
    //     println!("{}", val)
    // }





}
