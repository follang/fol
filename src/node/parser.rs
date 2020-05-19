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
            l.toend(e)
        }
    }
}

// VAR STATEMENT
impl forest {
    pub fn parse_stat_var(&mut self, l: &mut lexer::BAG, e: &mut err::FLAW, t: &var_stat, jump: bool) {
        let c = l.curr().loc().clone();
        let mut options: Vec<assign_opts> = Vec::new();
        let mut v = t.clone();

        if !jump {
            // option symbol
            if matches!(l.curr().key(), KEYWORD::option(_)) {
                self.help_assign_options(&mut options, l, e);
            }

            // assign var
            l.bump(); l.eat_space(e, false);

            // option elements
            if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
                self.help_assign_options(&mut options, l, e);
            }
        }
        v.set_options(options);

        if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
            l.bump(); l.eat_space(e, false);
            while matches!(l.curr().key(), KEYWORD::ident) {
                self.parse_stat_var(l, e, &v, true);
                l.bump()
            }
            if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::roundC_)) { l.bump(); } else { l.expect_report(KEYWORD::symbol(SYMBOL::roundC_), e) }
            l.eat_terminal(e, true);
            println!("{:>2} {} \t\t {}", l.curr().loc().row(), l.curr().key(), l.next().key());
            return
        }



        if matches!(l.curr().key(), KEYWORD::ident) {
            v.set_ident(l.curr().con().clone());
            l.bump();
        } else {
            let s = String::from("expected { ") + &KEYWORD::ident.to_string() + " }, got { " + &l.curr().key().to_string() + " }";
            l.report(s, e);
        }


        if l.curr().key().is_terminal(){
            self.trees.push(tree::new(root::stat(stat::Var(v)), c));
            l.eat_terminal(e, true);
            return;
        }

        l.toend(e);
        self.trees.push(tree::new(root::stat(stat::Var(v)), c));
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
        l.bump(); l.eat_space(e, true);
    }
}
