use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub mod var_stat;
pub use crate::syntax::parse::statement::var_stat::*;

pub struct StatParser {
    pub nodes: Nodes,
}
impl std::default::Default for StatParser {
    fn default() -> Self { Self { nodes: Nodes::new() } }
}

impl Parse for StatParser {
    fn parse(&mut self, mut lex: &mut lexer::Elements) -> Vod {
        if matches!(lex.curr(false).key(), KEYWORD::assign(ASSIGN::var_))
            || (matches!(lex.curr(false).key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false).key(), KEYWORD::assign(ASSIGN::var_)))
        {
            let mut parse_var = VarStatParser::default();
            parse_var.parse(&mut lex)?;
            self.nodes.extend(parse_var.nodes);
        } else if matches!(lex.curr(false).key(), KEYWORD::assign(ASSIGN::var_))
            || (matches!(lex.curr(false).key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false).key(), KEYWORD::assign(ASSIGN::var_)))
        {
        }
        Ok(())
    }
}
