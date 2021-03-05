use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::check;

pub use crate::syntax::nodes::stat::ident::*;

pub struct ParserStatIdent {
    pub nodes: Nodes,
    _once: bool,
}

impl ParserStatIdent {
    pub fn init() -> Self {
        Self { nodes: Nodes::new(), _once: false } 
    }
    pub fn once(&mut self) { self._once = true; }
}
impl Parse for ParserStatIdent {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        while !lex.curr(true)?.key().is_eof() {
            check::expect_ident(lex, true)?; lex.eat();
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

impl ParserStatIdent {
    pub fn parse_2(&mut self, lex: &mut lexer::Elements) -> Vod {
        check::expect_ident(lex, true)?; lex.eat();
        let identnode = NodeStatIdent::new(lex.curr(false)?.con().clone());
        self.nodes.push(Node::new(Box::new(identnode)));
        lex.jump(0, true)?;
        Ok(())
    }
}
