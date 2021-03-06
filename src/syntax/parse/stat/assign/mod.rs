use crate::types::Vod;
use crate::syntax::nodes::*;
use crate::syntax::lexer;
use super::Parse;

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
    pub nodes: Nodes,
}

impl ParserStatAss {
    pub fn init() -> Self {
        Self { nodes: Nodes::new() } 
    }
}
impl Parse for ParserStatAss {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut parser: Box<dyn Parse>;
        if lex.curr(true)?.con() == "var" || lex.peek(0, true)?.con() == "var" {
            parser = Box::new(ParserStatAssVar::init());
        } else if lex.curr(true)?.con() == "con" || lex.peek(0, true)?.con() == "con" {
            parser = Box::new(ParserStatAssCon::init());
        } else if lex.curr(true)?.con() == "typ" || lex.peek(0, true)?.con() == "typ" {
            parser = Box::new(ParserStatAssTyp::init());
        } else if lex.curr(true)?.con() == "ali" || lex.peek(0, true)?.con() == "ali" {
            parser = Box::new(ParserStatAssAli::init());
        } else if lex.curr(true)?.con() == "use" || lex.peek(0, true)?.con() == "use" {
            parser = Box::new(ParserStatAssUse::init());
        } else if lex.curr(true)?.con() == "pro" || lex.peek(0, true)?.con() == "pro" {
            parser = Box::new(ParserStatAssFun::init());
        } else if lex.curr(true)?.con() == "fun" || lex.peek(0, true)?.con() == "fun" {
            parser = Box::new(ParserStatAssFun::init());
        } else if lex.curr(true)?.con() == "imp" || lex.peek(0, true)?.con() == "imp" {
            parser = Box::new(ParserStatAssImp::init());
        } else if lex.curr(true)?.con() == "lab" || lex.peek(0, true)?.con() == "lab" {
            parser = Box::new(ParserStatAssLab::init());
        } else {
            //TODO: fix here
            // check::expect(lex,  KEYWORD::buildin(BUILDIN::ANY) , true)?;
            lex.until_term(true)?;
            return Ok(())
        }
        parser.parse(lex)?;
        self.nodes.extend(parser.nodes());
        Ok(())
    }
}
