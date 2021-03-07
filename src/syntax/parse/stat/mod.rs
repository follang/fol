use crate::types::{Vod, Errors};

use crate::syntax::nodes::Nodes;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, eater, branch, Body};

pub mod parameters;
pub mod generics;
pub mod datatype;
pub mod assign;
pub mod ident;
pub use crate::syntax::parse::stat::{
    parameters::*,
    generics::*,
    datatype::*,
    assign::*,
    ident::*,
};

pub struct ParserStat {
    nodes: Nodes,
    errors: Errors,
    style: Body,
    level: usize,
}

impl ParserStat {
    pub fn init(style: Body, level: usize) -> Self {
        Self { nodes: Nodes::new(), errors: Vec::new(), style, level} 
    }
    pub fn style(&mut self, style: Body) {
        self.style = style;
    }
}
impl Parse for ParserStat {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { self.errors.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let locus = lex.curr(false)?.loc().deep();
        while let Some(_) = lex.bump() {
            match self.style {
                Body::Top => {
                    if let Err(err) = self.parse_top(lex) { self.errors.push(err) }
                    if lex.curr(true)?.key().is_eof() { break }
                },
                Body::Typ => {
                    if let Err(err) = self.parse_typ(lex) { self.errors.push(err) }
                    if lex.curr(false)?.loc().deep() == locus - 1 { break }
                },
                Body::Imp => {
                    if let Err(err) = self.parse_imp(lex) { self.errors.push(err) }
                    if lex.curr(false)?.loc().deep() == locus - 1 { break }
                },
                Body::Fun => {
                    if let Err(err) = self.parse_fun(lex) { self.errors.push(err) }
                    if lex.curr(false)?.loc().deep() == locus - 1 { break }
                },
            }
        }
        Ok(())
    }
}

impl ParserStat {
    fn parse_top(&mut self, lex: &mut lexer::Elements) -> Vod {
        let token = lex.curr(true)?; lex.eat();
        if (lex.curr(true)?.key().is_assign()
            || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
            && branch::body_top(lex, true)? 
        {
            let mut parser = ParserStatAss::init();
            if let Err(err) = parser.parse(lex) { self.errors.push(err) }
            self.nodes.extend(parser.nodes());
            self.errors.extend(parser.errors());
        }
        else if lex.curr(false)?.key().is_void() { return Ok(()); } 
        // else if matches!(lex.curr(true)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) { return Ok(()); } 
        else if let Err(err) = check::unexpected_top(lex, token) { self.errors.push(err) }
        return Ok(());
    }

    fn parse_typ(&mut self, lex: &mut lexer::Elements) -> Vod {
        let token = lex.curr(true)?; lex.eat();
        if (lex.curr(true)?.key().is_assign()
            || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
            && branch::body_typ(lex, true)? 
        {
            let mut parser = ParserStatAss::init();
            if let Err(err) = parser.parse(lex) { self.errors.push(err) }
            self.nodes.extend(parser.nodes());
            self.errors.extend(parser.errors());
        }
        else if lex.curr(false)?.key().is_void() { return Ok(()); } 
        // else if matches!(lex.curr(true)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) { return Ok(()); } 
        else if let Err(err) = check::unexpected_typ(lex, token) { self.errors.push(err) }
        return Ok(());
    }

    fn parse_imp(&mut self, lex: &mut lexer::Elements) -> Vod {
        let token = lex.curr(true)?; lex.eat();
        if (lex.curr(true)?.key().is_assign()
            || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
            && branch::body_imp(lex, true)? 
        {
            let mut parser = ParserStatAss::init();
            if let Err(err) = parser.parse(lex) { self.errors.push(err) }
            self.nodes.extend(parser.nodes());
            self.errors.extend(parser.errors());
        }
        else if lex.curr(false)?.key().is_void() { return Ok(()); } 
        // else if matches!(lex.curr(true)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) { return Ok(()); } 
        else if let Err(err) = check::unexpected_imp(lex, token) { self.errors.push(err) }
        return Ok(());
    }

    fn parse_fun(&mut self, lex: &mut lexer::Elements) -> Vod {
        // let token = lex.curr(true)?; lex.eat();
        if (lex.curr(true)?.key().is_assign()
            || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
            && branch::body_fun(lex, true)? 
        {
            let mut parser = ParserStatAss::init();
            if let Err(err) = parser.parse(lex) { self.errors.push(err) }
            self.nodes.extend(parser.nodes());
            self.errors.extend(parser.errors());
        }
        else if lex.curr(false)?.key().is_void() { return Ok(()); } 
        // else if matches!(lex.curr(true)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) { return Ok(()); } 
        else { eater::until_term(lex, false)?; return Ok(()) }
        // else if let Err(err) = check::unexpected_fun(lex, token) { self.errors.push(err) }
        return Ok(());
    }
}

