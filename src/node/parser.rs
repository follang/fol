#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_macros)]

use std::fmt;
use crate::node::lexer;
use crate::node::ast::*;
use crate::scan::token::*;
use crate::scan::locate;
use crate::error::flaw;
use colored::Colorize;


pub struct forest {
    pub trees: Vec<tree>
}

pub fn new() -> forest {
    forest{ trees: Vec::new() }
}

impl forest {
    pub fn init(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) {
        if !flaw.list().is_empty() { return; }
        let f = self::new();
        while lex.not_empty() {
            self.parse_stat(lex, flaw);
        }
    }
    pub fn parse_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) {
        if lex.prev().key().is_terminal() || lex.prev().key().is_eof() {
            if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::var_) ) ||
                ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::var_) ) ) {
                self.parse_stat_var(lex, flaw, &mut var_stat::init(), false);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::fun_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::fun_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::pro_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::pro_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::log_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::log_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::typ_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::typ_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::ali_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::ali_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::use_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::use_) ) ) {
                // self.parse_stat_var(l, flaw);
            // } else if matches!( l.curr().key(), KEYWORD::assign(ASSIGN::def_) ) ||
                // ( matches!( l.curr().key(), KEYWORD::symbol(_) ) && matches!( l.next().key(), KEYWORD::assign(ASSIGN::def_) ) ) {
                // self.parse_stat_var(l, flaw);
            } else {
                lex.to_endline(flaw);
                lex.eat_termin(flaw);
            }
        }
    }
}


//------------------------------------------------------------------------------------------------------//
//                                             VAR STATEMENT                                            //
//------------------------------------------------------------------------------------------------------//
impl forest {
    pub fn parse_stat_var(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW, mut stat: &mut var_stat, group: bool) {
        let c = lex.curr().loc().clone();
        let mut options: Vec<assign_opts> = Vec::new();
        let mut list: Vec<String> = Vec::new();

        if !group {
            // option symbol
            if matches!(lex.curr().key(), KEYWORD::option(_)) {
                self.help_assign_options(&mut options, lex, flaw);
            }

            // eat assign var
            lex.bump();

            // option elements
            if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
                self.help_assign_options(&mut options, lex, flaw);
            }
            stat.set_options(options);

            // ERROR if not 'space'
            if !(matches!(lex.curr().key(), KEYWORD::void(VOID::space_))) {
                lex.space_add_report(lex.prev().key().to_string(), lex.prev().loc().clone(), flaw); return;
            } else { lex.eat_space(flaw); }

            // ERROR if '[' because is a little late for that
            if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
                lex.space_rem_report(lex.past().key().to_string(), lex.prev().loc().clone(), flaw); return;
            }

