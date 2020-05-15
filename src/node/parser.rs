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

pub fn init() -> forest {
    let el = Vec::new();
    // let loc =  locate::LOCATION::def();
    forest{ el }
}

impl forest {
    pub fn init(&mut self, l: &mut lexer::BAG) {
        while l.not_empty() {
            let node = self.parse_node(l);
            // if let NODE::comp(avec) = &mut self.el { avec.push(node) };
            self.el.push(node)
        }
    }
    pub fn parse_node(&self, l: &mut lexer::BAG) -> tree {
        let n: tree;
        // println!("{}", l);
        if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::var_) ) {
            n = self.parse_stat_var(l);
        } else if matches!( l.curr().key(), KEYWORD::void(_) ) {
            n = self.parse_expr_ident_str(l);
        } else {
            n = (node::expr(expr::Ident), l.curr().loc().clone());
            l.bump();
        }
        // println!("{}", n.1);
        return n;
    }

    pub fn parse_expr_ident_str(&self, l: &mut lexer::BAG) -> tree {
        l.bump();
        (node::stat(stat::Use), l.curr().loc().clone())
    }

    pub fn parse_stat_var(&self, l: &mut lexer::BAG) -> tree {
        let n: tree;
        println!("{} \t\t--- {}", l.curr().loc(), l.curr().key());
        let v = var_stat::init();
        l.bump();
        n = (node::stat(stat::Var(v)), l.curr().loc().clone());
        println!("{} \t\t--- {}", l.curr().loc(), l.curr().key());
        n
    }
}
