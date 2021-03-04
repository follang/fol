use crate::types::Vod;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, eater};

pub mod opts;
pub mod var;
pub mod typ;
pub mod imp;
pub mod ali;
pub mod r#use;
pub mod fun;
pub mod lab;
use crate::syntax::parse::stat::assign::{
    opts::ParserStatAssOpts,
    var::ParserStatAssVar,
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
        if matches!(lex.curr(true)?.key(), KEYWORD::assign(ASSIGN::var_))
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_))
                && matches!(lex.peek(0, true)?.key(), KEYWORD::assign(ASSIGN::var_)))
        {
            parser = Box::new(ParserStatAssVar::init());
        } else if matches!(lex.curr(true)?.key(), KEYWORD::assign(ASSIGN::typ_))
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_))
                && matches!(lex.peek(0, true)?.key(), KEYWORD::assign(ASSIGN::typ_)))
        {
            parser = Box::new(ParserStatAssTyp::init());
        } else if matches!(lex.curr(true)?.key(), KEYWORD::assign(ASSIGN::ali_))
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_))
                && matches!(lex.peek(0, true)?.key(), KEYWORD::assign(ASSIGN::ali_)))
        {
            parser = Box::new(ParserStatAssAli::init());
        } else if matches!(lex.curr(true)?.key(), KEYWORD::assign(ASSIGN::use_))
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_))
                && matches!(lex.peek(0, true)?.key(), KEYWORD::assign(ASSIGN::use_)))
        {
            parser = Box::new(ParserStatAssUse::init());
        } else if matches!(lex.curr(true)?.key(), KEYWORD::assign(ASSIGN::pro_))
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_))
                && matches!(lex.peek(0, true)?.key(), KEYWORD::assign(ASSIGN::pro_)))
        {
            parser = Box::new(ParserStatAssFun::init());
        } else if matches!(lex.curr(true)?.key(), KEYWORD::assign(ASSIGN::fun_))
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_))
                && matches!(lex.peek(0, true)?.key(), KEYWORD::assign(ASSIGN::fun_)))
        {
            parser = Box::new(ParserStatAssFun::init());
        } else if matches!(lex.curr(true)?.key(), KEYWORD::assign(ASSIGN::imp_))
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_))
                && matches!(lex.peek(0, true)?.key(), KEYWORD::assign(ASSIGN::imp_)))
        {
            parser = Box::new(ParserStatAssImp::init());
        } else if matches!(lex.curr(true)?.key(), KEYWORD::assign(ASSIGN::lab_))
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_))
                && matches!(lex.peek(0, true)?.key(), KEYWORD::assign(ASSIGN::lab_)))
        {
            parser = Box::new(ParserStatAssLab::init());
        } else {
            check::expect(lex,  KEYWORD::assign(ASSIGN::ANY) , true)?;
            lex.until_term(true)?;
            return Ok(())
        }
        parser.parse(lex)?;
        self.nodes.extend(parser.nodes());
        Ok(())
    }
}
