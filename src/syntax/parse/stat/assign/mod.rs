use crate::types::*;
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
}
impl std::default::Default for ParserStatAss {
    fn default() -> Self { Self { nodes: Nodes::new() } }
}

impl Parse for ParserStatAss {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if matches!(lex.curr(false).key(), KEYWORD::assign(ASSIGN::var_))
            || (matches!(lex.curr(false).key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false).key(), KEYWORD::assign(ASSIGN::var_)))
        {
            let mut parse_var = ParserStatAssVar::default();
            parse_var.parse(lex)?;
            self.nodes.extend(parse_var.nodes);
            lex.jump(false);
            return Ok(())
        }
        lex.until_term();
        Ok(())
    }
}
