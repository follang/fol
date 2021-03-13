// #![allow(unused_imports)]

use crate::types::{Vod, Errors};
use crate::syntax::nodes::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::Body;

pub mod opts;
pub mod var;
pub mod con;
pub mod typ;
pub mod imp;
pub mod seg;
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
    seg::ParserStatAssSeg,
    fun::ParserStatAssFun,
    lab::ParserStatAssLab,
};


pub mod generics;
pub mod parameters;
// use crate::syntax::parse::stat::assign::{
    // parameters::ParserStatParameters,
    // generics::ParserStatGenerics,
// };

pub struct ParserStatAss {
    nodes: Nodes,
    errors: Errors,
    _level: usize,
    _style: Body,
}

impl ParserStatAss {
    pub fn init(level: usize, style: Body) -> Self {
        Self {
            nodes: Nodes::new(),
            errors: Vec::new(),
            _level: level,
            _style: style.clone(),
        }
    }
    pub fn level(&self) -> usize { self._level }
    pub fn style(&self) -> Body { self._style }
}
impl Parse for ParserStatAss {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { self.errors.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut parser: Box<dyn Parse>;
        if lex.curr(true)?.con() == "var" || lex.peek(0, true)?.con() == "var" {
            parser = Box::new(ParserStatAssVar::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "con" || lex.peek(0, true)?.con() == "con" {
            parser = Box::new(ParserStatAssCon::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "typ" || lex.peek(0, true)?.con() == "typ" {
            parser = Box::new(ParserStatAssTyp::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "ali" || lex.peek(0, true)?.con() == "ali" {
            parser = Box::new(ParserStatAssAli::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "use" || lex.peek(0, true)?.con() == "use" {
            parser = Box::new(ParserStatAssUse::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "pro" || lex.peek(0, true)?.con() == "pro" {
            parser = Box::new(ParserStatAssFun::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "itr" || lex.peek(0, true)?.con() == "itr" {
            parser = Box::new(ParserStatAssFun::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "fun" || lex.peek(0, true)?.con() == "fun" {
            parser = Box::new(ParserStatAssFun::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "seg" || lex.peek(0, true)?.con() == "seg" {
            parser = Box::new(ParserStatAssSeg::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "imp" || lex.peek(0, true)?.con() == "imp" {
            parser = Box::new(ParserStatAssImp::init(self.level(), self.style()));
        } else if lex.curr(true)?.con() == "lab" || lex.peek(0, true)?.con() == "lab" {
            parser = Box::new(ParserStatAssLab::init(self.level(), self.style()));
        } else { unimplemented!(); }

        if let Err(err) = parser.parse(lex) { self.errors.push(err) }
        self.nodes.extend(parser.nodes());
        self.errors.extend(parser.errors());
        Ok(())
    }
}
