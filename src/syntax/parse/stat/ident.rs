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
        // loop {
            lex.expect( KEYWORD::ident , true)?;
            // if let KEYWORD::ident = lex.curr(true).key() {
            //     let assopt: AssOptsTrait = a.into();
            //     let node = Node::new(lex.curr(true).loc().clone(), Box::new(assopt));
            //     self.nodes.push(node);
            // }
        // }
        Ok(())
    }
}
