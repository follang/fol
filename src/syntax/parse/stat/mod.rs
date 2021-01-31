use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub mod assign;
pub use crate::syntax::parse::stat::assign::*;

pub struct ParserStat {
    pub nodes: Nodes,
}
impl std::default::Default for ParserStat {
    fn default() -> Self { Self { nodes: Nodes::new() } }
}

impl Parse for ParserStat {
    fn parse(&mut self, mut lex: &mut lexer::Elements) -> Vod {
        if matches!(lex.curr(false).key(), KEYWORD::assign(_))
            || (matches!(lex.curr(false).key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false).key(), KEYWORD::assign(_)))
        {
            let mut parse_ass = ParserStatAss::default();
            parse_ass.parse(&mut lex)?;
            self.nodes.extend(parse_ass.nodes);
        }
        Ok(())
    }
}
