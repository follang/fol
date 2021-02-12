use crate::types::Vod;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub mod opts;
pub mod var;
pub mod typ;
pub mod ali;
pub mod r#use;
use crate::syntax::parse::stat::assign::{
    opts::ParserStatAssOpts,
    var::ParserStatAssVar,
    typ::ParserStatAssTyp,
    ali::ParserStatAssAli,
    r#use::ParserStatAssUse,
};

pub struct ParserStatAss {
    pub nodes: Nodes,
    _source: Source,
}

impl ParserStatAss {
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src } 
    }
}
impl Parse for ParserStatAss {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if matches!(lex.curr(false)?.key(), KEYWORD::assign(ASSIGN::var_))
            || (matches!(lex.curr(false)?.key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false)?.key(), KEYWORD::assign(ASSIGN::var_)))
        {
            let mut parser = ParserStatAssVar::init(self._source.clone());
            parser.parse(lex)?;
            self.nodes.extend(parser.nodes());
            return Ok(())
        } else if matches!(lex.curr(false)?.key(), KEYWORD::assign(ASSIGN::typ_))
            || (matches!(lex.curr(false)?.key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false)?.key(), KEYWORD::assign(ASSIGN::typ_)))
        {
            let mut parser = ParserStatAssTyp::init(self._source.clone());
            parser.parse(lex)?;
            self.nodes.extend(parser.nodes());
            return Ok(())
        } else if matches!(lex.curr(false)?.key(), KEYWORD::assign(ASSIGN::ali_))
            || (matches!(lex.curr(false)?.key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false)?.key(), KEYWORD::assign(ASSIGN::ali_)))
        {
            let mut parser = ParserStatAssAli::init(self._source.clone());
            parser.parse(lex)?;
            self.nodes.extend(parser.nodes());
            return Ok(())
        } else if matches!(lex.curr(false)?.key(), KEYWORD::assign(ASSIGN::use_))
            || (matches!(lex.curr(false)?.key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false)?.key(), KEYWORD::assign(ASSIGN::use_)))
        {
            let mut parser = ParserStatAssUse::init(self._source.clone());
            parser.parse(lex)?;
            self.nodes.extend(parser.nodes());
            return Ok(())
        } else {
            lex.until_term(true)?;
            return Ok(())
        }
    }
}
