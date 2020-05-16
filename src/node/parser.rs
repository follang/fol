#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::fmt;
use crate::node::lexer;
use crate::node::ast::*;
use crate::scan::token::*;
use crate::scan::locate;


pub struct forest {
    pub el: Vec<tree>
}

pub fn new() -> forest {
    let el = Vec::new();
    // let loc =  locate::LOCATION::def();
    forest{ el }
}

impl forest {
    pub fn init(&mut self, l: &mut lexer::BAG) {
        while l.not_empty() {
            self.parse_node(l);
            // if let NODE::comp(avec) = &mut self.el { avec.push(node) };
        }
    }
    pub fn parse_node(&mut self, l: &mut lexer::BAG) {
        // println!("{}", l);
        if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::var_) ) ||
            ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::var_) ) ) {
            self.parse_stat_var(l);
        } else if matches!( l.curr().key(), KEYWORD::void(_) ) {
            self.parse_expr_ident_str(l);
        } else {
            l.bump();
        }
    }

    pub fn parse_expr_ident_str(&self, l: &mut lexer::BAG) -> tree {
        l.bump();
        (node::stat(stat::Use), l.curr().loc().clone())
    }

    pub fn parse_stat_var(&mut self, l: &mut lexer::BAG) {
        let n: tree;
        let v = var_stat::init();
        let c = l.curr().loc().clone();
        println!("{} \t\t--- {} {}", l.curr().loc(), l.curr().key(), l.curr().con());
        l.times(5);
        n = (node::stat(stat::Var(v)), c);
        self.el.push(n);
    }
}
