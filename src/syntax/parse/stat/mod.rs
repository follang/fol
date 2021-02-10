use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub mod datatype;
pub use crate::syntax::parse::stat::datatype::*;
pub mod assign;
pub use crate::syntax::parse::stat::assign::*;
pub mod ident;
pub use crate::syntax::parse::stat::ident::*;

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
