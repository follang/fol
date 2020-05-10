#![allow(dead_code)]

use std::fmt;
use crate::parse::lexer;
use crate::parse::node::*;
use crate::scan::token::*;


pub struct ROOT {
    pub el: Vec<NODE>
}

pub fn init() -> ROOT {
    let el: Vec<NODE> = Vec::new();
    ROOT{ el }
}

impl ROOT {
    pub fn parse_tree(&mut self, l: &mut lexer::BAG) {
        while l.not_empty() {
            let node = self.parse_node(l);
            // if let NODE::comp(avec) = &mut self.el { avec.push(node) };
            self.el.push(node)
        }
    }
    pub fn parse_node(&self, l: &mut lexer::BAG) -> NODE {
        let key = l.curr().key();
        println!("{}", l);
        if l.curr().key().is_symbol() {
            let e = NODE::expr(EXPR::ident_);
            l.bump();
            return e
        } else if matches!( key, KEYWORD::void(_) ) {
            let g = NODE::stat(STAT::use_);
            l.bump();
            return g
        } else {
            let g = NODE::expr(EXPR::ident_);
            l.bump();
            return g
        }
    }
}

// macro_rules! matches(
    // ($e:expr, $p:pat) => (
        // match $e {
            // $p => true,
            // _ => false
        // }
    // )
// );



pub fn parse_expr_ident_str(l: &mut lexer::BAG) -> NODE {
    NODE::expr(EXPR::ident_)
}

// pub fn parse_stat_var(l: &mut lexer::BAG) -> NODE {
    // l.bump();
    // // NODE::stat(STAT::var_)
// }
