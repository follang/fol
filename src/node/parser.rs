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
        self.parse_stat_var(l, e);
    // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::fun_) ) ||
        // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::fun_) ) ) {
        // self.parse_stat_var(l, e);
    // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::pro_) ) ||
        // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::pro_) ) ) {
        // self.parse_stat_var(l, e);
    // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::log_) ) ||
        // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::log_) ) ) {
        // self.parse_stat_var(l, e);
    // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::typ_) ) ||
        // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::typ_) ) ) {
        // self.parse_stat_var(l, e);
    // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::ali_) ) ||
        // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::ali_) ) ) {
        // self.parse_stat_var(l, e);
    // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::use_) ) ||
        // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::use_) ) ) {
        // self.parse_stat_var(l, e);
    // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::def_) ) ||
        // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::def_) ) ) {
        // self.parse_stat_var(l, e);
    } else {
        // let s = l.expect(KEYWORD::assign(ASSIGN::fun_), e);
        // println!("{}", s);
        if !matches!(l.curr().key(), KEYWORD::assign(ASSIGN::fun_)) { l.report(KEYWORD::assign(ASSIGN::fun_), e); }
        else { l.toend() }
    }
}

pub fn parse_expr_ident_str(&self, l: &mut lexer::BAG, e: &mut err::ERROR) -> tree {
    l.bump();
    tree::new(node::stat(stat::Use), l.curr().loc().clone())
}

pub fn parse_stat_var(&mut self, l: &mut lexer::BAG, e: &mut err::ERROR) ->tree {
    let v = var_stat::init(); let c = l.curr().loc().clone();
    if matches!(l.curr().key(), KEYWORD::symbol(_)) { l.bump() }
    println!("{} \t\t--- {} {}", l.curr().loc(), l.curr().key(), l.curr().con());
    l.toend();
    let n = tree::new(node::stat(stat::Var(v)), c);
    self.el.push(n.clone());
    n
}
}
