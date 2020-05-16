#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_macros)]

use std::fmt;
use crate::node::lexer;
use crate::node::ast::*;
use crate::scan::token::*;
use crate::scan::locate;
use crate::error::err;


pub struct forest {
    pub el: Vec<tree>
}

pub fn new() -> forest {
    let el = Vec::new();
    // let loc =  locate::LOCATION::def();
    forest{ el }
}

macro_rules! expect(($e:expr, $p:pat) => (
    match $e {
        $p => { true },
        _ => { false }
    }
));

impl forest {
    pub fn init(&mut self, l: &mut lexer::BAG, e: &mut err::ERROR) {
        while l.not_empty() {
            self.parse_node(l, e);
        }
    }
    pub fn parse_node(&mut self, l: &mut lexer::BAG, e: &mut err::ERROR) {
        // println!("{}", l);
        if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::var_) ) ||
            ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::var_) ) ) {
            self.parse_stat_var(l);
        } else if matches!( l.curr().key(), KEYWORD::void(_) ) {
            self.parse_expr_ident_str(l);
        } else {
            let s: String = String::from("expected {") +
                &KEYWORD::assign(ASSIGN::var_).to_string() +
                "} got {" + &l.curr().key().to_string() + "}";

            e.report(err::TYPE::parser, &s, l.curr().loc().clone());
            l.toend();
        }
    }

    pub fn parse_expr_ident_str(&self, l: &mut lexer::BAG) -> tree {
        l.bump();
        tree::new(node::stat(stat::Use), l.curr().loc().clone())
    }

    pub fn parse_stat_var(&mut self, l: &mut lexer::BAG) ->tree {
        let v = var_stat::init(); let c = l.curr().loc().clone();
        if expect!(l.curr().key(), KEYWORD::symbol(_)) { l.bump() }
        println!("{} \t\t--- {} {}", l.curr().loc(), l.curr().key(), l.curr().con());
        l.toend();
        let n = tree::new(node::stat(stat::Var(v)), c);
        self.el.push(n.clone());
        n
    }
}
