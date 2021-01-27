#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
mod types;
mod syntax;

#[macro_use]
extern crate regex;
extern crate colored;
extern crate dyn_clone;

use std::fs::File;
use crate::syntax::point;
use crate::syntax::lexer::{Element, Elements};
use crate::syntax::nodes::*;
use crate::syntax::token::*;

fn main() {
    let mut vec: Vec<Node> = Vec::new();
    let node = Expr::new(None, Box::new(NumberExpr::int(5)));
    let node2 = Expr::new(None, Box::new(NumberExpr::int(6)));
    vec.push(node.into());
    vec.push(node2.into());
    for e in vec {
        println!("{}", e);
    }
    let path = "./test/main/var".to_string();
    // let path = "./test/main".to_string();

    // let elem = source::Sources::init(path);
    // let elem = text2::Text::init(path);
    // let elem = stage1::Elements::init(path);
    // let mut el = Vec::new();
    // let mut er = Vec::new();
    let mut elem = Elements::init(path);
    // for o in elem.filter(|x| x.key() == KEYWORD::comment) {
    //     println!("{}", o);
    // }
    while let Some(c) = elem.bump() {
        match c {
            Ok(o) => { 
                // el.push(o); 
                // if o.key() == KEYWORD::literal(LITERAL::char_) {
                    println!("{}", o);
                // }
            },
            Err(e) => { 
                // er.push(e); 
                println!("{}", e);
            }
        }
    }
    // if er.len() == 0 {
    //     for c in el {
    //         println!("{}", c);
    //     }
    // } else {
    //     for c in er {
    //         println!("{}", c);
    //     }
    // }
}
