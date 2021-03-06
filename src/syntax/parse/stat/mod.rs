use crate::types::Vod;

use crate::syntax::nodes::Nodes;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, branch, Body};

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
        Self { nodes: Nodes::new(), _style: Body::Fun} 
    }
    pub fn style(&mut self, style: Body) {
        self._style = style;
    }
}
impl Parse for ParserStat {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let token = lex.curr(true)?;
        match self._style {
            Body::Top => {
                if (lex.curr(true)?.key().is_assign()
                    || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
                    && branch::body_top(lex, true)? {
                    let mut parse_ass = ParserStatAss::init();
                    parse_ass.parse(lex)?;
                    self.nodes.extend(parse_ass.nodes);
                    return Ok(())
                } else { return check::unexpected_top(lex, token); }
            }
            Body::Typ => {
                if (lex.curr(true)?.key().is_assign()
                    || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
                    && branch::body_typ(lex, true)? {
                    let mut parse_ass = ParserStatAss::init();
                    parse_ass.parse(lex)?;
                    self.nodes.extend(parse_ass.nodes);
                    return Ok(())
                } else { return check::unexpected_typ(lex, token); }
            }
            Body::Imp => {
                if (lex.curr(true)?.key().is_assign()
                    || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
                    && branch::body_imp(lex, true)? {
                    let mut parse_ass = ParserStatAss::init();
                    parse_ass.parse(lex)?;
                    self.nodes.extend(parse_ass.nodes);
                    return Ok(())
                } else { return check::unexpected_imp(lex, token); }
            }
            Body::Fun => {
                if (lex.curr(true)?.key().is_assign()
                    || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
                    && branch::body_fun(lex, true)? {
                    let mut parse_ass = ParserStatAss::init();
                    parse_ass.parse(lex)?;
                    self.nodes.extend(parse_ass.nodes);
                    return Ok(())
                } else { return check::unexpected_fun(lex, token); }
            }
        }
    }
}
