#![allow(dead_code)]

use std::fmt;
use crate::parse::lexer;
use crate::parse::node::*;


pub struct ROOT {
    el: NODE
}

pub fn init() -> ROOT {
    let vec: Vec<NODE> = Vec::new();
    ROOT{ el: NODE::comp(vec) }
}

impl ROOT {
    pub fn parse_tree(&self, l: &mut lexer::BAG) {
        while l.not_empty() {
            println!("{}", l);
            l.bump()
        }
    }
}

pub fn parse_node(r: &mut ROOT, l: &mut lexer::BAG) {

}

pub fn parse_expr_ident_string(l: &mut lexer::BAG) -> NODE {
    NODE::expr(EXPR::ident_)
}
