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
use colored::Colorize;


pub struct forest {
    pub trees: Vec<tree>
}

pub fn new() -> forest {
    forest{ trees: Vec::new() }
}

impl forest {
    pub fn init(&mut self, l: &mut lexer::BAG, e: &mut err::FLAW) {
        while l.not_empty() {
            self.parse_stat(l, e);
        }
    }
    pub fn parse_stat(&mut self, l: &mut lexer::BAG, e: &mut err::FLAW) {
    // println!("{}", l);
        if l.prev().key().is_terminal() || l.prev().key().is_eof() {
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
}


//------------------------------------------------------------------------------------------------------//
//                                             VAR STATEMENT                                            //
//------------------------------------------------------------------------------------------------------//
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

            // eat assign var
            l.bump();

            // option elements
            if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
                self.help_assign_options(&mut options, l, e);
            }
            t.set_options(options);

            // ERROR if not 'space'
            if !(matches!(l.curr().key(), KEYWORD::void(VOID::space_))) {
                l.space_add_report(l.prev().key().to_string(), e); return;
            } else { l.eat_space(e); }

            // ERROR if '[' because is a little late for that
            if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
                l.space_rem_report(l.past().key().to_string(), e); return;
            }

            // group variables matching "("
            if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
                l.bump(); l.eat_space(e);
                while matches!(l.curr().key(), KEYWORD::ident(_)) {
                    self.parse_stat_var(l, e, &mut t, true);
                    l.bump()
                }
                if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::roundC_)) {
                    l.bump();
                } else {
                    l.unexpect_report(KEYWORD::symbol(SYMBOL::roundC_).to_string(), e);
                    return
                }
                if l.curr().key().is_terminal() {
                    l.eat_termin(e);
                } else {
                    l.unexpect_report(KEYWORD::void(VOID::endline_).to_string(), e);
                    return
                }
                return
            }
        }

        //identifier
        if matches!(l.curr().key(), KEYWORD::ident(_)) {
            t.set_ident(l.curr().con().clone());
            l.bump();
        } else {
            l.unexpect_report(KEYWORD::ident(String::new()).to_string(), e);
            return
        }

        // list variables
        if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::comma_)) {
            l.bump(); l.eat_space(e);
            loop {
                if matches!(l.curr().key(), KEYWORD::ident(_)) {
                    list.push(l.curr().con().clone());
                    l.bump(); l.eat_space(e);
                }
                if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::comma_)) {
                    l.bump();
                    l.eat_space(e);
                }
                if !matches!(l.curr().key(), KEYWORD::ident(_)) { break }
            }
        }

        // short version (no type)
        if l.look().key().is_terminal(){
            self.trees.push(tree::new(root::stat(stat::Var(t.clone())), c.clone()));
            for e in list {
                let mut clo = t.clone();
                clo.set_ident(e);
                self.trees.push(tree::new(root::stat(stat::Var(clo)), c.clone()));
            }
            l.eat_termin(e);
            return;
        }

        // type separator ':'
        if matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::colon_)) {
            l.bump();

            // ERROR if not 'space'
            if !(matches!(l.curr().key(), KEYWORD::void(VOID::space_))) {
                l.space_add_report(l.prev().key().to_string(), e); return;
            } else { l.eat_space(e); }

            // ERROR if not any 'type'
            if !(matches!(l.curr().key(), KEYWORD::types(_))) {
                l.unexpect_report(KEYWORD::types(TYPE::ANY).to_string(), e); return;
            } else { self.retypes_stat(l, e); }

            // self.retypes_stat(l, e);

            // self.help_assign_options(&mut options, l, e);
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
            let el;
            match l.curr().key() {
                KEYWORD::option(OPTION::mut_) => { el = assign_opts::Mut }
                KEYWORD::option(OPTION::sta_) => { el = assign_opts::Sta }
                KEYWORD::option(OPTION::exp_) => { el = assign_opts::Exp }
                KEYWORD::option(OPTION::hid_) => { el = assign_opts::Hid }
                KEYWORD::option(OPTION::hep_) => { el = assign_opts::Hep }
                _ => {
                    l.unexpect_report(KEYWORD::option(OPTION::ANY).to_string(), e);
                    return
                }
            };
            v.push(el);
            l.bump();
            return
        }
        let deep = l.curr().loc().deep() -1;
        l.bump();
        loop {
            //TODO: finish options
            if ( matches!(l.curr().key(), KEYWORD::symbol(SYMBOL::squarC_)) && l.curr().loc().deep() == deep )
                || l.curr().key().is_eof() { break }
            l.bump();
        }
        l.bump();
    }
}


//------------------------------------------------------------------------------------------------------//
//                                             TYPE STATEMENT                                           //
//------------------------------------------------------------------------------------------------------//
impl forest {
    pub fn retypes_stat(&mut self, l: &mut lexer::BAG, e: &mut err::FLAW) -> Option<Box<root>> {
        l.log(">>");
        match l.curr().key() {
            KEYWORD::types(TYPE::int_) => { return self.retypes_int_stat(l, e) }
            _ => { return None; }
        }
    }
    pub fn retypes_int_stat(&mut self, l: &mut lexer::BAG, e: &mut err::FLAW) -> Option<Box<root>> {
        None
    }
}
