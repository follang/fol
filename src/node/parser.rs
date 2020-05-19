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
    pub trees: Vec<tree>
}

pub fn new() -> forest {
    let trees = Vec::new();
    // let loc =  locate::LOCATION::def();
    forest{ trees }
}

impl forest {
    pub fn init(&mut self, l: &mut lexer::BAG, e: &mut err::FLAW) {
        while l.not_empty() {
            self.parse_stat(l, e);
        }
    }
    pub fn parse_stat(&mut self, l: &mut lexer::BAG, e: &mut err::FLAW) {
    // println!("{}", l);
        if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::var_) ) ||
            ( matches!( l.curr().key(), KEYWORD::option(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::var_) ) ) {
            self.parse_stat_var(l, e, &var_stat::init(), false);
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
            if !matches!(l.curr().key(), KEYWORD::literal(_)) {
                let s = String::from("expected { ") + &KEYWORD::literal(LITERAL::ANY).to_string() +
                    " }, got { " + &l.curr().key().to_string() + " }";
                // l.report(s, e);
                l.toend();
                return
            }
            l.toend()
        }
    }
}

// VAR STATEMENT
impl forest {
    pub fn parse_stat_var(&mut self, l: &mut lexer::BAG, e: &mut err::FLAW, t: &var_stat, jump: bool) -> tree {
        let c = l.curr().loc().clone();
        let mut options: Vec<assign_opts> = Vec::new();

        if !jump {
            // option symbol
            if matches!(l.curr().key(), KEYWORD::option(_)) {
                self.help_assign_options(&mut options, l, e);
            }

            // assign var
            l.bump_n_eat(e);


            // option elements
            if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
                self.help_assign_options(&mut options, l, e);
            }
        }

        println!("{:>2} {} \t\t {}", l.curr().loc().row(), l.curr().key(), l.next().key());



        let mut v = var_stat::init();
        v.set_options(options);

        l.toend();
        let n = tree::new(root::stat(stat::Var(v)), c);
        self.trees.push(n.clone());
        n
    }

    pub fn help_assign_options(&mut self, v: &mut Vec<assign_opts>, l: &mut lexer::BAG, e: &mut err::FLAW) {
        if matches!(l.curr().key(), KEYWORD::option(_)) {
            let el = match l.curr().key() {
                KEYWORD::option(OPTION::mut_) => { assign_opts::Mut }
                KEYWORD::option(OPTION::sta_) => { assign_opts::Sta }
                KEYWORD::option(OPTION::exp_) => { assign_opts::Exp }
                KEYWORD::option(OPTION::hid_) => { assign_opts::Hid }
                KEYWORD::option(OPTION::hep_) => { assign_opts::Hep }
                _ => { assign_opts::Nor }
            };
            v.push(el);
            l.bump();
            return
        }
        let deep = l.curr().loc().deep() -1;
        l.bump();
        loop {
            //TODO: finish options
            if l.match_bracket(KEYWORD::symbol(SYMBOL::curlyC_), deep) { break }
            l.bump();
        }
        l.bump_n_eat(e);
    }
}
