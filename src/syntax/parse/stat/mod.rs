use crate::types::Vod;
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub mod contracts;
pub mod datatype;
pub mod assign;
pub mod ident;
pub use crate::syntax::parse::stat::{
        contracts::*,
        datatype::*,
        assign::*,
        ident::*,
};

pub struct ParserStat {
    pub nodes: Nodes,
    _source: Source,
}

impl ParserStat {
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src } 
    }
}
impl Parse for ParserStat {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if matches!(lex.curr(false)?.key(), KEYWORD::assign(_))
            || (matches!(lex.curr(false)?.key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false)?.key(), KEYWORD::assign(_)))
        {
            let mut parse_ass = ParserStatAss::init(self._source.clone());
            parse_ass.parse(lex)?;
            self.nodes.extend(parse_ass.nodes);
            return Ok(())
        }
        lex.until_term(true)?;
        Ok(())
    }
}
