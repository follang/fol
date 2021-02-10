use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::nodes::stat::ident::*;

pub struct ParserStatIdent {
    pub nodes: Nodes,
    _source: Source,
}

impl ParserStatIdent {
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src } 
    }
}
impl Parse for ParserStatIdent {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        // loop {
            // lex.jump(0, false)?;
            lex.debug().ok();
            lex.expect( KEYWORD::ident , true)?;
            let identnode = NodeStatIdent::default();
            // if let KEYWORD::ident = lex.curr(true).key() {
            //     let assopt: AssOptsTrait = a.into();
            //     let node = Node::new(lex.curr(true).loc().clone(), Box::new(assopt));
            //     self.nodes.push(node);
            // }
        // }
        Ok(())
    }
}
