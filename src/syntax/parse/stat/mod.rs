use crate::types::Vod;

use crate::syntax::nodes::Nodes;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, Body};

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
    _style: Body,
}

impl ParserStat {
    pub fn init() -> Self {
        Self { nodes: Nodes::new(), _style: Body::Top} 
    }
    pub fn style(&mut self, style: Body) {
        self._style = style;
    }
}
impl Parse for ParserStat {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if lex.curr(true)?.key().is_assign()
            || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign())
        {
            let mut parse_ass = ParserStatAss::init();
            parse_ass.parse(lex)?;
            self.nodes.extend(parse_ass.nodes);
            return Ok(())
        } else {
            if let Err(_) = check::expect(lex,  KEYWORD::Illegal, true) {
                // lex.debug(true, 0);
                lex.until_term(false)?;
                // return Err(e)
            }
        }
        Ok(())
    }
}
