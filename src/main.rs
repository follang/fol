#![allow(unused_imports)]
#![allow(unused_variables)]

mod syntax;
#[macro_use]
extern crate getset;
extern crate colored;

fn main() {
    // for mut e in reader::iteratize("./etc") {
    //     for s in scanner::vectorize(&mut e) {
    //         println!("{}", s);
    //         // s.log("--");
    //     }
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
