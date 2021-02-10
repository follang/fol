use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub mod opts;
pub use crate::syntax::parse::stat::assign::opts::*;
pub mod var;
pub use crate::syntax::parse::stat::assign::var::*;

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
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if matches!(lex.curr(false)?.key(), KEYWORD::assign(ASSIGN::var_))
            || (matches!(lex.curr(false)?.key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false)?.key(), KEYWORD::assign(ASSIGN::var_)))
        {
            let mut parse_var = ParserStatAssVar::init(self._source.clone());
            parse_var.parse(lex)?;
            self.nodes.extend(parse_var.nodes);
            return Ok(())
        }
        lex.until_term(true)?;
        Ok(())
    }
}
