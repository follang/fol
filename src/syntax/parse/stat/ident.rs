use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::nodes::stat::ident::*;

pub struct ParserStatIdent {
    pub nodes: Nodes,
}
impl std::default::Default for ParserStatIdent {
    fn default() -> Self { Self { nodes: Nodes::new() } }
}

impl Parse for ParserStatIdent {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        lex.expect( KEYWORD::ident , true)?;
        Ok(())
    }
}
