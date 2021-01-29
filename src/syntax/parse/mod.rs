use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;

pub mod statement;
pub use crate::syntax::parse::statement::*;
pub mod expression;

pub trait Parse {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Con<Nodes>;
}

pub struct Parser {
    pub nodes: Nodes,
    pub errors: Errors
}
impl std::default::Default for Parser {
    fn default() -> Self { Self { nodes: Vec::new(), errors: Vec::new() } }
}

impl Parser {
    pub fn parse(&mut self, mut lex: &mut lexer::Elements) {
        if let Some(val) = lex.bump() { if let Err(e) = val { crash!(e) }; };
        if matches!(lex.curr(false).key(), KEYWORD::assign(_))
            || (matches!(lex.curr(false).key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false).key(), KEYWORD::assign(_)))
        {
            let parse_stat = StatParser::default().parse(&mut lex);
            match parse_stat {
                Ok(val) => { self.nodes.extend(val) },
                Err(err) => { self.errors.push(err) }
            }
        }
    }
}
