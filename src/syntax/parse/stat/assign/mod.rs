// #![allow(unused_imports)]

use crate::types::{Vod, Errors};
use crate::syntax::nodes::*;
use crate::syntax::lexer;
use super::Parse;
// use crate::types::*;

pub mod opts;
pub mod var;
pub mod con;
pub mod typ;
pub mod imp;
pub mod ali;
pub mod r#use;
pub mod fun;
pub mod lab;
use crate::syntax::parse::stat::assign::{
    var::ParserStatAssVar,
    con::ParserStatAssCon,
    typ::ParserStatAssTyp,
    imp::ParserStatAssImp,
    ali::ParserStatAssAli,
    r#use::ParserStatAssUse,
    fun::ParserStatAssFun,
    lab::ParserStatAssLab,
};

pub struct ParserStatAss {
    nodes: Nodes,
    errors: Errors,
    _level: usize,
}

impl ParserStatAss {
    pub fn init(level: usize) -> Self {
        Self {
            nodes: Nodes::new(),
            errors: Vec::new(),
            _level: level,
        }
    }
    pub fn level(&self) -> usize { self._level }
}
impl Parse for ParserStatAss {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { self.errors.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut parser: Box<dyn Parse>;
        if lex.curr(true)?.con() == "var" || lex.peek(0, true)?.con() == "var" {
            parser = Box::new(ParserStatAssVar::init(self.level()));
        } else if lex.curr(true)?.con() == "con" || lex.peek(0, true)?.con() == "con" {
            parser = Box::new(ParserStatAssCon::init(self.level()));
        } else if lex.curr(true)?.con() == "typ" || lex.peek(0, true)?.con() == "typ" {
            parser = Box::new(ParserStatAssTyp::init(self.level()));
        } else if lex.curr(true)?.con() == "ali" || lex.peek(0, true)?.con() == "ali" {
            parser = Box::new(ParserStatAssAli::init(self.level()));
        } else if lex.curr(true)?.con() == "use" || lex.peek(0, true)?.con() == "use" {
            parser = Box::new(ParserStatAssUse::init(self.level()));
        } else if lex.curr(true)?.con() == "pro" || lex.peek(0, true)?.con() == "pro" {
            parser = Box::new(ParserStatAssFun::init(self.level()));
        } else if lex.curr(true)?.con() == "fun" || lex.peek(0, true)?.con() == "fun" {
            parser = Box::new(ParserStatAssFun::init(self.level()));
        } else if lex.curr(true)?.con() == "imp" || lex.peek(0, true)?.con() == "imp" {
            parser = Box::new(ParserStatAssImp::init(self.level()));
        } else if lex.curr(true)?.con() == "lab" || lex.peek(0, true)?.con() == "lab" {
            parser = Box::new(ParserStatAssLab::init(self.level()));
        } else {
            //TODO: fix here
            // check::expect(lex,  KEYWORD::buildin(BUILDIN::ANY) , true)?;
            lex.until_term(true)?;
            return Ok(())
        }
        // lex.debug(false, 0).ok();
        // if let Err(err) = parser.parse(lex) { return Err(err) }
        if let Err(err) = parser.parse(lex) { self.errors.push(err) }
        self.nodes.extend(parser.nodes());
        self.errors.extend(parser.errors());
        Ok(())
    }
}
