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
            self.parse_stat_var(l, e, &mut var_stat::init(), false);
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
            l.to_end(e)
        }
    }
}

// VAR STATEMENT
impl forest {
    pub fn parse_stat_var(&mut self, l: &mut lexer::BAG, e: &mut err::FLAW, mut t: &mut var_stat, group: bool) {
        let c = l.curr().loc().clone();
        let mut options: Vec<assign_opts> = Vec::new();
        let mut list: Vec<String> = Vec::new();

        if !group {
            // option symbol
            if matches!(l.curr().key(), KEYWORD::option(_)) {
                self.help_assign_options(&mut options, l, e);
            }

            // assign var
            l.bump(); l.eat_space(e);

            // option elements
            if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
                self.help_assign_options(&mut options, l, e);
                if l.curr().key().is_space() {
                    l.eat_space(e);
                } else {
                    l.expect_report(KEYWORD::void(VOID::space_).to_string(), e);
                    return
                }
                l.eat_space(e);
            }
            t.set_options(options);

            // group variables
            if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
                l.bump(); l.eat_space(e);
                while matches!(l.curr().key(), KEYWORD::ident) {
                    self.parse_stat_var(l, e, &mut t, true);
                    l.bump()
                }
                if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::roundC_)) {
                    l.bump();
                } else {
                    l.expect_report(KEYWORD::symbol(SYMBOL::roundC_).to_string(), e);
                    return
                }
                if l.curr().key().is_terminal() {
                    l.eat_termin(e);
                } else {
                    l.expect_report(KEYWORD::void(VOID::endline_).to_string(), e);
                    return
                }
                return
            }
        }

        //identifier
        l.eat_space(e);
        if matches!(l.curr().key(), KEYWORD::ident) {
            t.set_ident(l.curr().con().clone());
            l.bump();
        } else {
            l.expect_report(KEYWORD::ident.to_string(), e);
            return
        }

        // list variables
        if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::comma_)) {
            l.bump(); l.eat_space(e);
            loop {
                if matches!(l.curr().key(), KEYWORD::ident) {
                    list.push(l.curr().con().clone());
                    l.bump(); l.eat_space(e);
                }
                if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::comma_)) {
                    l.bump();
                    l.eat_space(e);
                }
                if !matches!(l.curr().key(), KEYWORD::ident) { break }
            }
        }


        // short version (no type)
        if l.curr().key().is_terminal(){
            self.trees.push(tree::new(root::stat(stat::Var(t.clone())), c));
            l.eat_termin(e);
            return;
        }

        if !(matches!(l.look().key(), KEYWORD::operator(OPERATOR::assign_))
            || matches!(l.look().key(), KEYWORD::operator(OPERATOR::assign2_))
            || matches!(l.look().key(), KEYWORD::symbol(SYMBOL::colon_))
            ) {
            l.eat_space(e);
            l.expect_report(KEYWORD::operator(OPERATOR::assign_).to_string() + " } or { " +
                &KEYWORD::operator(OPERATOR::assign2_).to_string() + " } or { " +
                &KEYWORD::symbol(SYMBOL::colon_).to_string(), e);
        }
        l.eat_space(e);


        if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::colon_)) {}

        println!("{:>2}  {:>2}  {:>2}\t\t{} \t {}", "", l.curr().loc().col(), l.curr().loc().row(), l.curr().key(), l.next().key());

        // short assign ':='
        if matches!(l.curr().key(), KEYWORD::operator(OPERATOR::assign2_)) || matches!(l.curr().key(), KEYWORD::operator(OPERATOR::assign2_)) {
        }

        l.to_end(e);
        self.trees.push(tree::new(root::stat(stat::Var(t.clone())), c.clone()));

        for e in list {
            let mut clo = t.clone();
            clo.set_ident(e);
            self.trees.push(tree::new(root::stat(stat::Var(clo)), c.clone()));
        }

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
        l.bump();
    }
}
