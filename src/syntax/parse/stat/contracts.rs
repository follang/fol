use crate::types::Vod;
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::check;

pub use crate::syntax::nodes::stat::ident::*;

pub struct ParserStatContract {
    pub nodes: Nodes,
}

impl ParserStatContract {
    pub fn init() -> Self {
        Self { nodes: Nodes::new() } 
    }
}
impl Parse for ParserStatContract {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        // eat "("
        lex.jump(0, false)?;

        // match ")" if there and return
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundC_) {
            lex.jump(0, true)?;
            return Ok(())
        }
        while !lex.curr(true)?.key().is_eof() {
            check::expect(lex, KEYWORD::ident , true)?; lex.eat();
            let identnode = NodeStatIdent::new(lex.curr(false)?.con().clone());
            self.nodes.push(Node::new(Box::new(identnode)));
            lex.jump(0, true)?;
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundC_) 
            {
                lex.jump(0, true)?;
                break
            } else if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::comma_) {
                lex.jump(0, true)?; lex.eat();
            }
        }
        Ok(())
    }
}