            // group variables matching "("
            if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundO_)) {
                lex.bump(); lex.eat_space(flaw);
                while matches!(lex.curr().key(), KEYWORD::ident(_)) {
                    self.parse_stat_var(lex, flaw, &mut stat, true);
                    lex.eat_termin(flaw);
                }
                if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::roundC_)) {
                    lex.bump();
                } else {
                    lex.unexpect_report(KEYWORD::symbol(SYMBOL::roundC_).to_string(), lex.curr().loc().clone(), flaw);
                    return
                }
                if lex.curr().key().is_terminal() {
                    lex.eat_termin(flaw);
                } else {
                    lex.unexpect_report(KEYWORD::void(VOID::endline_).to_string(), lex.curr().loc().clone(), flaw);
                    return
                }
                return
            }
        }

        //identifier
        if matches!(lex.curr().key(), KEYWORD::ident(_)) {
            stat.set_ident(lex.curr().con().clone());
            lex.bump();
        } else {
            lex.unexpect_report(KEYWORD::ident(String::new()).to_string(), lex.curr().loc().clone(), flaw);
            return
        }

        // list variables
        while matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::comma_)) {
            lex.jump();
            if matches!(lex.look().key(), KEYWORD::ident(_)) {
                lex.eat_space(flaw);
                list.push(lex.curr().con().clone());
                lex.jump();
            }
        }

        // type separator ':'
        if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::colon_)) {
            lex.bump();

            // ERROR if not 'space'
            if !(matches!(lex.curr().key(), KEYWORD::void(VOID::space_))) {
                lex.space_add_report(lex.prev().key().to_string(), lex.prev().loc().clone(), flaw);
                return;
            } else { lex.eat_space(flaw); }

            // ERROR if not any 'type'
            if !(matches!(lex.curr().key(), KEYWORD::types(_))) {
                lex.unexpect_report(KEYWORD::types(TYPE::ANY).to_string(), lex.curr().loc().clone(), flaw);
                return;

            // types
            } else {
                self.parse_type_stat(lex, flaw);
            }
        }

        // list types
        while matches!(lex.look().key(), KEYWORD::symbol(SYMBOL::comma_)) {
            lex.jump();
            if matches!(lex.look().key(), KEYWORD::types(_)) {
                lex.eat_space(flaw);
                self.parse_type_stat(lex, flaw);
            }
        }

        // short version (no type)
        if lex.look().key().is_terminal(){
            self.trees.push(tree::new(root::stat(stat::Var(stat.clone())), c.clone()));
            for e in list {
                let mut clo = stat.clone();
                clo.set_ident(e);
                self.trees.push(tree::new(root::stat(stat::Var(clo)), c.clone()));
            }
            lex.eat_termin(flaw);
            return;
        }
        lex.log(">>>");

        // now is the equal

        stat.set_body(self.parse_expr_var(lex, flaw));

        self.trees.push(tree::new(root::stat(stat::Var(stat.clone())), c.clone()));
        for e in list {
            let mut clo = stat.clone();
            clo.set_ident(e);
            self.trees.push(tree::new(root::stat(stat::Var(clo)), c.clone()));
        }

    }
    pub fn help_assign_options(&mut self, v: &mut Vec<assign_opts>, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) {
        if matches!(lex.curr().key(), KEYWORD::option(_)) {
            let el;
            match lex.curr().key() {
                KEYWORD::option(OPTION::mut_) => { el = assign_opts::Mut }
                KEYWORD::option(OPTION::sta_) => { el = assign_opts::Sta }
                KEYWORD::option(OPTION::exp_) => { el = assign_opts::Exp }
                KEYWORD::option(OPTION::hid_) => { el = assign_opts::Hid }
                KEYWORD::option(OPTION::hep_) => { el = assign_opts::Hep }
                _ => {
                    lex.unexpect_report(KEYWORD::option(OPTION::ANY).to_string(), lex.curr().loc().clone(), flaw);
                    return
                }
            };
            v.push(el);
            lex.bump();
            return
        }
        let deep = lex.curr().loc().deep() -1;
        lex.bump();
        loop {
            //TODO: finish options
            if ( matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarC_)) && lex.curr().loc().deep() == deep )
                || lex.curr().key().is_eof() { break }
            lex.bump();
        }
        lex.bump();
    }
    pub fn parse_expr_var(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<root>> {
        lex.to_endline(flaw);
        lex.eat_termin(flaw);
        None
    }
}


//------------------------------------------------------------------------------------------------------//
//                                             TYPE STATEMENT                                           //
//------------------------------------------------------------------------------------------------------//
impl forest {
    pub fn parse_type_stat(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<root>> {
        match lex.curr().key() {
            _ => {
                lex.bump();
                if matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarO_)) {
                    let deep = lex.curr().loc().deep() -1;
                    loop {

                        //TODO: finish types
                        if ( matches!(lex.curr().key(), KEYWORD::symbol(SYMBOL::squarC_)) && lex.curr().loc().deep() == deep )
                            || lex.curr().key().is_eof() { break }
                        lex.bump();
                    }
                    lex.bump();
                }
                return None;
            }
        }
    }
    pub fn retypes_int_stat(&mut self, l: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Option<Box<root>> {
        None
    }
}
