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
    _once: bool,
}

impl ParserStatIdent {
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src, _once: false } 
    }
    pub fn only_one(&mut self) { self._once = true; }
}
impl Parse for ParserStatIdent {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        while !lex.curr(true)?.key().is_eof() {
            lex.expect( KEYWORD::ident , true)?; lex.eat();
            let identnode = NodeStatIdent::new(lex.curr(false)?.con().clone());
            self.nodes.push(Node::new(Box::new(identnode)));
            lex.jump(0, true)?;
            if self._once { return Ok(()) }
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) 
                || lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::equal_)
                || lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundO_)
                || lex.curr(true)?.key().is_terminal()
            {
                break
            } else if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::comma_) {
                lex.jump(0, true)?; lex.eat();
            }
        }
        Ok(())
    }
}
