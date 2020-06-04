#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::parsing::lexer;
use crate::parsing::ast::*;
use crate::parsing::stat::var;
use crate::parsing::stat::typ;

use crate::scanning::token::*;
use crate::scanning::locate;
use crate::error::flaw;
use colored::Colorize;

use crate::error::flaw::Con;

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
            if let Err(e) = self.parse_tree(lex, flaw) { lex.to_endline(flaw); lex.eat_termin(flaw); };
        }
    }
    pub fn parse_tree(&mut self, lex: &mut lexer::BAG, flaw: &mut flaw::FLAW) -> Con<()> {
        if lex.prev().key().is_terminal() || lex.prev().key().is_eof() {
            if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::var_) ) ||
                ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::var_) ) ) {
                var::parse_stat_var(self, lex, flaw, None)?;
            // } else if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::fun_) ) ||
                // ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::fun_) ) ) {
            // } else if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::pro_) ) ||
                // ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::pro_) ) ) {
            // } else if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::log_) ) ||
                // ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::log_) ) ) {
            } else if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::typ_) ) ||
                ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::typ_) ) ) {
                typ::parse_stat_typ(self, lex, flaw, None)?;
            // } else if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::ali_) ) ||
                // ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::ali_) ) ) {
            // } else if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::use_) ) ||
                // ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::use_) ) ) {
            // } else if matches!( lex.curr().key(), KEYWORD::assign(ASSIGN::def_) ) ||
                // ( matches!( lex.curr().key(), KEYWORD::option(_) ) && matches!( lex.next().key(), KEYWORD::assign(ASSIGN::def_) ) ) {
            } else {
                lex.to_endline(flaw); lex.eat_termin(flaw);
            }
        }
        Ok(())
    }
}
