use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::nodes::stat::ident::*;

pub struct ParserStatContract {
    pub nodes: Nodes,
    _source: Source,
}

impl ParserStatContract {
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src } 
    }
}
impl Parse for ParserStatContract {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        lex.debug().ok();
        // eat "("
        lex.jump(0, false)?;

        // match ")" if there and return
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundC_) {
            lex.jump(0, true)?;
            return Ok(())
        }
        while !lex.curr(true)?.key().is_eof() {
            lex.expect( KEYWORD::ident , true)?; lex.eat();
            let identnode = NodeStatIdent::new(lex.curr(false)?.con().clone());
            self.nodes.push(Node::new(Box::new(identnode)));
            lex.jump(0, true)?;
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) 
                || lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::equal_)
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
