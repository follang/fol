use crate::types::Vod;
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, eater};

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
    pub nodes: Nodes,
}

impl ParserStat {
    pub fn init() -> Self {
        Self { nodes: Nodes::new()} 
    }
}
impl Parse for ParserStat {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if lex.curr(true)?.key().is_assign()
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_)) && lex.peek(0, true)?.key().is_assign())
        {
            let mut parse_ass = ParserStatAss::init();
            parse_ass.parse(lex)?;
            self.nodes.extend(parse_ass.nodes);
            return Ok(())
        }
        lex.until_term(false)?;
        Ok(())
    }
}
